use eule::utils::serializable_instant::SerializableInstant;
use tokio::time::{sleep, Duration, Instant};

#[tokio::test]
async fn test_serializable_instant_now() {
    let instant = SerializableInstant::now();
    assert!(instant.to_system_time() > std::time::UNIX_EPOCH);
}

#[tokio::test]
async fn test_serializable_instant_elapsed() {
    let instant = SerializableInstant::now();
    sleep(Duration::from_millis(1000)).await;
    let elapsed = instant.elapsed();
    assert!(elapsed >= Duration::from_millis(1000));
}

#[tokio::test]
async fn test_serializable_instant_conversion() {
    let now = Instant::now();
    let serializable = SerializableInstant::from_instant(now);
    let back_to_instant = serializable.to_instant();

    // Due to conversion process, expect some small difference
    // This difference should be less than 1 millisecond
    let difference = if back_to_instant >= now {
        back_to_instant - now
    } else {
        now - back_to_instant
    };
    assert!(difference < Duration::from_millis(1));
}

#[tokio::test]
async fn test_serializable_instant_duration_since() {
    let instant1 = SerializableInstant::now();
    sleep(Duration::from_millis(1000)).await;
    let instant2 = SerializableInstant::now();

    let duration = instant2.duration_since(instant1);
    assert!(duration >= Duration::from_millis(1000));
}
