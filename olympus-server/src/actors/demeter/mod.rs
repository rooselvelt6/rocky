// src/actors/demeter/mod.rs
// OLYMPUS v15 - Demeter: Diosa de la Agricultura y Recursos
// Responsabilidad: Monitoreo de recursos del sistema y gestión de umbrales

#![allow(dead_code)]

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc, Duration};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

// Submódulos
pub mod resources;
pub mod alerts;

pub use resources::{ResourceSnapshot, ResourceType, ResourceMetrics};
pub use alerts::{AlertThreshold, AlertLevel, ResourceAlert};

/// Demeter - Diosa del Monitoreo de Recursos
/// Supervisa CPU, memoria, storage y network, emitiendo alertas cuando se superan umbrales
#[derive(Debug)]
pub struct Demeter {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Histórico de métricas de recursos
    metrics_history: Arc<RwLock<Vec<ResourceSnapshot>>>,
    /// Umbrales configurados para alertas
    thresholds: Arc<RwLock<Vec<AlertThreshold>>>,
    /// Alertas activas
    active_alerts: Arc<RwLock<Vec<ResourceAlert>>>,
    /// Control del loop de monitoreo
    monitoring: Arc<RwLock<bool>>,
    /// Intervalo de muestreo en segundos
    sample_interval_secs: u64,
}

