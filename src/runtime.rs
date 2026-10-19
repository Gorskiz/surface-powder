//! Ordered simulation on a worker; presentation never waits for a physics update.
use crate::sim::{Input, World};
use crate::{pour_world, save_world, Stroke};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, TryLockError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Default)]
struct ThreadUsage {
    cpu: Duration,
    preemptions: i64,
    minor_faults: i64,
    major_faults: i64,
}
#[cfg(target_os = "linux")]
fn thread_usage() -> Option<ThreadUsage> {
    let mut usage = std::mem::MaybeUninit::<sdl2::libc::rusage>::zeroed();
    if unsafe { sdl2::libc::getrusage(sdl2::libc::RUSAGE_THREAD, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    let usage = unsafe { usage.assume_init() };
    let time = |v: sdl2::libc::timeval| Duration::new(v.tv_sec as u64, v.tv_usec as u32 * 1000);
    Some(ThreadUsage {
        cpu: time(usage.ru_utime) + time(usage.ru_stime),
        preemptions: usage.ru_nivcsw,
        minor_faults: usage.ru_minflt,
        major_faults: usage.ru_majflt,
    })
}
#[cfg(not(target_os = "linux"))]
fn thread_usage() -> Option<ThreadUsage> {
    None
}

#[derive(Clone, Copy, Debug)]
pub enum Stress {
    Dense,
    Extreme,
}
impl Stress {
    pub fn inject(self, world: &mut World) {
        match self {
            Self::Dense => crate::inject_dense_explosion(world),
            Self::Extreme => {
                // 64 separated patches keep the blast cap busy across the whole scene.
                for gy in 0..8 {
                    for gx in 0..8 {
                        let cx = 25 + gx * 50;
                        let cy = 18 + gy * 37;
                        for y in cy - 10..cy + 10 {
                            for x in cx - 12..cx + 13 {
                                world.cells[y * world.width + x] =
                                    crate::sim::Cell::new(crate::sim::Material::Nitro);
                            }
                        }
                        world.apply_pressure(cx as i32, cy as i32, 24, 10.);
                    }
                }
            }
        }
    }
}

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
    pub fn start(mut world: World, stress: Option<Stress>) -> Self {
        let mut display = World::new(world.width, world.height);
        display.copy_visual_from(&world);
        let display = Arc::new(Mutex::new(display));
        let published = Arc::clone(&display);
        let (commands, receive) = mpsc::channel();
        let (notify, notices) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("powder-physics".into())
            .spawn(move || {
                world.prepare();
                let interval = Duration::from_nanos(16_666_667);
                let mut next = Instant::now() + interval;
                let mut input = Input::default();
                // The UI sends the first controls only after graphics initialization.
                let mut paused = true;
                let mut strokes = Vec::new();
                let mut radius = 3;
                let mut pending = None;
                let mut dirty = false;
                let mut updates = 0u64;
                let mut timings = Vec::with_capacity(240);
                let mut cpu_times = Vec::with_capacity(240);
                let mut max_wait = Duration::ZERO;
                let mut preemptions = 0;
                let mut minor_faults = 0;
                let mut major_faults = 0;
                let mut batch_start = Instant::now();
                'run: loop {
                    for command in pending.take().into_iter().chain(receive.try_iter()) {
                        match command {
                            Command::Edit(edit) => {
                                edit(&mut world);
                                dirty = true;
                            }
                            Command::Controls(i, p, s, r) => {
                                if paused && !p {
                                    // Start a fresh cadence after startup/resume, without catch-up ticks.
                                    next = Instant::now();
                                }
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
                        let usage = if stress.is_some() { thread_usage() } else { None };
                        if updates == 0 { batch_start = Instant::now(); }
                        let measured = stress.is_some().then(Instant::now);
                        for &stroke in &strokes {
                            pour_world(&mut world, stroke, radius);
                        }
                        if let Some(stress) = stress.filter(|_| updates % 60 == 0) {
                            // Refill gaps only in this diagnostic workload to sustain density.
                            for cell in &mut world.cells {
                                if cell.material == crate::sim::Material::Empty {
                                    *cell = crate::sim::Cell::new(crate::sim::Material::Water);
                                }
                            }
                            stress.inject(&mut world);
                        }
                        world.step(input);
                        updates += 1;
                        if let Some(start) = measured {
                            let elapsed = start.elapsed();
                            if let (Some(before), Some(after)) = (usage, thread_usage()) {
                                let cpu = after.cpu.saturating_sub(before.cpu);
                                let wait = elapsed.saturating_sub(cpu);
                                cpu_times.push(cpu);
                                max_wait = max_wait.max(wait);
                                preemptions += after.preemptions - before.preemptions;
                                minor_faults += after.minor_faults - before.minor_faults;
                                major_faults += after.major_faults - before.major_faults;
                                if elapsed > interval {
                                    eprintln!("powder: slow update wall={:.3}ms cpu={:.3}ms waiting={:.3}ms preemptions={} faults={}/{}",
                                        elapsed.as_secs_f64()*1000., cpu.as_secs_f64()*1000., wait.as_secs_f64()*1000.,
                                        after.preemptions-before.preemptions, after.minor_faults-before.minor_faults, after.major_faults-before.major_faults);
                                }
                            }
                            timings.push(elapsed);
                            if timings.len() == 240 {
                                timings.sort_unstable();
                                let ms = |d: Duration| d.as_secs_f64() * 1000.;
                                eprintln!("powder: stress physics rate={:.2}Hz mean={:.3}ms p95={:.3}ms max={:.3}ms late={}/240",
                                    240. / batch_start.elapsed().as_secs_f64(),
                                    timings.iter().map(|&d| ms(d)).sum::<f64>() / 240.,
                                    ms(timings[228]), ms(timings[239]),
                                    timings.iter().filter(|&&d| d > interval).count());
                                if !cpu_times.is_empty() {
                                    cpu_times.sort_unstable();
                                    eprintln!("powder: stress CPU mean={:.3}ms p95={:.3}ms max={:.3}ms waiting_max={:.3}ms preemptions={} faults={}/{}",
                                        cpu_times.iter().map(|&d| ms(d)).sum::<f64>() / cpu_times.len() as f64,
                                        ms(cpu_times[cpu_times.len()*95/100]), ms(*cpu_times.last().unwrap()), ms(max_wait), preemptions, minor_faults, major_faults);
                                    cpu_times.clear();
                                    max_wait = Duration::ZERO;
                                    preemptions = 0;
                                    minor_faults = 0;
                                    major_faults = 0;
                                }
                                timings.clear();
                                batch_start = Instant::now();
                            }
                        }
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
            })
            .expect("Could not start simulation worker");
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
    fn startup_waits_for_controls_without_advancing_physics() {
        let mut initial = World::new(32, 32);
        initial.paint_line(10, 4, 10, 4, 0, Material::Sand);
        let runtime = Runtime::start(initial, None);
        std::thread::sleep(Duration::from_millis(50));
        let world = runtime.finish().unwrap();
        assert_eq!(world.at(10, 4), Material::Sand);
        assert_eq!(world.at(10, 5), Material::Empty);
    }
    #[test]
    fn queued_edits_survive_shutdown_in_order() {
        let runtime = Runtime::start(World::new(32, 32), None);
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
        let runtime = Runtime::start(World::new(32, 32), None);
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
        let runtime = Runtime::start(World::new(32, 32), None);
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
