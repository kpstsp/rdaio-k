use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct PlaybackControl {
    pub paused: Arc<AtomicBool>,
    pub stopped: Arc<AtomicBool>,
    #[allow(dead_code)]
    pub position_ms: Arc<Mutex<u64>>,
}

impl PlaybackControl {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            stopped: Arc::new(AtomicBool::new(false)),
            position_ms: Arc::new(Mutex::new(0)),
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        println!("[Control] Set paused=true");
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        println!("[Control] Set paused=false");
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        println!("[Control] Set stopped=true");
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub fn set_position(&self, ms: u64) {
        *self.position_ms.lock().unwrap() = ms;
        println!("[Control] Set position={}", ms);
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> u64 {
        *self.position_ms.lock().unwrap()
    }
}

impl Default for PlaybackControl {
    fn default() -> Self {
        Self::new()
    }
}
