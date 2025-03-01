use chrono::DateTime;
use chrono::NaiveDateTime;

use std::time::{Duration, SystemTime};
use tracing::info;

/// Compute the waiting time for the next round cycle
///
/// Example:
///
/// ```
/// // If current time is 2021-01-01 00:00:15
/// let wait = compute_wait_next_round_cycle(10); // Returns 5 seconds
/// ```
fn compute_wait_next_round_cycle(cycle_secs: u64) -> Duration {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    std::time::Duration::from_secs(cycle_secs - (timestamp % cycle_secs))
}

/// Wait until next round cycle
///
/// Example:
///
/// ```
/// // If current time is 2021-01-01 00:00:15
/// wait_next_round_cycle(10).await; // Wait 5 seconds
/// ```
pub async fn wait_next_round_cycle(cycle_secs: u64) {
    let wait = compute_wait_next_round_cycle(cycle_secs);

    info!(
        "Waiting for next round cycle, waiting for {} seconds",
        wait.as_secs()
    );

    tokio::time::sleep(wait).await;
}

/// Round datetime to seconds
///
/// Example:
///
/// ```
///  // If current time is 2021-01-01 00:00:15
/// round_datetime_to_seconds(datetime, 10); // Returns 2021-01-01 00:00:10
/// ```
pub fn round_datetime_to_seconds(datetime: NaiveDateTime, seconds: u64) -> NaiveDateTime {
    let timestamp = datetime.and_utc().timestamp();
    let timestamp = timestamp - (timestamp % seconds as i64);
    DateTime::from_timestamp(timestamp, 0).unwrap().naive_utc()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_next_round_cycle() {
        let cycle = 10; // 10 second cycle
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        wait_next_round_cycle(cycle).await;

        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed_time = end_time - start_time;
        assert!(elapsed_time <= cycle);
    }

    #[tokio::test]
    async fn test_compute_wait_next_round_cycle() {
        let cycle = 600; // 10 minute cycle
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let wait = compute_wait_next_round_cycle(cycle);

        let wait_secs = wait.as_secs();

        let end_time = now % cycle;
        let elapsed_time = end_time + wait_secs;

        assert!(wait_secs <= cycle);
        assert!(elapsed_time <= cycle);
    }

    #[test]
    fn test_round_datetime_to_seconds() {
        let datetime_format = "%Y-%m-%d %H:%M:%S";

        let datetime =
            NaiveDateTime::parse_from_str("2021-01-01 00:00:15", datetime_format).unwrap();
        let rounded = round_datetime_to_seconds(datetime, 10);

        assert_eq!(
            format!("{}", rounded.format(datetime_format)),
            "2021-01-01 00:00:10"
        );
    }
}
