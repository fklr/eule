use crate::utils::serializable_instant::SerializableInstant;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

/// Represents a single cleanup task for a channel.
#[derive(Serialize, Deserialize, Clone)]
pub struct CleanupTask {
    /// The interval between cleanups.
    pub interval: Duration,
    /// The time of the last cleanup.
    pub last_cleanup: SerializableInstant,
}

impl CleanupTask {
    /// Creates a new CleanupTask with the given interval.
    ///
    /// # Parameters
    /// - `interval`: The time interval between cleanups.
    ///
    /// # Returns
    /// A new CleanupTask instance.
    pub async fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_cleanup: SerializableInstant::now(),
        }
    }

    /// Checks if it's time to perform a cleanup based on the interval and last cleanup time.
    ///
    /// # Returns
    /// `true` if it's time to perform a cleanup, `false` otherwise.
    pub async fn is_due(&self) -> bool {
        self.last_cleanup.elapsed() >= self.interval
    }
}
