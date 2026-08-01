//! Per-source token-bucket rate limiting for the evaluation endpoint
//! (API-005: 60 requests/minute sustained with a burst of 20; API-006:
//! a limited response is 429 with Retry-After and no clinical result).
//! The edge WAF enforces the same policy; the application enforces it
//! independently so the control does not rely on deployment topology.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::Instant;

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

pub struct RateLimiter {
    /// Tokens added per second (sustained rate / 60).
    refill_per_second: f64,
    /// Maximum tokens (burst capacity).
    capacity: f64,
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
}

impl RateLimiter {
    pub fn new(per_minute: u32, burst: u32) -> Self {
        Self {
            refill_per_second: f64::from(per_minute) / 60.0,
            capacity: f64::from(burst),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `Ok(())` when the request is admitted, or `Err(retry_after
    /// _seconds)` when the source has exhausted its budget.
    pub fn check(&self, source: IpAddr) -> Result<(), u32> {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter lock");
        let bucket = buckets.entry(source).or_insert(Bucket {
            tokens: self.capacity,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_second).min(self.capacity);
        bucket.last_refill = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            let deficit = 1.0 - bucket.tokens;
            let wait = (deficit / self.refill_per_second).ceil().max(1.0);
            Err(wait as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_then_limited_with_retry_after() {
        let limiter = RateLimiter::new(60, 3);
        let ip: IpAddr = "192.0.2.1".parse().unwrap();
        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_ok());
        assert!(limiter.check(ip).is_ok());
        let retry = limiter
            .check(ip)
            .expect_err("fourth request must be limited");
        assert!(retry >= 1);
    }

    #[test]
    fn sources_are_independent() {
        let limiter = RateLimiter::new(60, 1);
        let a: IpAddr = "192.0.2.1".parse().unwrap();
        let b: IpAddr = "192.0.2.2".parse().unwrap();
        assert!(limiter.check(a).is_ok());
        assert!(limiter.check(b).is_ok());
        assert!(limiter.check(a).is_err());
        assert!(limiter.check(b).is_err());
    }
}
