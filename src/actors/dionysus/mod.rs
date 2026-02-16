// src/actors/dionysus/mod.rs
// OLYMPUS v15 - Dionysus: Dios del Vino y Análisis Avanzado
// Responsabilidad: Analytics, estadísticas, tendencias y detección de anomalías

#![allow(dead_code)]

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, EventPayload};
use crate::errors::ActorError;

// Importar el motor analítico
pub mod engine;
pub use engine::AnalyticsEngine;

/// Dionysus - Dios del Análisis
/// Procesa eventos del sistema y genera métricas, tendencias y reportes
#[derive(Debug)]
pub struct Dionysus {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Motor analítico principal
    analytics: Arc<RwLock<AnalyticsEngine>>,
    /// Métricas en tiempo real
    metrics: Arc<RwLock<RealTimeMetrics>>,
    /// Cache de queries frecuentes
    query_cache: Arc<RwLock<HashMap<String, (DateTime<Utc>, serde_json::Value)>>>,
}

/// Métricas en tiempo real del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub events_processed: u64,
    pub events_per_second: f64,
    pub last_calculation: DateTime<Utc>,
    pub memory_usage_mb: f64,
}

impl Default for RealTimeMetrics {
    fn default() -> Self {
        Self {
            events_processed: 0,
            events_per_second: 0.0,
            last_calculation: Utc::now(),
            memory_usage_mb: 0.0,
        }
    }
}

