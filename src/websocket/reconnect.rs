use std::time::Duration;
use tracing::{debug, info};

/// Manages WebSocket reconnection logic with exponential backoff
#[derive(Debug, Clone)]
pub struct ReconnectManager {
    max_attempts: u32,
    current_attempt: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter: bool,
}

impl ReconnectManager {
    /// Create new reconnection manager with default settings
    pub fn new() -> Self {
        Self {
            max_attempts: 0, // 0 means infinite attempts
            current_attempt: 0,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes max
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Set maximum number of attempts (0 = infinite)
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set base delay between attempts
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay cap
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Check if we should attempt reconnection
    pub fn should_reconnect(&self) -> bool {
        if self.max_attempts == 0 {
            true // Infinite attempts
        } else {
            self.current_attempt < self.max_attempts
        }
    }

    /// Get the next delay duration with exponential backoff
    pub fn next_delay(&mut self) -> Duration {
        if !self.should_reconnect() {
            return Duration::from_secs(0);
        }

        let attempt = self.current_attempt as f64;
        let delay_secs = self.base_delay.as_secs_f64() * self.backoff_multiplier.powf(attempt);

        // Cap at maximum delay
        let delay_secs = delay_secs.min(self.max_delay.as_secs_f64());

        // Add jitter to prevent thundering herd
        let final_delay = if self.jitter {
            self.add_jitter(delay_secs)
        } else {
            delay_secs
        };

        self.current_attempt += 1;

        let duration = Duration::from_secs_f64(final_delay);

        info!(
            "Reconnection attempt {} of {}, waiting {:?}",
            self.current_attempt,
            if self.max_attempts == 0 {
                "∞".to_string()
            } else {
                self.max_attempts.to_string()
            },
            duration
        );

        duration
    }

    /// Reset counter on successful connection
    pub fn reset(&mut self) {
        if self.current_attempt > 0 {
            info!("Connection successful, resetting reconnection counter");
            self.current_attempt = 0;
        }
    }

    /// Get current attempt number
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }

    /// Get maximum attempts
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Check if we've exhausted all attempts
    pub fn is_exhausted(&self) -> bool {
        self.max_attempts > 0 && self.current_attempt >= self.max_attempts
    }

    /// Add jitter to delay to prevent thundering herd
    fn add_jitter(&self, delay_secs: f64) -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create pseudo-random jitter based on current time
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();

        // Jitter factor between 0.5 and 1.5
        let jitter_factor = 0.5 + (hash as f64 / u64::MAX as f64);

        delay_secs * jitter_factor
    }
}

impl Default for ReconnectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Reconnection strategy enumeration
#[derive(Debug, Clone)]
pub enum ReconnectStrategy {
    /// Fixed delay between attempts
    Fixed(Duration),
    /// Linear backoff with fixed increment
    Linear { base: Duration, increment: Duration },
    /// Exponential backoff with multiplier
    Exponential {
        base: Duration,
        multiplier: f64,
        max: Duration,
    },
    /// Custom strategy with callback
    Custom(fn(u32) -> Duration),
}

impl ReconnectStrategy {
    /// Calculate delay for given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            Self::Fixed(delay) => *delay,
            Self::Linear { base, increment } => *base + (*increment * attempt),
            Self::Exponential {
                base,
                multiplier,
                max,
            } => {
                let delay_secs = base.as_secs_f64() * multiplier.powi(attempt as i32);
                let capped = delay_secs.min(max.as_secs_f64());
                Duration::from_secs_f64(capped)
            }
            Self::Custom(callback) => callback(attempt),
        }
    }
}

/// Advanced reconnection manager with strategy support
#[derive(Debug)]
pub struct AdvancedReconnectManager {
    strategy: ReconnectStrategy,
    max_attempts: u32,
    current_attempt: u32,
    last_attempt_time: Option<std::time::Instant>,
    min_interval: Duration,
}

impl AdvancedReconnectManager {
    /// Create new advanced reconnection manager
    pub fn new(strategy: ReconnectStrategy) -> Self {
        Self {
            strategy,
            max_attempts: 0,
            current_attempt: 0,
            last_attempt_time: None,
            min_interval: Duration::from_millis(100),
        }
    }