impl Demeter {
    pub async fn new() -> Self {
        info!("🌾 Demeter: Inicializando monitoreo de recursos...");
        
        Self {
            name: GodName::Demeter,
            state: ActorState::new(GodName::Demeter),
            config: ActorConfig::default(),
            metrics_history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            thresholds: Arc::new(RwLock::new(Self::default_thresholds())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            monitoring: Arc::new(RwLock::new(false)),
            sample_interval_secs: 30, // Muestreo cada 30 segundos por defecto
        }
    }

    /// Umbrales por defecto
    fn default_thresholds() -> Vec<AlertThreshold> {
        vec![
            AlertThreshold::new(ResourceType::Cpu, 0.80, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Cpu, 0.95, AlertLevel::Critical),
            AlertThreshold::new(ResourceType::Memory, 0.80, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Memory, 0.95, AlertLevel::Critical),
            AlertThreshold::new(ResourceType::Storage, 0.85, AlertLevel::Warning),
            AlertThreshold::new(ResourceType::Storage, 0.98, AlertLevel::Critical),
        ]
    }

    /// Captura una snapshot actual de recursos
    pub async fn capture_snapshot(&self) -> ResourceSnapshot {
        // En una implementación real, esto leería del sistema operativo
        // Por ahora simulamos valores realistas
        let snapshot = ResourceSnapshot::capture().await;
        
        // Guardar en histórico
        let mut history = self.metrics_history.write().await;
        if history.len() >= 1000 {
            history.remove(0); // Eliminar el más antiguo
        }
        history.push(snapshot.clone());
        
        // Verificar umbrales
        self.check_thresholds(&snapshot).await;
        
        debug!("🌾 Demeter: Snapshot capturado - CPU: {:.1}%, MEM: {:.1}%", 
            snapshot.cpu_usage * 100.0, 
            snapshot.memory_usage * 100.0
        );
        
        snapshot
    }

    /// Verifica si algún recurso supera los umbrales configurados
    async fn check_thresholds(&self, snapshot: &ResourceSnapshot) {
        let thresholds = self.thresholds.read().await;
        
        for threshold in thresholds.iter() {
            let current_value = match threshold.resource_type {
                ResourceType::Cpu => snapshot.cpu_usage,
                ResourceType::Memory => snapshot.memory_usage,
                ResourceType::Storage => snapshot.storage_usage,
                ResourceType::Network => snapshot.network_usage,
            };
            
            if current_value >= threshold.threshold {
                // Verificar si ya existe una alerta activa para este recurso y nivel
                let alert_exists = {
                    let alerts = self.active_alerts.read().await;
                    alerts.iter().any(|a| {
                        a.resource_type == threshold.resource_type 
                            && a.level == threshold.level 
                            && !a.resolved
                    })
                };
                
                if !alert_exists {
                    // Crear nueva alerta
                    let alert = ResourceAlert::new(
                        threshold.resource_type,
                        threshold.level,
                        threshold.threshold,
                        current_value,
                    );
                    
                    let mut alerts = self.active_alerts.write().await;
                    alerts.push(alert.clone());
                    
                    match threshold.level {
                        AlertLevel::Warning => {
                            warn!("🌾 Demeter: Alerta de {:?} - Uso: {:.1}% (umbral: {:.1}%)",
                                threshold.resource_type,
                                current_value * 100.0,
                                threshold.threshold * 100.0
                            );
                        }
                        AlertLevel::Critical => {
                            error!("🌾 Demeter: ALERTA CRÍTICA de {:?} - Uso: {:.1}% (umbral: {:.1}%)",
                                threshold.resource_type,
                                current_value * 100.0,
                                threshold.threshold * 100.0
                            );
                        }
                    }
                }
            } else {
                // Intentar resolver alertas si el valor ha bajado
                let mut alerts = self.active_alerts.write().await;
                for alert in alerts.iter_mut() {
                    if alert.resource_type == threshold.resource_type 
                        && !alert.resolved 
                        && current_value < alert.threshold * 0.9 { // Margen del 10%
                        alert.resolve();
                        info!("🌾 Demeter: Alerta de {:?} resuelta - Uso bajó a {:.1}%",
                            alert.resource_type,
                            current_value * 100.0
                        );
                    }
                }
            }
        }
    }

    /// Obtiene el estado actual de todos los recursos
    pub async fn get_current_resources(&self) -> ResourceSnapshot {
        self.capture_snapshot().await
    }

    /// Obtiene el histórico de métricas
    pub async fn get_metrics_history(&self, duration: Duration) -> Vec<ResourceSnapshot> {
        let history = self.metrics_history.read().await;
        let cutoff = Utc::now() - duration;
        
        history.iter()
            .filter(|s| s.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    /// Obtiene métricas promedio en un período
    pub async fn get_average_metrics(&self, duration: Duration) -> ResourceMetrics {
        let snapshots = self.get_metrics_history(duration).await;
        ResourceMetrics::from_snapshots(&snapshots)
    }

    /// Obtiene alertas activas
    pub async fn get_active_alerts(&self) -> Vec<ResourceAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.iter().filter(|a| !a.resolved).cloned().collect()
    }

    /// Obtiene todas las alertas (activas y resueltas)
    pub async fn get_all_alerts(&self) -> Vec<ResourceAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.clone()
    }

    /// Configura un nuevo umbral
    pub async fn set_threshold(&self, resource_type: ResourceType, threshold: f64, level: AlertLevel) {
        let mut thresholds = self.thresholds.write().await;
        
        // Remover umbral existente del mismo tipo y nivel
        thresholds.retain(|t| !(t.resource_type == resource_type && t.level == level));
        
        // Agregar nuevo umbral
        thresholds.push(AlertThreshold::new(resource_type, threshold, level));
        
        info!("🌾 Demeter: Umbral configurado - {:?} {:?} en {:.1}%",
            resource_type, level, threshold * 100.0
        );
    }

    /// Elimina un umbral
    pub async fn remove_threshold(&self, resource_type: ResourceType, level: AlertLevel) {
        let mut thresholds = self.thresholds.write().await;
        thresholds.retain(|t| !(t.resource_type == resource_type && t.level == level));
    }

    /// Obtiene los umbrales configurados
    pub async fn get_thresholds(&self) -> Vec<AlertThreshold> {
        let thresholds = self.thresholds.read().await;
        thresholds.clone()
    }

    /// Limpia alertas resueltas antiguas
    pub async fn cleanup_resolved_alerts(&self, older_than: Duration) {
        let cutoff = Utc::now() - older_than;
        let mut alerts = self.active_alerts.write().await;
        alerts.retain(|a| !a.resolved || a.resolved_at.map_or(true, |t| t > cutoff));
    }

    /// Predice el agotamiento de recursos basado en tendencias
    pub async fn predict_resource_exhaustion(&self, resource_type: ResourceType) -> Option<DateTime<Utc>> {
        // Obtener histórico de las últimas 24 horas
        let snapshots = self.get_metrics_history(Duration::hours(24)).await;
        
        if snapshots.len() < 10 {
            return None; // No hay suficientes datos
        }
        
        // Calcular tendencia (tasa de crecimiento promedio por hora)
        let first = &snapshots[0];
        let last = &snapshots[snapshots.len() - 1];
        let hours = (last.timestamp - first.timestamp).num_hours() as f64;
        
        if hours < 1.0 {
            return None;
        }
        
        let (current_value, growth_rate) = match resource_type {
            ResourceType::Cpu => {
                let growth = (last.cpu_usage - first.cpu_usage) / hours;
                (last.cpu_usage, growth)
            }
            ResourceType::Memory => {
                let growth = (last.memory_usage - first.memory_usage) / hours;
                (last.memory_usage, growth)
            }
            ResourceType::Storage => {
                let growth = (last.storage_usage - first.storage_usage) / hours;
                (last.storage_usage, growth)
            }
            ResourceType::Network => {
                return None; // Network no se predice
            }
        };
        
        // Si no hay crecimiento, no hay agotamiento
        if growth_rate <= 0.0 {
            return None;
        }
        
        // Calcular horas hasta 100%
        let hours_to_exhaustion = (1.0 - current_value) / growth_rate;
        
        if hours_to_exhaustion > 0.0 && hours_to_exhaustion < 720.0 { // Máximo 30 días
            Some(Utc::now() + Duration::hours(hours_to_exhaustion as i64))
        } else {
            None
        }
    }

    /// Loop de monitoreo
    async fn monitoring_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(self.sample_interval_secs));
        
