//! Ordered snapshots on the simulation thread; blocking I/O on a sleeping writer.
use crate::sim::World;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

struct Request {
    path: PathBuf,
    bytes: Vec<u8>,
    announce: bool,
}
#[derive(Default)]
struct Pending {
    requests: VecDeque<Request>,
    stopping: bool,
}
pub struct SaveWriter {
    pending: Arc<(Mutex<Pending>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}
impl SaveWriter {
    pub fn start(notices: mpsc::Sender<String>) -> Self {
        Self::with_writer(notices, |path, bytes| {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            World::write_save(bytes, path)
        })
    }
    fn with_writer(
        notices: mpsc::Sender<String>,
        mut write: impl FnMut(&Path, &[u8]) -> std::io::Result<()> + Send + 'static,
    ) -> Self {
        let pending = Arc::new((Mutex::new(Pending::default()), Condvar::new()));
        let queue = Arc::clone(&pending);
        let thread = thread::Builder::new()
            .name("powder-save".into())
            .spawn(move || loop {
                let request = {
                    let (lock, ready) = &*queue;
                    let mut state = lock.lock().unwrap();
                    while state.requests.is_empty() && !state.stopping {
                        state = ready.wait(state).unwrap();
                    }
                    match state.requests.pop_front() {
                        Some(request) => request,
                        None => break,
                    }
                }; // Never hold the queue lock across filesystem operations.
                match write(&request.path, &request.bytes) {
                    Ok(()) if request.announce => {
                        let _ = notices.send("SAVED".into());
                    }
                    Ok(()) => {}
                    Err(error) => {
                        eprintln!("save failed: {error}");
                        let _ = notices.send("SAVE FAILED".into());
                    }
                }
            })
            .expect("Could not start save writer");
        Self {
            pending,
            thread: Some(thread),
        }
    }
    pub fn queue(&self, path: PathBuf, bytes: Vec<u8>, announce: bool) {
        let (lock, ready) = &*self.pending;
        let mut state = lock.lock().unwrap();
        // Repeated requests for the same slot retain only the newest pending
        // snapshot. Memory stays bounded to one writing + one waiting scene per
        // destination, even when a disk flush is slow and Save is tapped repeatedly.
        if let Some(request) = state.requests.iter_mut().find(|r| r.path == path) {
            request.bytes = bytes;
            request.announce |= announce;
        } else {
            state.requests.push_back(Request {
                path,
                bytes,
                announce,
            });
        }
        ready.notify_one();
    }
}
impl Drop for SaveWriter {
    fn drop(&mut self) {
        let (lock, ready) = &*self.pending;
        lock.lock().unwrap().stopping = true;
        ready.notify_one();
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                eprintln!("save writer stopped unexpectedly");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_blocked_flush_does_not_block_new_snapshots_and_shutdown_drains_latest() {
        let (notices, receive) = mpsc::channel();
        let (entered, writing) = mpsc::channel();
        let (release, wait) = mpsc::channel();
        let (written, results) = mpsc::channel();
        let mut first = true;
        let writer = SaveWriter::with_writer(notices, move |path, bytes| {
            if first {
                first = false;
                entered.send(()).unwrap();
                wait.recv().unwrap();
            }
            written.send((path.to_owned(), bytes.to_vec())).unwrap();
            Ok(())
        });
        writer.queue("slot".into(), vec![1], true);
        writing
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        writer.queue("slot".into(), vec![2], true);
        writer.queue("other-slot".into(), vec![9], false);
        writer.queue("slot".into(), vec![3], false);
        assert!(receive.try_recv().is_err());
        release.send(()).unwrap();
        drop(writer);
        assert_eq!(
            results.into_iter().collect::<Vec<_>>(),
            vec![
                (PathBuf::from("slot"), vec![1]),
                (PathBuf::from("slot"), vec![3]),
                (PathBuf::from("other-slot"), vec![9]),
            ]
        );
        assert_eq!(
            receive.into_iter().collect::<Vec<_>>(),
            vec!["SAVED", "SAVED"]
        );
    }
    #[test]
    fn failed_background_writes_report_failure() {
        let (notices, receive) = mpsc::channel();
        let writer = SaveWriter::with_writer(notices, |_, _| {
            Err(std::io::Error::other("fixture disk failure"))
        });
        writer.queue("slot".into(), vec![], false);
        drop(writer);
        assert_eq!(receive.recv().unwrap(), "SAVE FAILED");
    }
}
