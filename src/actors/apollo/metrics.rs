// src/actors/apollo/metrics.rs
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetrics {
    pub total_events: u64,
    pub events_per_type: std::collections::HashMap<String, u64>,
}
