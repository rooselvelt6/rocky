// src/actors/dionysus/engine.rs
// OLYMPUS v15 - Dionysus Analytical Engine

use crate::actors::GodName;
use crate::traits::message::EventPayload;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

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
                *self
                    .event_counts
                    .entry("total_errors".to_string())
                    .or_insert(0) += 1;
                self.record_error(actor, error);
            }
            EventPayload::ClinicalAlert {
                patient_id,
                severity,
            } => {
                *self
                    .event_counts
                    .entry(format!("clinical_alert_{}", severity))
                    .or_insert(0) += 1;
                *self.patient_alerts.entry(patient_id.clone()).or_insert(0) += 1;
            }
            EventPayload::SecurityAlert { threat_type, .. } => {
                *self
                    .event_counts
                    .entry(format!("security_alert_{}", threat_type))
                    .or_insert(0) += 1;
            }
            EventPayload::DataReceived { data_type, .. } => {
                *self
                    .event_counts
                    .entry(format!("data_received_{}", data_type))
                    .or_insert(0) += 1;
            }
            _ => {
                *self
                    .event_counts
                    .entry("other_events".to_string())
                    .or_insert(0) += 1;
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

    pub fn get_top_alerts(&self) -> HashMap<String, u64> {
        self.get_top_alerts_with_limit(5)
    }

    /// Obtiene los top N pacientes por número de alertas
    pub fn get_top_alerts_with_limit(&self, limit: usize) -> HashMap<String, u64> {
        let mut alerts: Vec<_> = self.patient_alerts.iter().collect();
        alerts.sort_by(|a, b| b.1.cmp(a.1));
        alerts
            .into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Obtiene desglose de errores por tipo y actor
    pub fn get_error_breakdown(&self) -> serde_json::Value {
        let mut by_actor: HashMap<String, u64> = HashMap::new();
        let mut by_type: HashMap<String, u64> = HashMap::new();

        for error_entry in &self.error_log {
            // Parsear formato "[ActorName] error message"
            if let Some(start) = error_entry.find('[') {
                if let Some(end) = error_entry.find(']') {
                    let actor_name = &error_entry[start + 1..end];
                    *by_actor.entry(actor_name.to_string()).or_insert(0) += 1;
                }
            }

            // Clasificar por tipo de error (heurística simple)
            let error_lower = error_entry.to_lowercase();
            let error_type = if error_lower.contains("timeout") || error_lower.contains("timed out")
            {
                "timeout"
            } else if error_lower.contains("connection") || error_lower.contains("connect") {
                "connection"
            } else if error_lower.contains("auth") || error_lower.contains("unauthorized") {
                "authentication"
            } else if error_lower.contains("valid") || error_lower.contains("invalid") {
                "validation"
            } else if error_lower.contains("not found") || error_lower.contains("missing") {
                "not_found"
            } else {
                "other"
            };

            *by_type.entry(error_type.to_string()).or_insert(0) += 1;
        }

        serde_json::json!({
            "by_actor": by_actor,
            "by_type": by_type,
            "total_unique_errors": self.error_log.len(),
        })
    }

    /// Obtiene estadísticas de actividad por actor
    pub fn get_actor_statistics(&self) -> serde_json::Value {
        let total_activity: u64 = self.actor_activity.values().sum();
        let actor_count = self.actor_activity.len() as u64;

        let avg_activity = if actor_count > 0 {
            total_activity as f64 / actor_count as f64
        } else {
            0.0
        };

        // Encontrar actor más y menos activo
        let most_active = self
            .actor_activity
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(actor, count)| (actor.clone(), *count));

        let least_active = self
            .actor_activity
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(actor, count)| (actor.clone(), *count));

        serde_json::json!({
            "total_actors": actor_count,
            "total_activity": total_activity,
            "average_activity_per_actor": avg_activity,
            "most_active": most_active.map(|(a, c)| serde_json::json!({"actor": a, "count": c})),
            "least_active": least_active.map(|(a, c)| serde_json::json!({"actor": a, "count": c})),
        })
    }

    /// Calcula estadísticas sobre eventos clínicos
    pub fn get_clinical_statistics(&self) -> serde_json::Value {
        let clinical_alerts: HashMap<String, u64> = self
            .event_counts
            .iter()
            .filter(|(k, _)| k.starts_with("clinical_alert_"))
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        let total_clinical = clinical_alerts.values().sum::<u64>();

        // Agrupar por severidad
        let mut by_severity: HashMap<String, u64> = HashMap::new();
        for (key, count) in &clinical_alerts {
            let severity = key.replace("clinical_alert_", "");
            by_severity.insert(severity, *count);
        }

        serde_json::json!({
            "total_clinical_alerts": total_clinical,
            "by_severity": by_severity,
            "patients_with_alerts": self.patient_alerts.len(),
        })
    }
}
