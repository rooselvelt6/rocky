use chrono::{DateTime, Utc};
/// Apollo v12 - Dios de la Música, Artes y Conocimiento
/// Sistema de auditoría y logging mejorado
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApolloEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: ApolloEventType,
    pub source: String,
    pub target: Option<String>,
    pub data: serde_json::Value,
    pub severity: EventSeverity,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApolloEventType {
    SystemStart,
    SystemStop,
    UserLogin,
    UserLogout,
    PatientCreated,
    PatientUpdated,
    PatientDeleted,
    AssessmentCreated,
    AssessmentUpdated,
    SecurityEvent,
    ClinicalEvent,
    SystemError,
    PerformanceMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ApolloV12 {
    events: Vec<ApolloEvent>,
    max_events: usize,
    auto_flush_interval: u64,
}

impl ApolloV12 {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 10000,
            auto_flush_interval: 100, // eventos
        }
    }

    pub fn log_event(
        &mut self,
        event_type: ApolloEventType,
        source: &str,
        data: serde_json::Value,
    ) {
        let event = ApolloEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            source: source.to_string(),
            target: None,
            data,
            severity: EventSeverity::Info,
            metadata: HashMap::new(),
        };

        self.events.push(event);

        // Auto-flush si es necesario
        if self.events.len() >= self.auto_flush_interval as usize {
            self.flush_events();
        }

        tracing::info!(
            "☀️ Apollo: Evento registrado - {:?} desde {}",
            event_type,
            source
        );
    }

    pub fn log_error(&mut self, error: &str, source: &str, context: Option<serde_json::Value>) {
        let mut data = serde_json::json!({
            "error": error,
        });

        if let Some(ctx) = context {
            data = serde_json::json!({
                "error": error,
                "context": ctx,
            });
        }

        let event = ApolloEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ApolloEventType::SystemError,
            source: source.to_string(),
            target: None,
            data,
            severity: EventSeverity::Error,
            metadata: HashMap::new(),
        };

        self.events.push(event);

        tracing::error!("☀️ Apollo: ERROR registrado - {} desde {}", error, source);
    }

    pub fn log_security_event(
        &mut self,
        event: &str,
        source: &str,
        user_id: Option<String>,
        severity: EventSeverity,
    ) {
        let data = serde_json::json!({
            "security_event": event,
            "user_id": user_id,
        });

        let mut metadata = HashMap::new();
        if let Some(uid) = user_id {
            metadata.insert("user_id".to_string(), uid);
        }

        let apollo_event = ApolloEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ApolloEventType::SecurityEvent,
            source: source.to_string(),
            target: None,
            data,
            severity,
            metadata,
        };

        self.events.push(apollo_event);

        tracing::warn!(
            "🔒 Apollo: Evento de seguridad - {} desde {}",
            event,
            source
        );
    }

    pub fn log_clinical_event(
        &mut self,
        patient_id: &str,
        event_type: &str,
        data: serde_json::Value,
    ) {
        let event_data = serde_json::json!({
            "patient_id": patient_id,
            "clinical_event": event_type,
            "details": data,
        });

        let event = ApolloEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ApolloEventType::ClinicalEvent,
            source: "clinical_system".to_string(),
            target: Some(patient_id.to_string()),
            data: event_data,
            severity: EventSeverity::Info,
            metadata: HashMap::new(),
        };

        self.events.push(event);

        tracing::info!(
            "☀️ Apollo: Evento clínico - {} para paciente {}",
            event_type,
            patient_id
        );
    }

    pub fn log_performance_metric(&mut self, metric_name: &str, value: f64, unit: &str) {
        let data = serde_json::json!({
            "metric_name": metric_name,
            "value": value,
            "unit": unit,
        });

        let event = ApolloEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ApolloEventType::PerformanceMetric,
            source: "performance_monitor".to_string(),
            target: None,
            data,
            severity: EventSeverity::Info,
            metadata: HashMap::new(),
        };

        self.events.push(event);
    }

    pub fn get_events(
        &self,
        limit: Option<usize>,
        event_type: Option<ApolloEventType>,
    ) -> Vec<ApolloEvent> {
        let mut filtered_events = self.events.clone();

        // Filtrar por tipo si se especifica
        if let Some(event_type) = event_type {
            filtered_events.retain(|e| e.event_type == event_type);
        }

        // Ordenar por timestamp (más reciente primero)
        filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Aplicar límite
        if let Some(limit) = limit {
            filtered_events.truncate(limit);
        }

        filtered_events
    }

    pub fn get_events_by_source(&self, source: &str, limit: Option<usize>) -> Vec<ApolloEvent> {
        let mut source_events = self.events.clone();
        source_events.retain(|e| e.source == source);
        source_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            source_events.truncate(limit);
        }

        source_events
    }

    pub fn get_events_by_severity(
        &self,
        severity: EventSeverity,
        limit: Option<usize>,
    ) -> Vec<ApolloEvent> {
        let mut severity_events = self.events.clone();
        severity_events.retain(|e| e.severity == severity);
        severity_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            severity_events.truncate(limit);
        }

        severity_events
    }

    pub fn get_error_events(&self, limit: Option<usize>) -> Vec<ApolloEvent> {
        self.get_events_by_severity(EventSeverity::Error, limit)
    }

    pub fn get_critical_events(&self, limit: Option<usize>) -> Vec<ApolloEvent> {
        self.get_events_by_severity(EventSeverity::Critical, limit)
    }

    pub fn search_events(&self, query: &str, limit: Option<usize>) -> Vec<ApolloEvent> {
        let mut matching_events = Vec::new();

        for event in &self.events {
            // Búsqueda simple en data serializada
            let event_json = serde_json::to_string(event).unwrap_or_default();
            if event_json.contains(query) {
                matching_events.push(event.clone());
            }
        }

        matching_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            matching_events.truncate(limit);
        }

        matching_events
    }

    pub fn flush_events(&mut self) {
        let events_count = self.events.len();

        // En v13, esto persistiría en base de datos
        // Por ahora, solo logging
        tracing::info!(
            "☀️ Apollo: Flushing {} eventos a base de datos",
            events_count
        );

        // Mantener últimos max_events
        if events_count > self.max_events {
            let keep_count = self.max_events - 1000; // buffer
            self.events.truncate(keep_count);
        }

        // Simular limpieza de eventos viejos
        if events_count > self.max_events / 2 {
            let remove_count = events_count - (self.max_events / 2);
            self.events.drain(0..remove_count);
        }
    }

    pub fn get_statistics(&self) -> ApolloStatistics {
        let total_events = self.events.len();
        let mut event_type_counts = HashMap::new();
        let mut severity_counts = HashMap::new();

        for event in &self.events {
            *event_type_counts
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;
            *severity_counts
                .entry(format!("{:?}", event.severity))
                .or_insert(0) += 1;
        }

        ApolloStatistics {
            total_events,
            event_type_counts,
            severity_counts,
            oldest_event: self
                .events
                .iter()
                .min_by_key(|e| e.timestamp)
                .map(|e| e.timestamp),
            newest_event: self
                .events
                .iter()
                .max_by_key(|e| e.timestamp)
                .map(|e| e.timestamp),
        }
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
        tracing::info!("☀️ Apollo: Todos los eventos limpiados");
    }
}

#[derive(Debug, Serialize)]
pub struct ApolloStatistics {
    pub total_events: usize,
    pub event_type_counts: HashMap<String, u32>,
    pub severity_counts: HashMap<String, u32>,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
}

impl Default for ApolloV12 {
    fn default() -> Self {
        Self::new()
    }
}
