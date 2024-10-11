use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    rate: usize,
    per: Duration,
    allowance: Mutex<f64>,
    last_check: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(rate: usize, per: u64) -> Self {
        Self {
            rate,
            per: Duration::from_secs(per),
            allowance: Mutex::new(rate as f64),
            last_check: Mutex::new(Instant::now()),
        }
    }

    pub async fn check(&self) -> Result<(), ()> {
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
            Err(())
        } else {
            *allowance -= 1.0;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 1); // 2 requests per second

        let result = block_on(async {
            let r1 = limiter.check().await;
            let r2 = limiter.check().await;
            let r3 = limiter.check().await;
            (r1, r2, r3)
        });

        assert!(result.0.is_ok());
        assert!(result.1.is_ok());
        assert!(result.2.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_recovery() {
        let limiter = RateLimiter::new(2, 1);

        assert!(limiter.check().await.is_ok());
        assert!(limiter.check().await.is_ok());
        assert!(limiter.check().await.is_err());

        tokio::time::sleep(Duration::from_secs(1)).await;

        assert!(limiter.check().await.is_ok());
    }
}