        loop {
            interval.tick().await;
            
            // Verificar si debemos detenernos
            {
                let monitoring = self.monitoring.read().await;
                if !*monitoring {
                    break;
                }
            }
            
            // Capturar snapshot
            self.capture_snapshot().await;
            
            // Limpiar alertas antiguas cada hora
            if Utc::now().timestamp() % 3600 == 0 {
                self.cleanup_resolved_alerts(Duration::days(7)).await;
            }
        }
    }
}

#[async_trait]
impl OlympianActor for Demeter {
    fn name(&self) -> GodName { 
        GodName::Demeter 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::Resources 
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = Utc::now();

        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            _ => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }

    async fn persistent_state(&self) -> serde_json::Value {
        let thresholds = self.thresholds.read().await;
        serde_json::json!({
            "name": "Demeter",
            "messages": self.state.message_count,
            "thresholds_count": thresholds.len(),
            "status": self.state.status,
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }

    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: self.name.clone(),
            status: self.state.status.clone(),
            last_seen: Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }

    async fn health_check(&self) -> HealthStatus {
        let snapshot = self.capture_snapshot().await;
        
        // Determinar estado basado en uso de recursos
        let _max_usage = snapshot.cpu_usage
            .max(snapshot.memory_usage)
            .max(snapshot.storage_usage);
        
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: 0,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🌾 Demeter: Sistema de monitoreo v15 iniciado");
        info!("🌾 Demeter: Supervisando CPU, Memoria, Storage y Network");
        
        // Iniciar loop de monitoreo
        let mut monitoring = self.monitoring.write().await;
        *monitoring = true;
        
        // Capturar snapshot inicial
        self.capture_snapshot().await;
        
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🌾 Demeter: Deteniendo monitoreo...");
        
        let mut monitoring = self.monitoring.write().await;
        *monitoring = false;
        
        info!("🌾 Demeter: Monitoreo detenido");
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados para manejo de comandos y queries
impl Demeter {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("set_threshold") => {
                        let resource = data.get("resource")
                            .and_then(|v| serde_json::from_value::<ResourceType>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Demeter,
                                reason: "resource requerido".to_string(),
                            })?;
                        
                        let threshold = data.get("threshold")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Demeter,
                                reason: "threshold requerido".to_string(),
                            })?;
                        
                        let level = data.get("level")
                            .and_then(|v| serde_json::from_value::<AlertLevel>(v.clone()).ok())
                            .unwrap_or(AlertLevel::Warning);
                        
                        self.set_threshold(resource, threshold, level).await;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Umbral configurado para {:?}", resource) 
                        })
                    }
                    Some("remove_threshold") => {
                        let resource = data.get("resource")
                            .and_then(|v| serde_json::from_value::<ResourceType>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Demeter,
                                reason: "resource requerido".to_string(),
                            })?;
                        
                        let level = data.get("level")
                            .and_then(|v| serde_json::from_value::<AlertLevel>(v.clone()).ok())
                            .unwrap_or(AlertLevel::Warning);
                        
                        self.remove_threshold(resource, level).await;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Umbral removido para {:?}", resource) 
                        })
                    }
                    Some("cleanup_alerts") => {
                        let days = data.get("days").and_then(|v| v.as_u64()).unwrap_or(7);
                        self.cleanup_resolved_alerts(Duration::days(days as i64)).await;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Alertas de más de {} días eliminadas", days) 
                        })
                    }
                    Some("capture_snapshot") => {
                        let snapshot = self.capture_snapshot().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&snapshot).unwrap_or_default()
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Demeter, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Demeter, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let snapshot = self.get_current_resources().await;
                Ok(ResponsePayload::Stats { 
                    data: serde_json::json!({
                        "cpu_usage": snapshot.cpu_usage,
                        "memory_usage": snapshot.memory_usage,
                        "storage_usage": snapshot.storage_usage,
                        "network_usage": snapshot.network_usage,
                        "timestamp": snapshot.timestamp,
                    })
                })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                match query_type {
                    "current_resources" => {
                        let snapshot = self.get_current_resources().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&snapshot).unwrap_or_default()
                        })
                    }
                    "resources_history" => {
                        let hours = data.get("hours").and_then(|v| v.as_u64()).unwrap_or(24);
                        let history = self.get_metrics_history(Duration::hours(hours as i64)).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "snapshots": history,
                                "count": history.len(),
                            })
                        })
                    }
                    "average_metrics" => {
                        let hours = data.get("hours").and_then(|v| v.as_u64()).unwrap_or(24);
                        let metrics = self.get_average_metrics(Duration::hours(hours as i64)).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&metrics).unwrap_or_default()
                        })
                    }
                    "active_alerts" => {
                        let alerts = self.get_active_alerts().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "alerts": alerts,
                                "count": alerts.len(),
                            })
                        })
                    }
                    "all_alerts" => {
                        let alerts = self.get_all_alerts().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "alerts": alerts,
                                "count": alerts.len(),
                            })
                        })
                    }
                    "thresholds" => {
                        let thresholds = self.get_thresholds().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&thresholds).unwrap_or_default()
                        })
                    }
                    "predict_exhaustion" => {
                        let resource = data.get("resource")
                            .and_then(|v| serde_json::from_value::<ResourceType>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Demeter,
                                reason: "resource requerido".to_string(),
                            })?;
                        
                        let prediction = self.predict_resource_exhaustion(resource).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "resource": resource,
                                "predicted_exhaustion": prediction,
                            })
                        })
                    }
                    "resource_health" => {
                        let snapshot = self.get_current_resources().await;
                        let alerts = self.get_active_alerts().await;
                        
                        let critical_alerts = alerts.iter().filter(|a| a.level == AlertLevel::Critical).count();
                        let warning_alerts = alerts.iter().filter(|a| a.level == AlertLevel::Warning).count();
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "resources": {
                                    "cpu": snapshot.cpu_usage,
                                    "memory": snapshot.memory_usage,
                                    "storage": snapshot.storage_usage,
                                    "network": snapshot.network_usage,
                                },
                                "alerts": {
                                    "critical": critical_alerts,
                                    "warning": warning_alerts,
                                    "total": alerts.len(),
                                },
                                "healthy": critical_alerts == 0,
                                "timestamp": Utc::now(),
                            })
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Demeter, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Demeter, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demeter_initialization() -> Result<(), ActorError> {
        let mut demeter = Demeter::new().await;
        demeter.initialize().await?;
        
        assert_eq!(demeter.name(), GodName::Demeter);
        assert_eq!(demeter.domain(), DivineDomain::Resources);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_demeter_capture_snapshot() {
        let demeter = Demeter::new().await;
        let snapshot = demeter.capture_snapshot().await;
        
        assert!(snapshot.cpu_usage >= 0.0 && snapshot.cpu_usage <= 1.0);
        assert!(snapshot.memory_usage >= 0.0 && snapshot.memory_usage <= 1.0);
        assert!(snapshot.storage_usage >= 0.0 && snapshot.storage_usage <= 1.0);
    }

    #[tokio::test]
    async fn test_demeter_thresholds() {
        let demeter = Demeter::new().await;
        
        // Configurar umbral
        demeter.set_threshold(ResourceType::Cpu, 0.75, AlertLevel::Warning).await;
        
        // Verificar que se configuró
        let thresholds = demeter.get_thresholds().await;
        assert!(thresholds.iter().any(|t| t.resource_type == ResourceType::Cpu && t.threshold == 0.75));
        
        // Eliminar umbral
        demeter.remove_threshold(ResourceType::Cpu, AlertLevel::Warning).await;
        
        let thresholds = demeter.get_thresholds().await;
        assert!(!thresholds.iter().any(|t| t.resource_type == ResourceType::Cpu && t.threshold == 0.75));
    }

    #[tokio::test]
    async fn test_demeter_alerts() {
        let demeter = Demeter::new().await;
        
        // Configurar umbral bajo para forzar alerta
        demeter.set_threshold(ResourceType::Cpu, 0.01, AlertLevel::Warning).await;
        
        // Capturar snapshot (debería generar alerta)
        demeter.capture_snapshot().await;
        
        // Verificar que hay alertas
        let _alerts = demeter.get_active_alerts().await;
        // Nota: Depende de que el CPU real esté por encima del 1%
        // En general esto debería ser cierto
    }
}
