//! Ordered simulation on a worker; presentation never waits for a physics update.
use crate::sim::{Input, World};
use crate::{pour_world, save_world, Stroke};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, TryLockError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type Edit = Box<dyn FnOnce(&mut World) + Send>;
enum Command {
    Edit(Edit),
    Controls(Input, bool, Vec<Stroke>, i32),
    Save(PathBuf, bool),
    Stop,
}

pub struct Runtime {
    commands: mpsc::Sender<Command>,
    notices: mpsc::Receiver<String>,
    display: Arc<Mutex<World>>,
    thread: Option<JoinHandle<World>>,
}
impl Runtime {
    pub fn start(mut world: World) -> Self {
        let mut display = World::new(world.width, world.height);
        display.copy_visual_from(&world);
        let display = Arc::new(Mutex::new(display));
        let published = Arc::clone(&display);
        let (commands, receive) = mpsc::channel();
        let (notify, notices) = mpsc::channel();
        let thread = thread::spawn(move || {
            let interval = Duration::from_nanos(16_666_667);
            let mut next = Instant::now() + interval;
            let mut input = Input::default();
            let mut paused = false;
            let mut strokes = Vec::new();
            let mut radius = 3;
            let mut pending = None;
            let mut dirty = false;
            'run: loop {
                for command in pending.take().into_iter().chain(receive.try_iter()) {
                    match command {
                        Command::Edit(edit) => {
                            edit(&mut world);
                            dirty = true;
                        }
                        Command::Controls(i, p, s, r) => {
                            input = i;
                            paused = p;
                            strokes = s;
                            radius = r;
                        }
                        Command::Save(path, success) => {
                            let notice = save_world(&world, &path);
                            if success || notice == "SAVE FAILED" {
                                let _ = notify.send(notice);
                            }
                        }
                        Command::Stop => break 'run,
                    }
                }
                let now = Instant::now();
                if !paused && now >= next {
                    for &stroke in &strokes {
                        pour_world(&mut world, stroke, radius);
                    }
                    world.step(input);
                    dirty = true;
                    next += interval;
                    // A backlog never hides new input: service commands between every tick.
                    if next < now {
                        next = now;
                    }
                } else if paused {
                    next = now + interval;
                }
                if dirty {
                    if let Ok(mut snapshot) = published.try_lock() {
                        snapshot.copy_visual_from(&world);
                        dirty = false;
                    }
                }
                let wait = if dirty {
                    Duration::from_millis(1)
                } else if paused {
                    Duration::from_millis(100)
                } else {
                    next.saturating_duration_since(Instant::now())
                };
                match receive.recv_timeout(wait) {
                    Ok(command) => pending = Some(command),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            world
        });
        Self {
            commands,
            notices,
            display,
            thread: Some(thread),
        }
    }
    pub fn edit(&self, edit: impl FnOnce(&mut World) + Send + 'static) -> Result<(), String> {
        self.commands
            .send(Command::Edit(Box::new(edit)))
            .map_err(|_| "SIMULATION STOPPED".into())
    }
    pub fn controls(
        &self,
        input: Input,
        paused: bool,
        strokes: Vec<Stroke>,
        radius: i32,
    ) -> Result<(), String> {
        self.commands
            .send(Command::Controls(input, paused, strokes, radius))
            .map_err(|_| "SIMULATION STOPPED".into())
    }
    pub fn save(&self, path: PathBuf, success: bool) -> Result<(), String> {
        self.commands
            .send(Command::Save(path, success))
            .map_err(|_| "SAVE FAILED".into())
    }
    pub fn copy_latest(&self, visual: &mut World) -> Result<(), String> {
        if self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            return Err("Simulation worker stopped".into());
        }
        match self.display.try_lock() {
            Ok(snapshot) => visual.copy_visual_from(&snapshot),
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Poisoned(_)) => return Err("Simulation snapshot unavailable".into()),
        }
        Ok(())
    }
    pub fn notice(&self) -> Option<String> {
        self.notices.try_recv().ok()
    }
    pub fn finish(mut self) -> Result<World, String> {
        let _ = self.commands.send(Command::Stop);
        self.thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| "Simulation worker stopped".into())
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.commands.send(Command::Stop);
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Material;
    #[test]
    fn queued_edits_survive_shutdown_in_order() {
        let runtime = Runtime::start(World::new(32, 32));
        runtime
            .controls(Input::default(), true, Vec::new(), 3)
            .unwrap();
        runtime
            .edit(|w| w.paint_line(10, 10, 10, 10, 0, Material::Wall))
            .unwrap();
        runtime
            .edit(|w| w.paint_line(10, 10, 10, 10, 0, Material::Empty))
            .unwrap();
        runtime
            .edit(|w| w.paint_line(12, 12, 12, 12, 0, Material::Metal))
            .unwrap();
        let world = runtime.finish().unwrap();
        assert_eq!(world.at(10, 10), Material::Empty);
        assert_eq!(world.at(12, 12), Material::Metal);
    }
    #[test]
    fn presentation_snapshot_is_available_while_simulation_is_busy() {
        let runtime = Runtime::start(World::new(32, 32));
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        runtime
            .edit(move |_| {
                entered_tx.send(()).unwrap();
                let _ = release_rx.recv();
            })
            .unwrap();
        entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        let mut visual = World::new(32, 32);
        let result = runtime.copy_latest(&mut visual);
        release_tx.send(()).unwrap();
        assert!(result.is_ok());
        runtime.finish().unwrap();
    }
    #[test]
    fn save_acknowledgement_follows_pending_edits() {
        let runtime = Runtime::start(World::new(32, 32));
        runtime
            .controls(Input::default(), true, Vec::new(), 3)
            .unwrap();
        runtime
            .edit(|w| w.paint_line(8, 8, 8, 8, 0, Material::Wall))
            .unwrap();
        let path = std::env::temp_dir().join(format!("powder-worker-{}.bin", std::process::id()));
        runtime.save(path.clone(), true).unwrap();
        let notice = runtime
            .notices
            .recv_timeout(Duration::from_secs(3))
            .unwrap();
        assert_ne!(notice, "SAVE FAILED");
        assert_eq!(World::load(&path).unwrap().at(8, 8), Material::Wall);
        runtime.finish().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