impl Dionysus {
    pub async fn new() -> Self {
        info!("🍷 Dionysus: Inicializando motor analítico...");
        
        Self {
            name: GodName::Dionysus,
            state: ActorState::new(GodName::Dionysus),
            config: ActorConfig::default(),
            analytics: Arc::new(RwLock::new(AnalyticsEngine::default())),
            metrics: Arc::new(RwLock::new(RealTimeMetrics::default())),
            query_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Procesa un evento recibido de Apollo u otros actores
    async fn process_event(&self, source: GodName, event: &EventPayload) {
        let mut analytics = self.analytics.write().await;
        analytics.process_event(source, event);
        
        // Actualizar métricas en tiempo real
        let mut metrics = self.metrics.write().await;
        metrics.events_processed += 1;
        
        // Calcular eventos por segundo cada 100 eventos
        if metrics.events_processed % 100 == 0 {
            let now = Utc::now();
            let elapsed = (now - metrics.last_calculation).num_seconds() as f64;
            if elapsed > 0.0 {
                metrics.events_per_second = 100.0 / elapsed;
            }
            metrics.last_calculation = now;
        }
        
        debug!("🍷 Dionysus: Evento procesado de {:?}", source);
    }

    /// Invalida la caché de queries
    async fn invalidate_cache(&self) {
        let mut cache = self.query_cache.write().await;
        let now = Utc::now();
        // Eliminar entradas mayores a 5 minutos
        cache.retain(|_, (timestamp, _)| {
            (now - *timestamp) < Duration::minutes(5)
        });
    }

    /// Obtiene métricas del sistema
    async fn get_system_metrics(&self) -> serde_json::Value {
        let analytics = self.analytics.read().await;
        let metrics = self.metrics.read().await;
        
        serde_json::json!({
            "events_processed": metrics.events_processed,
            "events_per_second": metrics.events_per_second,
            "health_index": analytics.health_index,
            "total_actors": analytics.actor_activity.len(),
            "total_errors": analytics.event_counts.get("total_errors").unwrap_or(&0),
        })
    }
}

#[async_trait]
impl OlympianActor for Dionysus {
    fn name(&self) -> GodName { 
        GodName::Dionysus 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::Analysis 
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = Utc::now();

        match msg.payload {
            MessagePayload::Event(event) => {
                let source = msg.from.unwrap_or(GodName::Zeus);
                self.process_event(source, &event).await;
                Ok(ResponsePayload::Ack { message_id: msg.id })
            }
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }

    async fn persistent_state(&self) -> serde_json::Value {
        let analytics = self.analytics.read().await;
        serde_json::json!({
            "name": "Dionysus",
            "messages": self.state.message_count,
            "events_processed": analytics.event_counts.values().sum::<u64>(),
            "status": self.state.status,
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        // TODO: Restaurar estado desde persistencia
        Ok(())
    }

    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name.clone(),
            status: self.state.status.clone(),
            last_seen: Utc::now(),
            load: 0.0, // Se calcularía dinámicamente
            memory_usage_mb: 0.0,
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }

    async fn health_check(&self) -> HealthStatus {
        let analytics = self.analytics.read().await;
        let error_count = *analytics.event_counts.get("total_errors").unwrap_or(&0);
        
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count,
            last_error: analytics.error_log.back().map(|e| e.clone()),
            memory_usage_mb: 0.0,
            timestamp: Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🍷 Dionysus: Motor analítico v15 iniciado");
        info!("🍷 Dionysus: Listo para procesar eventos y generar insights");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🍷 Dionysus: Deteniendo motor analítico...");
        // Guardar estado si es necesario
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados para manejo de comandos y queries
impl Dionysus {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("clear_cache") => {
                        let mut cache = self.query_cache.write().await;
                        cache.clear();
                        Ok(ResponsePayload::Success { 
                            message: "Caché de queries limpiada".to_string() 
                        })
                    }
                    Some("reset_metrics") => {
                        let mut analytics = self.analytics.write().await;
                        *analytics = AnalyticsEngine::default();
                        let mut metrics = self.metrics.write().await;
                        *metrics = RealTimeMetrics::default();
                        Ok(ResponsePayload::Success { 
                            message: "Métricas reiniciadas".to_string() 
                        })
                    }
                    Some("invalidate_cache") => {
                        self.invalidate_cache().await;
                        Ok(ResponsePayload::Success { 
                            message: "Caché invalidada".to_string() 
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Dionysus, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Dionysus, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let data = self.get_system_metrics().await;
                Ok(ResponsePayload::Stats { data })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                // Verificar caché para queries frecuentes
                let cache_key = format!("{}:{}", query_type, data.to_string());
                {
                    let cache = self.query_cache.read().await;
                    if let Some((timestamp, result)) = cache.get(&cache_key) {
                        if (Utc::now() - *timestamp) < Duration::minutes(5) {
                            return Ok(ResponsePayload::Data { data: result.clone() });
                        }
                    }
                }
                
                let result = match query_type {
                    "system_health" => {
                        let analytics = self.analytics.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "health_index": analytics.health_index,
                                "total_events": analytics.actor_activity.values().sum::<u64>(),
                                "active_actors": analytics.actor_activity.len(),
                                "top_patient_alerts": analytics.get_top_alerts(),
                                "recent_errors": analytics.error_log.iter().rev().take(10).collect::<Vec<_>>(),
                            })
                        })
                    }
                    "actor_activity" => {
                        let actor = data.get("actor")
                            .and_then(|v| serde_json::from_value::<GodName>(v.clone()).ok());
                        
                        if let Some(actor_name) = actor {
                            let analytics = self.analytics.read().await;
                            let activity = analytics.actor_activity.get(&actor_name).unwrap_or(&0);
                            Ok(ResponsePayload::Data { 
                                data: serde_json::json!({
                                    "actor": actor_name,
                                    "activity": activity,
                                })
                            })
                        } else {
                            Err(ActorError::InvalidQuery { 
                                god: GodName::Dionysus, 
                                reason: "Actor no especificado".to_string() 
                            })
                        }
                    }
                    "all_actors_activity" => {
                        let analytics = self.analytics.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&analytics.actor_activity).unwrap_or_default()
                        })
                    }
                    "patient_alerts" => {
                        let patient_id = data.get("patient_id").and_then(|v| v.as_str());
                        
                        if let Some(pid) = patient_id {
                            let analytics = self.analytics.read().await;
                            let alerts = analytics.patient_alerts.get(pid).unwrap_or(&0);
                            Ok(ResponsePayload::Data { 
                                data: serde_json::json!({
                                    "patient_id": pid,
                                    "alert_count": alerts,
                                })
                            })
                        } else {
                            // Devolver todos los pacientes con alertas
                            let analytics = self.analytics.read().await;
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&analytics.patient_alerts).unwrap_or_default()
                            })
                        }
                    }
                    "top_patients_by_alerts" => {
                        let limit = data.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                        let analytics = self.analytics.read().await;
                        let top = analytics.get_top_alerts_with_limit(limit);
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&top).unwrap_or_default()
                        })
                    }
                    "error_summary" => {
                        let analytics = self.analytics.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "total_errors": analytics.event_counts.get("total_errors").unwrap_or(&0),
                                "recent_errors": analytics.error_log.iter().rev().take(20).collect::<Vec<_>>(),
                                "error_count_by_type": analytics.get_error_breakdown(),
                            })
                        })
                    }
                    "event_breakdown" => {
                        let analytics = self.analytics.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "event_counts": analytics.event_counts,
                                "total_by_actor": analytics.actor_activity,
                            })
                        })
                    }
                    "health_trend" => {
                        // Devolver health index actual y tendencia
                        let analytics = self.analytics.read().await;
                        let metrics = self.metrics.read().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "health_index": analytics.health_index,
                                "events_processed": metrics.events_processed,
                                "events_per_second": metrics.events_per_second,
                                "timestamp": Utc::now(),
                            })
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Dionysus, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                };
                
                // Guardar en caché si es exitoso
                if let Ok(ResponsePayload::Data { data: ref response_data }) = result {
                    let mut cache = self.query_cache.write().await;
                    cache.insert(cache_key, (Utc::now(), response_data.clone()));
                }
                
                result
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Dionysus, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::message::{ActorMessage, EventPayload};
    use serde_json::json;

    #[tokio::test]
    async fn test_dionysus_initialization() -> Result<(), ActorError> {
        let mut dionysus = Dionysus::new().await;
        dionysus.initialize().await?;
        
        assert_eq!(dionysus.name(), GodName::Dionysus);
        assert_eq!(dionysus.domain(), DivineDomain::Analysis);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_dionysus_event_processing() -> Result<(), ActorError> {
        let mut dionysus = Dionysus::new().await;
        dionysus.initialize().await?;

        // Simular evento de actor iniciado
        let event_msg = ActorMessage {
            id: "evt1".to_string(),
            from: Some(GodName::Athena),
            to: GodName::Dionysus,
            priority: crate::traits::message::MessagePriority::Normal,
            payload: MessagePayload::Event(EventPayload::ActorStarted { actor: GodName::Athena }),
            timestamp: Utc::now(),
            metadata: json!({}),
        };

        let response = dionysus.handle_message(event_msg).await?;
        assert!(matches!(response, ResponsePayload::Ack { .. }));

        // Verificar que se procesó el evento
        let metrics = dionysus.metrics.read().await;
        assert_eq!(metrics.events_processed, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_dionysus_health_check() -> Result<(), ActorError> {
        let dionysus = Dionysus::new().await;
        let health = dionysus.health_check().await;
        
        assert_eq!(health.god, GodName::Dionysus);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_dionysus_query_system_health() -> Result<(), ActorError> {
        let mut dionysus = Dionysus::new().await;
        dionysus.initialize().await?;

        // Procesar algunos eventos primero
        for i in 0..5 {
            let event_msg = ActorMessage {
                id: format!("evt{}", i),
                from: Some(GodName::Athena),
                to: GodName::Dionysus,
                priority: crate::traits::message::MessagePriority::Normal,
                payload: MessagePayload::Event(EventPayload::ActorStarted { actor: GodName::Athena }),
                timestamp: Utc::now(),
                metadata: json!({}),
            };
            dionysus.handle_message(event_msg).await?;
        }

        // Query de health
        let query = QueryPayload::Custom(json!({"query_type": "system_health"}));
        let query_msg = ActorMessage {
            id: "q1".to_string(),
            from: Some(GodName::Zeus),
            to: GodName::Dionysus,
            priority: crate::traits::message::MessagePriority::Normal,
            payload: MessagePayload::Query(query),
            timestamp: Utc::now(),
            metadata: json!({}),
        };

        let response = dionysus.handle_message(query_msg).await?;
        
        if let ResponsePayload::Data { data } = response {
            assert!(data.get("health_index").is_some());
            assert!(data.get("total_events").is_some());
        } else {
            panic!("Expected Data response");
        }

        Ok(())
    }
}
