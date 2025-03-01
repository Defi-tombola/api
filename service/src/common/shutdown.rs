use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info};

use super::atomic::{await_signal, SignalFlag};

/// The flag used to shutdown the application
pub type ShutdownFlag = SignalFlag;

/// Listen for Ctrl+C and set the shutdown flag
///
/// When user presses Ctrl+C, the shutdown flag will be set to true,
/// which will stop all running tasks.
///
/// For usage, see `await_shutdown_signal`.
pub fn spawn_ctrl_c_listener() -> ShutdownFlag {
    let shutdown = Arc::new(AtomicBool::new(false));

    tokio::task::spawn({
        let shutdown = shutdown.clone();

        async move {
            info!("Press Ctrl+C to exit");
            match tokio::signal::ctrl_c().await {
                Ok(_) => {
                    info!("Ctrl-C received, shutting down");
                    shutdown.store(true, Ordering::Release);
                }
                Err(e) => {
                    error!("Failed to listen for Ctrl+C: {e}");
                }
            }
        }
    });

    shutdown
}

/// Wait for shutdown signal
///
/// This function will wait for shutdown signal to be set to true. After that,
/// dependent tasks should be cleaned up and terminated.
///
/// # Example
///
/// ```rust
/// let shutdown = shutdown_on_ctrl_c();
///
/// tokio::select! {
///    _ = await_shutdown_signal(shutdown.clone()) => {
///       // Cleanup and exit
///       break;
///    }
///    _ = some_other_task() => ()
/// }
/// ```
pub async fn await_shutdown_signal(shutdown: ShutdownFlag) {
    await_signal(shutdown, 2).await
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_shutdown() {
        let shutdown = spawn_ctrl_c_listener();

        // Simulate ctrl+c after 100 milliseconds
        tokio::spawn({
            let shutdown = shutdown.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                shutdown.store(true, Ordering::Release);
            }
        });

        // Wait for shutdown OR panic after 3 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.tick().await; // Skip first tick
        tokio::select! {
            _ = await_shutdown_signal(shutdown) => (),
            _ = interval.tick() => {
                panic!("Shutdown signal was not received");
            }
        };
    }
}
