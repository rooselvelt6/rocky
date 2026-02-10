// src/actors/dionysus/engine.rs
// OLYMPUS v15 - Dionysus Analytical Engine

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::traits::message::EventPayload;
use crate::actors::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEngine {
    pub event_counts: HashMap<String, u64>,
    pub actor_activity: HashMap<GodName, u64>,
    pub patient_alerts: HashMap<String, u64>,
    pub error_log: VecDeque<String>,
    pub health_index: f64,
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self {
            event_counts: HashMap::new(),
            actor_activity: HashMap::new(),
            patient_alerts: HashMap::new(),
            error_log: VecDeque::with_capacity(50),
            health_index: 100.0,
        }
    }
}

impl AnalyticsEngine {
    pub fn process_event(&mut self, source: GodName, event: &EventPayload) {
        // Increment activity for the source actor
        *self.actor_activity.entry(source).or_insert(0) += 1;

        // Process specific event types
        match event {
            EventPayload::ActorStarted { actor } => {
                let key = format!("actor_started_{:?}", actor);
                *self.event_counts.entry(key).or_insert(0) += 1;
            }
            EventPayload::ActorStopped { actor, .. } => {
                let key = format!("actor_stopped_{:?}", actor);
                *self.event_counts.entry(key).or_insert(0) += 1;
            }
            EventPayload::ErrorOccurred { error, actor } => {
                *self.event_counts.entry("total_errors".to_string()).or_insert(0) += 1;
                self.record_error(actor, error);
            }
            EventPayload::ClinicalAlert { patient_id, severity } => {
                *self.event_counts.entry(format!("clinical_alert_{}", severity)).or_insert(0) += 1;
                *self.patient_alerts.entry(patient_id.clone()).or_insert(0) += 1;
            }
            EventPayload::SecurityAlert { threat_type, .. } => {
                *self.event_counts.entry(format!("security_alert_{}", threat_type)).or_insert(0) += 1;
            }
            EventPayload::DataReceived { data_type, .. } => {
                *self.event_counts.entry(format!("data_received_{}", data_type)).or_insert(0) += 1;
            }
            _ => {
                *self.event_counts.entry("other_events".to_string()).or_insert(0) += 1;
            }
        }

        self.recalculate_health();
    }

    fn record_error(&mut self, actor: &GodName, error: &str) {
        if self.error_log.len() >= 50 {
            self.error_log.pop_front();
        }
        self.error_log.push_back(format!("[{:?}] {}", actor, error));
    }

    fn recalculate_health(&mut self) {
        let error_count = *self.event_counts.get("total_errors").unwrap_or(&0) as f64;
        let total_activity: u64 = self.actor_activity.values().sum();
        
        if total_activity > 0 {
            let error_rate = error_count / total_activity as f64;
            self.health_index = (1.0 - error_rate) * 100.0;
        } else {
            self.health_index = 100.0;
        }
    }

    pub fn get_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "health_index": self.health_index,
            "total_events": self.actor_activity.values().sum::<u64>(),
            "active_actors": self.actor_activity.len(),
            "top_patient_alerts": self.get_top_alerts(),
            "recent_errors": self.error_log,
        })
    }

    fn get_top_alerts(&self) -> HashMap<String, u64> {
        let mut alerts: Vec<_> = self.patient_alerts.iter().collect();
        alerts.sort_by(|a, b| b.1.cmp(a.1));
        alerts.into_iter().take(5).map(|(k, v)| (k.clone(), *v)).collect()
    }
}
