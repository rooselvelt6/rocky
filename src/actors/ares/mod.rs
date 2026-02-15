// src/actors/ares/mod.rs
// OLYMPUS v15 - Ares: Dios de la Guerra y Resolución de Conflictos
// Responsabilidad: Detectar, mediar y resolver conflictos entre actores

#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use chrono::{Duration, Utc};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

// Submódulos
pub mod detector;
pub mod strategies;
pub mod history;

pub use detector::{ConflictDetector, Conflict, ConflictType, ConflictSeverity};
pub use strategies::{ResolutionStrategy, ResolutionResult, ConflictResolver};
pub use history::{ConflictHistory, ConflictStats};

/// Ares - Dios de la Resolución de Conflictos
/// Gestiona la detección y resolución de conflictos entre actores del sistema
#[derive(Debug)]
#[allow(dead_code)]
pub struct Ares {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Detector de conflictos
    detector: Arc<RwLock<ConflictDetector>>,
    /// Resolvedor de conflictos
    resolver: Arc<RwLock<ConflictResolver>>,
    /// Historial de conflictos
    history: Arc<RwLock<ConflictHistory>>,
    /// Conflictos activos
    active_conflicts: Arc<RwLock<HashMap<String, Conflict>>>,
    /// Estadísticas
    stats: Arc<RwLock<ConflictStats>>,
}

impl Ares {
    pub async fn new() -> Self {
        info!("⚔️ Ares: Inicializando sistema de resolución de conflictos...");
        
        Self {
            name: GodName::Ares,
            state: ActorState::new(GodName::Ares),
            config: ActorConfig::default(),
            detector: Arc::new(RwLock::new(ConflictDetector::new())),
            resolver: Arc::new(RwLock::new(ConflictResolver::new())),
            history: Arc::new(RwLock::new(ConflictHistory::new())),
            active_conflicts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ConflictStats::default())),
        }
    }

    /// Detecta un conflicto potencial
    pub async fn detect_conflict(
        &self,
        actor_a: GodName,
        actor_b: GodName,
        resource: &str,
        conflict_type: ConflictType,
    ) -> Result<Conflict, ActorError> {
        let mut detector = self.detector.write().await;
        let conflict = detector.detect(actor_a, actor_b, resource, conflict_type)?;
        
        // Registrar conflicto activo
        let mut active = self.active_conflicts.write().await;
        active.insert(conflict.id.clone(), conflict.clone());
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.total_detected += 1;
        stats.current_active += 1;
        
        warn!("⚔️ Ares: Conflicto detectado entre {:?} y {:?} por recurso '{}'",
            actor_a, actor_b, resource);
        
        Ok(conflict)
    }

    /// Resuelve un conflicto
    pub async fn resolve_conflict(
        &self,
        conflict_id: &str,
        strategy: ResolutionStrategy,
    ) -> Result<ResolutionResult, ActorError> {
        let mut active = self.active_conflicts.write().await;
        
        let mut conflict = active.remove(conflict_id)
            .ok_or_else(|| ActorError::NotFound { god: GodName::Ares })?;
        
        drop(active);
        
        // Resolver
        let mut resolver = self.resolver.write().await;
        let result = resolver.resolve(&mut conflict, strategy).await?;
        
        // Guardar en historial
        let mut history = self.history.write().await;
        history.mark_resolved(&conflict.id, result.clone())?;
        
        // Actualizar estadísticas
        let mut stats = self.stats.write().await;
        stats.total_resolved += 1;
        stats.current_active -= 1;
        
        if result.success {
            stats.successful_resolutions += 1;
            info!("⚔️ Ares: Conflicto {} resuelto exitosamente", conflict_id);
        } else {
            stats.failed_resolutions += 1;
            error!("⚔️ Ares: Fallo al resolver conflicto {}", conflict_id);
        }
        
        Ok(result)
    }

    /// Escalate un conflicto a Zeus
    pub async fn escalate_conflict(&self, conflict_id: &str, reason: &str) -> Result<(), ActorError> {
        let mut active = self.active_conflicts.write().await;
        
        if let Some(conflict) = active.get_mut(conflict_id) {
            conflict.escalate(reason);
            
            let mut stats = self.stats.write().await;
            stats.total_escalated += 1;
            
            warn!("⚔️ Ares: Conflicto {} escalado a Zeus: {}", conflict_id, reason);
            Ok(())
        } else {
            Err(ActorError::NotFound { god: GodName::Ares })
        }
    }

    /// Obtiene un conflicto activo
    pub async fn get_conflict(&self, conflict_id: &str) -> Option<Conflict> {
        let active = self.active_conflicts.read().await;
        active.get(conflict_id).cloned()
    }

    /// Lista conflictos activos
    pub async fn list_active_conflicts(&self) -> Vec<Conflict> {
        let active = self.active_conflicts.read().await;
        active.values().cloned().collect()
    }

    /// Lista conflictos por tipo
    pub async fn list_conflicts_by_type(&self, conflict_type: ConflictType) -> Vec<Conflict> {
        let active = self.active_conflicts.read().await;
        active.values()
            .filter(|c| c.conflict_type == conflict_type)
            .cloned()
            .collect()
    }

    /// Obtiene el historial de conflictos
    pub async fn get_conflict_history(&self, limit: usize) -> Vec<Conflict> {
        let history = self.history.read().await;
        history.get_recent(limit).iter().map(|entry| entry.conflict.clone()).collect()
    }

    /// Obtiene estadísticas
    pub async fn get_statistics(&self) -> ConflictStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Verifica si hay deadlock potencial
    pub async fn check_deadlock_potential(&self, actors: &[GodName]) -> Option<String> {
        let detector = self.detector.read().await;
        detector.check_circular_wait(actors)
    }

    /// Sugiere estrategia de resolución
    pub async fn suggest_strategy(&self, conflict: &Conflict) -> ResolutionStrategy {
        let resolver = self.resolver.read().await;
        resolver.suggest_strategy(conflict)
    }

    /// Limpia conflictos antiguos
    pub async fn cleanup_old_conflicts(&self, older_than: Duration) {
        let cutoff = Utc::now() - older_than;
        let mut active = self.active_conflicts.write().await;
        
        let to_remove: Vec<String> = active.iter()
            .filter(|(_, c)| c.detected_at < cutoff && c.is_resolved())
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in to_remove {
            active.remove(&id);
        }
    }
}

