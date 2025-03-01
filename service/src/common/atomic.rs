use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// The flag used to signal change of state
pub type SignalFlag = Arc<AtomicBool>;

/// Wait for signal
///
/// This function will wait for signal to be set to true.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use std::sync::atomic::AtomicBool;
/// let signal = Arc::new(AtomicBool::new(false));
///
/// tokio::select! {
///   _ = await_signal(signal.clone(), 2) => {
///      // Signal received
///   }
///   _ = some_other_task() => ()
/// }
/// ```
pub async fn await_signal(signal: SignalFlag, secs: u64) {
    while !signal.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_secs(secs)).await;
    }
}
