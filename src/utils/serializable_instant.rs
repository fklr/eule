//! Utility struct for serializable time instants.
//!
//! The `SerializableInstant` struct allows for easy serialization and deserialization
//! of time instants, which is useful for storing timestamps in persistent storage
//! or sending them over the network.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::Instant;

/// A serializable representation of a point in time.
///
/// # Examples
///
/// Creating a new `SerializableInstant` and calculating elapsed time:
///
/// ```
/// # use std::time::Duration;
/// # use std::thread::sleep;
/// # use eule::utils::serializable_instant::SerializableInstant;
/// #
/// # fn main() {
/// let instant = SerializableInstant::now();
/// sleep(Duration::from_secs(1));
/// let elapsed = instant.elapsed();
/// assert!(elapsed >= Duration::from_secs(1));
/// # }
/// ```
///
/// Converting between `SerializableInstant` and `SystemTime`:
///
/// ```
/// # use std::time::SystemTime;
/// # use eule::utils::serializable_instant::SerializableInstant;
/// #
/// # fn main() {
/// let system_time = SystemTime::now();
/// let serializable = SerializableInstant::from(system_time);
/// let back_to_system_time: SystemTime = serializable.into();
/// # }
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SerializableInstant {
    secs: u64,
    nanos: u32,
}

impl SerializableInstant {
    /// Creates a new SerializableInstant representing the current time.
    ///
    /// # Returns
    ///
    /// A new SerializableInstant instance representing the current system time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// #
    /// # fn main() {
    /// let now = SerializableInstant::now();
    /// # }
    /// ```
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        Self {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }

    /// Calculates the duration elapsed since this instant.
    ///
    /// # Returns
    ///
    /// A Duration representing the time elapsed since this instant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// #
    /// # fn main() {
    /// let instant = SerializableInstant::now();
    /// sleep(Duration::from_secs(1));
    /// let elapsed = instant.elapsed();
    /// assert!(elapsed >= Duration::from_secs(1));
    /// # }
    /// ```
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self)
    }

    /// Calculates the duration elapsed since another instant.
    ///
    /// # Arguments
    ///
    /// * `earlier` - The earlier SerializableInstant to calculate the duration from.
    ///
    /// # Returns
    ///
    /// A Duration representing the time elapsed between `earlier` and this instant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// #
    /// # fn main() {
    /// let instant1 = SerializableInstant::now();
    /// sleep(Duration::from_secs(1));
    /// let instant2 = SerializableInstant::now();
    /// let duration = instant2.duration_since(instant1);
    /// assert!(duration >= Duration::from_secs(1));
    /// # }
    /// ```
    pub fn duration_since(&self, earlier: SerializableInstant) -> Duration {
        if self.secs < earlier.secs || (self.secs == earlier.secs && self.nanos <= earlier.nanos) {
            Duration::new(0, 0)
        } else {
            Duration::new(
                self.secs - earlier.secs,
                self.nanos.saturating_sub(earlier.nanos),
            )
        }
    }

    /// Creates a new SerializableInstant from a SystemTime.
    ///
    /// # Arguments
    ///
    /// * `time` - The SystemTime to convert.
    ///
    /// # Returns
    ///
    /// A new SerializableInstant instance representing the given SystemTime.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::SystemTime;
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// #
    /// # fn main() {
    /// let system_time = SystemTime::now();
    /// let serializable = SerializableInstant::from_system_time(system_time);
    /// # }
    /// ```
    pub fn from_system_time(time: SystemTime) -> Self {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        Self {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }

    /// Converts this SerializableInstant to a SystemTime.
    ///
    /// # Returns
    ///
    /// A SystemTime equivalent to this SerializableInstant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// #
    /// # fn main() {
    /// let serializable = SerializableInstant::now();
    /// let system_time = serializable.to_system_time();
    /// # }
    /// ```
    pub fn to_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::new(self.secs, self.nanos)
    }

    /// Creates a new SerializableInstant from an Instant.
    ///
    /// # Arguments
    ///
    /// * `instant` - The Instant to convert.
    ///
    /// # Returns
    ///
    /// A new SerializableInstant instance representing the given Instant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::time::Instant;
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// #
    /// # fn main() {
    /// let instant = Instant::now();
    /// let serializable = SerializableInstant::from_instant(instant);
    /// # }
    /// ```
    pub fn from_instant(instant: Instant) -> Self {
        // Convert Instant to SystemTime and then to SerializableInstant
        let system_time = SystemTime::now() - instant.elapsed();
        Self::from_system_time(system_time)
    }

    /// Converts this SerializableInstant to an Instant.
    ///
    /// # Returns
    ///
    /// An Instant equivalent to this SerializableInstant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::utils::serializable_instant::SerializableInstant;
    /// #
    /// # fn main() {
    /// let serializable = SerializableInstant::now();
    /// let instant = serializable.to_instant();
    /// # }
    /// ```
    pub fn to_instant(&self) -> Instant {
        // Convert SerializableInstant to SystemTime, then to Instant
        let system_time = self.to_system_time();
        let duration_since_now = SystemTime::now()
            .duration_since(system_time)
            .unwrap_or(Duration::new(0, 0));
        Instant::now() - duration_since_now
    }
}

impl From<SystemTime> for SerializableInstant {
    fn from(time: SystemTime) -> Self {
        Self::from_system_time(time)
    }
}

impl From<SerializableInstant> for SystemTime {
    fn from(serializable: SerializableInstant) -> Self {
        serializable.to_system_time()
    }
}
