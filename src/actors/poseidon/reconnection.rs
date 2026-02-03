// src/actors/poseidon/reconnection.rs
// OLYMPUS v13 - Poseidon Reconnection Policy
// Exponential backoff para reconexiones

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionPolicy {
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_attempts: u32,
    pub backoff_multiplier: f64,
    pub jitter: f64,
}

impl Default for ReconnectionPolicy {
    fn default() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            max_attempts: 10,
            backoff_multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl ReconnectionPolicy {
    pub fn next_delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay_ms as f64 * (self.backoff_multiplier.powi(attempt as i32));
        let delay = delay.min(self.max_delay_ms as f64);

        // Add jitter
        let jitter_amount = delay * self.jitter;
        let delay = delay + (rand::random::<f64>() * jitter_amount * 2.0 - jitter_amount);

        Duration::from_millis((delay as u64).max(self.initial_delay_ms))
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    pub fn max_delay(&self) -> Duration {
        Duration::from_millis(self.max_delay_ms)
    }
}