#[async_trait]
impl OlympianActor for Ares {
    fn name(&self) -> GodName { 
        GodName::Ares 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::ConflictResolution 
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
        let stats = self.get_statistics().await;
        serde_json::json!({
            "name": "Ares",
            "messages": self.state.message_count,
            "active_conflicts": stats.current_active,
            "total_resolved": stats.total_resolved,
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
        let stats = self.get_statistics().await;
        
        HealthStatus {
            god: self.name.clone(),
            status: self.state.status.clone(),
            uptime_seconds: (Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: stats.failed_resolutions as u64,
            last_error: None,
            memory_usage_mb: 0.0,
            timestamp: Utc::now(),
        }
    }

    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("⚔️ Ares: Sistema de resolución de conflictos v15 iniciado");
        info!("⚔️ Ares: Detector y resolvedor de conflictos activos");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("⚔️ Ares: Deteniendo sistema de resolución...");
        
        let active = self.active_conflicts.read().await;
        let count = active.len();
        
        if count > 0 {
            warn!("⚔️ Ares: {} conflictos activos al detenerse", count);
        }
        
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados
impl Ares {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("detect_conflict") => {
                        let actor_a = data.get("actor_a")
                            .and_then(|v| serde_json::from_value::<GodName>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Ares,
                                reason: "actor_a requerido".to_string(),
                            })?;
                        
                        let actor_b = data.get("actor_b")
                            .and_then(|v| serde_json::from_value::<GodName>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Ares,
                                reason: "actor_b requerido".to_string(),
                            })?;
                        
                        let resource = data.get("resource")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Ares,
                                reason: "resource requerido".to_string(),
                            })?;
                        
                        let conflict_type = data.get("type")
                            .and_then(|v| serde_json::from_value::<ConflictType>(v.clone()).ok())
                            .unwrap_or(ConflictType::Resource);
                        
                        let conflict = self.detect_conflict(actor_a, actor_b, resource, conflict_type).await?;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "conflict_id": conflict.id,
                                "severity": conflict.severity,
                            })
                        })
                    }
                    Some("resolve_conflict") => {
                        let conflict_id = data.get("conflict_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Ares,
                                reason: "conflict_id requerido".to_string(),
                            })?;
                        
                        let strategy = data.get("strategy")
                            .and_then(|v| serde_json::from_value::<ResolutionStrategy>(v.clone()).ok())
                            .unwrap_or(ResolutionStrategy::Priority);
                        
                        let result = self.resolve_conflict(conflict_id, strategy).await?;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "success": result.success,
                                "message": result.message,
                            })
                        })
                    }
                    Some("escalate_conflict") => {
                        let conflict_id = data.get("conflict_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Ares,
                                reason: "conflict_id requerido".to_string(),
                            })?;
                        
                        let reason = data.get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Escalado manualmente");
                        
                        self.escalate_conflict(conflict_id, reason).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Conflicto {} escalado", conflict_id) 
                        })
                    }
                    Some("cleanup_old") => {
                        let days = data.get("days").and_then(|v| v.as_u64()).unwrap_or(7);
                        self.cleanup_old_conflicts(Duration::days(days as i64)).await;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Conflictos de más de {} días limpiados", days) 
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Ares, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Ares, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let stats = self.get_statistics().await;
                Ok(ResponsePayload::Stats { 
                    data: serde_json::to_value(&stats).unwrap_or_default()
                })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                match query_type {
                    "get_conflict" => {
                        let conflict_id = data.get("conflict_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Ares,
                                reason: "conflict_id requerido".to_string(),
                            })?;
                        
                        if let Some(conflict) = self.get_conflict(conflict_id).await {
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&conflict).unwrap_or_default()
                            })
                        } else {
                            Err(ActorError::NotFound { god: GodName::Ares })
                        }
                    }
                    "list_active" => {
                        let conflicts = self.list_active_conflicts().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "conflicts": conflicts,
                                "count": conflicts.len(),
                            })
                        })
                    }
                    "list_by_type" => {
                        let conflict_type = data.get("type")
                            .and_then(|v| serde_json::from_value::<ConflictType>(v.clone()).ok())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Ares,
                                reason: "type requerido".to_string(),
                            })?;
                        
                        let conflicts = self.list_conflicts_by_type(conflict_type).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "conflicts": conflicts,
                                "count": conflicts.len(),
                            })
                        })
                    }
                    "get_history" => {
                        let limit = data.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                        let history = self.get_conflict_history(limit).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "conflicts": history,
                                "count": history.len(),
                            })
                        })
                    }
                    "check_deadlock" => {
                        let actors: Vec<GodName> = data.get("actors")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        
                        let deadlock = self.check_deadlock_potential(&actors).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "potential_deadlock": deadlock.is_some(),
                                "details": deadlock,
                            })
                        })
                    }
                    "suggest_strategy" => {
                        let conflict_id = data.get("conflict_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Ares,
                                reason: "conflict_id requerido".to_string(),
                            })?;
                        
                        if let Some(conflict) = self.get_conflict(conflict_id).await {
                            let strategy = self.suggest_strategy(&conflict).await;
                            Ok(ResponsePayload::Data { 
                                data: serde_json::json!({
                                    "suggested_strategy": strategy,
                                })
                            })
                        } else {
                            Err(ActorError::NotFound { god: GodName::Ares })
                        }
                    }
                    "statistics" => {
                        let stats = self.get_statistics().await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::to_value(&stats).unwrap_or_default()
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Ares, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Ares, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ares_initialization() -> Result<(), ActorError> {
        let mut ares = Ares::new().await;
        ares.initialize().await?;
        
        assert_eq!(ares.name(), GodName::Ares);
        assert_eq!(ares.domain(), DivineDomain::ConflictResolution);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_detect_and_resolve_conflict() -> Result<(), ActorError> {
        let ares = Ares::new().await;
        
        // Detectar conflicto
        let conflict = ares.detect_conflict(
            GodName::Zeus,
            GodName::Hades,
            "database_connection",
            ConflictType::Resource,
        ).await?;
        
        assert!(!conflict.id.is_empty());
        
        // Verificar que está activo
        let active = ares.list_active_conflicts().await;
        assert_eq!(active.len(), 1);
        
        // Resolver conflicto
        let result = ares.resolve_conflict(&conflict.id, ResolutionStrategy::Priority).await?;
        
        // Verificar que se resolvió
        let active = ares.list_active_conflicts().await;
        assert!(active.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_escalate_conflict() -> Result<(), ActorError> {
        let ares = Ares::new().await;
        
        let conflict = ares.detect_conflict(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        ).await?;
        
        ares.escalate_conflict(&conflict.id, "Requiere decisión de alto nivel").await?;
        
        let escalated = ares.get_conflict(&conflict.id).await.unwrap();
        assert!(escalated.is_escalated());
        
        Ok(())
    }
}
