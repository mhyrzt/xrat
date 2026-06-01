use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct CancellationFlag {
    flag: Arc<AtomicBool>,
}

impl CancellationFlag {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for CancellationFlag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CancellationFlag")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct CancellationReceiver {
    flag: Arc<AtomicBool>,
}

impl CancellationReceiver {
    pub fn from_flag(flag: &CancellationFlag) -> Self {
        Self {
            flag: flag.flag.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for CancellationReceiver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CancellationReceiver")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationFlag, CancellationReceiver};

    #[test]
    fn flag_starts_uncancelled() {
        let flag = CancellationFlag::new();
        assert!(!flag.is_cancelled());
    }

    #[test]
    fn cancel_marks_flag_and_receiver() {
        let flag = CancellationFlag::new();
        let receiver = CancellationReceiver::from_flag(&flag);

        flag.cancel();

        assert!(flag.is_cancelled());
        assert!(receiver.is_cancelled());
    }

    #[test]
    fn receiver_outlives_flag() {
        let flag = CancellationFlag::new();
        let receiver = CancellationReceiver::from_flag(&flag);
        drop(flag);

        assert!(!receiver.is_cancelled());
    }

    #[test]
    fn clones_share_state() {
        let flag = CancellationFlag::new();
        let cloned = flag.clone();

        flag.cancel();

        assert!(cloned.is_cancelled());
    }
}
