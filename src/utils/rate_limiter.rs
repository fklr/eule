use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    rate: u32,
    per: Duration,
    allowance: Mutex<f64>,
    last_check: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(rate: u32, per: Duration) -> Self {
        RateLimiter {
            rate,
            per,
            allowance: Mutex::new(rate as f64),
            last_check: Mutex::new(Instant::now()),
        }
    }

    pub async fn check(&self) -> Result<(), &'static str> {
        let mut allowance = self.allowance.lock().await;
        let mut last_check = self.last_check.lock().await;

        let current = Instant::now();
        let time_passed = current.duration_since(*last_check).as_secs_f64();
        *last_check = current;

        *allowance += time_passed * (self.rate as f64 / self.per.as_secs_f64());
        if *allowance > self.rate as f64 {
            *allowance = self.rate as f64;
        }

        if *allowance < 1.0 {
            Err("Rate limit exceeded")
        } else {
            *allowance -= 1.0;
            Ok(())
        }
    }
}