    /// Set maximum attempts
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set minimum interval between attempts
    pub fn with_min_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    /// Check if reconnection should be attempted
    pub fn should_reconnect(&self) -> bool {
        if self.max_attempts > 0 && self.current_attempt >= self.max_attempts {
            return false;
        }

        // Check minimum interval
        if let Some(last_time) = self.last_attempt_time {
            if last_time.elapsed() < self.min_interval {
                debug!("Minimum interval not reached, skipping reconnection attempt");
                return false;
            }
        }

        true
    }

    /// Get next delay and increment attempt counter
    pub fn next_delay(&mut self) -> Option<Duration> {
        if !self.should_reconnect() {
            return None;
        }

        let delay = self.strategy.calculate_delay(self.current_attempt);
        self.current_attempt += 1;
        self.last_attempt_time = Some(std::time::Instant::now());

        Some(delay)
    }

    /// Reset on successful connection
    pub fn reset(&mut self) {
        if self.current_attempt > 0 {
            info!(
                "Advanced reconnection manager reset after {} attempts",
                self.current_attempt
            );
            self.current_attempt = 0;
            self.last_attempt_time = None;
        }
    }

    /// Get current attempt statistics
    pub fn stats(&self) -> ReconnectStats {
        ReconnectStats {
            current_attempt: self.current_attempt,
            max_attempts: self.max_attempts,
            last_attempt: self.last_attempt_time.map(|t| t.elapsed()),
            strategy_name: match self.strategy {
                ReconnectStrategy::Fixed(_) => "fixed",
                ReconnectStrategy::Linear { .. } => "linear",
                ReconnectStrategy::Exponential { .. } => "exponential",
                ReconnectStrategy::Custom(_) => "custom",
            },
        }
    }
}

/// Reconnection statistics
#[derive(Debug, Clone)]
pub struct ReconnectStats {
    pub current_attempt: u32,
    pub max_attempts: u32,
    pub last_attempt: Option<Duration>,
    pub strategy_name: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_reconnect_manager() {
        let mut manager = ReconnectManager::new()
            .with_max_attempts(3)
            .with_base_delay(Duration::from_secs(1))
            .with_backoff_multiplier(2.0)
            .with_jitter(false);

        assert!(manager.should_reconnect());

        let delay1 = manager.next_delay();
        assert_eq!(delay1, Duration::from_secs(1));

        let delay2 = manager.next_delay();
        assert_eq!(delay2, Duration::from_secs(2));

        let delay3 = manager.next_delay();
        assert_eq!(delay3, Duration::from_secs(4));

        // Should not reconnect after max attempts
        assert!(!manager.should_reconnect());
        assert!(manager.is_exhausted());
    }

    #[test]
    fn test_infinite_attempts() {
        let mut manager = ReconnectManager::new()
            .with_max_attempts(0) // Infinite
            .with_base_delay(Duration::from_secs(1));

        for _ in 0..100 {
            assert!(manager.should_reconnect());
            manager.next_delay();
        }
    }

    #[test]
    fn test_reset_functionality() {
        let mut manager = ReconnectManager::new()
            .with_max_attempts(3)
            .with_base_delay(Duration::from_secs(1));

        manager.next_delay();
        manager.next_delay();
        assert_eq!(manager.current_attempt(), 2);

        manager.reset();
        assert_eq!(manager.current_attempt(), 0);
        assert!(manager.should_reconnect());
    }

    #[test]
    fn test_reconnect_strategies() {
        let fixed = ReconnectStrategy::Fixed(Duration::from_secs(5));
        assert_eq!(fixed.calculate_delay(0), Duration::from_secs(5));
        assert_eq!(fixed.calculate_delay(10), Duration::from_secs(5));

        let linear = ReconnectStrategy::Linear {
            base: Duration::from_secs(1),
            increment: Duration::from_secs(2),
        };
        assert_eq!(linear.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(linear.calculate_delay(1), Duration::from_secs(3));
        assert_eq!(linear.calculate_delay(2), Duration::from_secs(5));

        let exponential = ReconnectStrategy::Exponential {
            base: Duration::from_secs(1),
            multiplier: 2.0,
            max: Duration::from_secs(10),
        };
        assert_eq!(exponential.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(exponential.calculate_delay(1), Duration::from_secs(2));
        assert_eq!(exponential.calculate_delay(2), Duration::from_secs(4));
        assert_eq!(exponential.calculate_delay(10), Duration::from_secs(10)); // Capped
    }
}
