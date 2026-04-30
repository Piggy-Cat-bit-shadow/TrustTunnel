use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) const EVICTION_TIMEOUT: Duration = Duration::from_secs(300);
pub(crate) const EVICTION_INTERVAL: Duration = Duration::from_secs(60);

pub(crate) struct AuthRateLimiterConfig {
    pub max_failures: u32,
    pub refill_interval: Duration,
    pub eviction_timeout: Duration,
}

struct IpEntry {
    tokens: f64,
    last_check: Instant,
}

pub(crate) struct AuthRateLimiter {
    config: AuthRateLimiterConfig,
    entries: Mutex<HashMap<IpAddr, IpEntry>>,
}

impl AuthRateLimiter {
    pub fn new(config: AuthRateLimiterConfig) -> Self {
        Self {
            config,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `true` if the failure is within the allowed budget,
    /// `false` if the IP should be blocked.
    pub fn record_failure(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.entry(ip).or_insert_with(|| IpEntry {
            tokens: self.config.max_failures as f64,
            last_check: now,
        });

        let elapsed = now.duration_since(entry.last_check);
        let refill = elapsed.as_secs_f64() / self.config.refill_interval.as_secs_f64();
        entry.tokens = (entry.tokens + refill).min(self.config.max_failures as f64);
        entry.last_check = now;
        entry.tokens -= 1.0;

        entry.tokens >= 0.0
    }

    /// Returns `true` if the IP is currently blocked. Does not modify state.
    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get(&ip) {
            let elapsed = now.duration_since(entry.last_check);
            let refill = elapsed.as_secs_f64() / self.config.refill_interval.as_secs_f64();
            let current_tokens = (entry.tokens + refill).min(self.config.max_failures as f64);
            current_tokens < 0.0
        } else {
            false
        }
    }

    pub fn evict_expired(&self) {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap();
        entries
            .retain(|_, entry| now.duration_since(entry.last_check) < self.config.eviction_timeout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ip(last_octet: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, last_octet))
    }

    fn limiter_with_capacity(max_failures: u32, refill_ms: u64) -> AuthRateLimiter {
        AuthRateLimiter::new(AuthRateLimiterConfig {
            max_failures,
            refill_interval: Duration::from_millis(refill_ms),
            eviction_timeout: Duration::from_secs(300),
        })
    }

    #[test]
    fn failures_within_budget_are_accepted() {
        let limiter = limiter_with_capacity(3, 10_000);
        let addr = ip(1);

        assert!(limiter.record_failure(addr));
        assert!(limiter.record_failure(addr));
        assert!(limiter.record_failure(addr));
    }

    #[test]
    fn excess_failures_are_rejected() {
        let limiter = limiter_with_capacity(3, 10_000);
        let addr = ip(2);

        limiter.record_failure(addr);
        limiter.record_failure(addr);
        limiter.record_failure(addr);
        assert!(!limiter.record_failure(addr));
    }

    #[test]
    fn is_blocked_returns_true_after_budget_exhausted() {
        let limiter = limiter_with_capacity(2, 10_000);
        let addr = ip(3);

        limiter.record_failure(addr);
        limiter.record_failure(addr);
        limiter.record_failure(addr);

        assert!(limiter.is_blocked(addr));
    }

    #[test]
    fn is_blocked_returns_false_for_unknown_ip() {
        let limiter = limiter_with_capacity(5, 10_000);
        assert!(!limiter.is_blocked(ip(42)));
    }

    #[test]
    fn is_blocked_does_not_consume_tokens() {
        let limiter = limiter_with_capacity(1, 10_000);
        let addr = ip(4);

        limiter.record_failure(addr);

        // At this point tokens = 0, not yet blocked
        assert!(!limiter.is_blocked(addr));
        assert!(!limiter.is_blocked(addr));
        assert!(!limiter.is_blocked(addr));

        // One more failure exhausts the budget
        assert!(!limiter.record_failure(addr));
        assert!(limiter.is_blocked(addr));
    }

    #[test]
    fn tokens_refill_over_time() {
        let limiter = limiter_with_capacity(2, 50);
        let addr = ip(5);

        limiter.record_failure(addr);
        limiter.record_failure(addr);
        limiter.record_failure(addr);
        assert!(limiter.is_blocked(addr));

        // Wait for at least one refill interval so the IP is unblocked.
        std::thread::sleep(Duration::from_millis(120));
        assert!(!limiter.is_blocked(addr));
    }

    #[test]
    fn recovery_allows_further_failures_after_refill() {
        let limiter = limiter_with_capacity(2, 50);
        let addr = ip(6);

        limiter.record_failure(addr);
        limiter.record_failure(addr);
        limiter.record_failure(addr);

        std::thread::sleep(Duration::from_millis(120));

        // After refill the next failure should be accepted
        assert!(limiter.record_failure(addr));
    }

    #[test]
    fn different_ips_are_independent() {
        let limiter = limiter_with_capacity(1, 10_000);
        let addr_a = ip(10);
        let addr_b = ip(20);

        limiter.record_failure(addr_a);
        limiter.record_failure(addr_a);

        assert!(limiter.is_blocked(addr_a));
        assert!(!limiter.is_blocked(addr_b));
    }

    #[test]
    fn evict_expired_removes_stale_entries() {
        let limiter = AuthRateLimiter::new(AuthRateLimiterConfig {
            max_failures: 5,
            refill_interval: Duration::from_secs(10),
            eviction_timeout: Duration::from_millis(50),
        });
        let addr = ip(7);

        limiter.record_failure(addr);
        assert!(limiter.entries.lock().unwrap().contains_key(&addr));

        std::thread::sleep(Duration::from_millis(100));
        limiter.evict_expired();

        assert!(!limiter.entries.lock().unwrap().contains_key(&addr));
    }

    #[test]
    fn evict_expired_keeps_recent_entries() {
        let limiter = AuthRateLimiter::new(AuthRateLimiterConfig {
            max_failures: 5,
            refill_interval: Duration::from_secs(10),
            eviction_timeout: Duration::from_millis(200),
        });
        let addr = ip(8);

        limiter.record_failure(addr);
        limiter.evict_expired();

        assert!(limiter.entries.lock().unwrap().contains_key(&addr));
    }

    #[test]
    fn concurrent_failures_are_safe() {
        use std::sync::Arc;
        use std::thread;

        let limiter = Arc::new(limiter_with_capacity(100, 10_000));
        let addr = ip(9);
        let mut handles = Vec::new();

        for _ in 0..8 {
            let l = Arc::clone(&limiter);
            handles.push(thread::spawn(move || {
                for _ in 0..10 {
                    l.record_failure(addr);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // 80 failures consumed from budget of 100; tokens should be 20
        assert!(!limiter.is_blocked(addr));
    }
}
