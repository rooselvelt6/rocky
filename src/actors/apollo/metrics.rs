use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::actors::GodName;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetrics {
    pub total_events: u64,
    pub events_per_type: HashMap<String, u64>,
    pub events_per_actor: HashMap<GodName, u64>,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub last_event_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl EventMetrics {
    pub fn record_event(&mut self, actor: GodName, event_type: &str) {
        self.total_events += 1;
        *self.events_per_type.entry(event_type.to_string()).or_insert(0) += 1;
        *self.events_per_actor.entry(actor).or_insert(0) += 1;
        self.last_event_time = Some(chrono::Utc::now());
    }

    pub fn record_error(&mut self) {
        self.total_errors += 1;
    }
}
