pub use crate::support::cancel::{CancellationFlag, CancellationReceiver};

pub type TuiCancellationToken = CancellationFlag;
pub type TuiCancellationReceiver = CancellationReceiver;

pub fn new_cancellation() -> (TuiCancellationToken, TuiCancellationReceiver) {
    let flag = CancellationFlag::new();
    let receiver = CancellationReceiver::from_flag(&flag);
    (flag, receiver)
}

#[cfg(test)]
mod tests {
    use super::new_cancellation;

    #[test]
    fn token_and_receiver_share_state() {
        let (token, receiver) = new_cancellation();
        token.cancel();

        assert!(receiver.is_cancelled());
    }
}
