// src/actors/moirai/mod.rs
// OLYMPUS v15 - Moirai: Diosas del Destino y Predicciones Clínicas
// Responsabilidad: Lifecycle de pacientes, predicciones de outcomes y análisis de trayectorias

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use chrono::{Utc, Duration};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload};
use crate::errors::ActorError;

// Submódulos
pub mod threads;
pub mod predictions;
pub mod trajectories;
pub mod fate;

pub use threads::{PatientThread, ThreadStatus, ThreadEvent, TrajectoryPoint, FateOutcome};
pub use predictions::{PredictionEngine, ClinicalPrediction, PredictionType, RiskAssessment};
pub use trajectories::TrajectoryAnalyzer;
pub use fate::FateEngine;

/// Moirai - Diosas del Destino y Predicciones
/// Gestiona el ciclo de vida de pacientes, predice outcomes y analiza trayectorias clínicas
#[derive(Debug)]
pub struct Moirai {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    /// Threads (hilos del destino) para cada paciente
    threads: Arc<RwLock<HashMap<String, PatientThread>>>,
    /// Motor de predicciones
    prediction_engine: Arc<RwLock<PredictionEngine>>,
    /// Analizador de trayectorias
    trajectory_analyzer: Arc<RwLock<TrajectoryAnalyzer>>,
    /// Motor del destino
    fate_engine: Arc<RwLock<FateEngine>>,
    /// Histórico de predicciones
    prediction_history: Arc<RwLock<Vec<ClinicalPrediction>>>,
}

impl Moirai {
    pub async fn new() -> Self {
        info!("🧵 Moirai: Inicializando sistema de predicciones clínicas...");
        
        Self {
            name: GodName::Moirai,
            state: ActorState::new(GodName::Moirai),
            config: ActorConfig::default(),
            threads: Arc::new(RwLock::new(HashMap::new())),
            prediction_engine: Arc::new(RwLock::new(PredictionEngine::new())),
            trajectory_analyzer: Arc::new(RwLock::new(TrajectoryAnalyzer::new())),
            fate_engine: Arc::new(RwLock::new(FateEngine::new())),
            prediction_history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }

    /// Crea un nuevo thread para un paciente
    pub async fn create_thread(&self, patient_id: &str, initial_data: serde_json::Value) -> Result<PatientThread, ActorError> {
        let mut threads = self.threads.write().await;
        
        if threads.contains_key(patient_id) {
            return Err(ActorError::AlreadyRunning { god: GodName::Moirai });
        }
        
        let thread = PatientThread::new(patient_id, initial_data.clone());
        threads.insert(patient_id.to_string(), thread.clone());
        
        info!("🧵 Moirai: Thread creado para paciente {}", patient_id);
        
        // Generar predicción inicial
        self.generate_initial_prediction(patient_id, &initial_data).await?;
        
        Ok(thread)
    }

    /// Actualiza el estado de un thread
    pub async fn update_thread(&self, patient_id: &str, clinical_data: serde_json::Value) -> Result<(), ActorError> {
        let mut threads = self.threads.write().await;
        
        if let Some(thread) = threads.get_mut(patient_id) {
            thread.add_event(ThreadEvent::ClinicalUpdate {
                timestamp: Utc::now(),
                data: clinical_data.clone(),
            });
            
            // Analizar trayectoria
            let trajectory = self.analyze_trajectory(patient_id).await?;
            thread.trajectory = Some(trajectory);
            
            // Actualizar predicción
            self.update_prediction(patient_id, &clinical_data).await?;
            
            debug!("🧵 Moirai: Thread actualizado para paciente {}", patient_id);
            Ok(())
        } else {
            Err(ActorError::NotFound { god: GodName::Moirai })
        }
    }

    /// Obtiene un thread por ID de paciente
    pub async fn get_thread(&self, patient_id: &str) -> Option<PatientThread> {
        let threads = self.threads.read().await;
        threads.get(patient_id).cloned()
    }

    /// Lista todos los threads activos
    pub async fn list_threads(&self, status_filter: Option<ThreadStatus>) -> Vec<PatientThread> {
        let threads = self.threads.read().await;
        threads.values()
            .filter(|t| status_filter.as_ref().map_or(true, |s| t.status == *s))
            .cloned()
            .collect()
    }

    /// Cierra un thread (paciente dado de alta o fallecido)
    pub async fn close_thread(&self, patient_id: &str, outcome: FateOutcome) -> Result<(), ActorError> {
        let mut threads = self.threads.write().await;
        
        if let Some(thread) = threads.get_mut(patient_id) {
            thread.close(outcome);
            info!("🧵 Moirai: Thread cerrado para paciente {} - Outcome: {:?}", 
                patient_id, outcome);
            Ok(())
        } else {
            Err(ActorError::NotFound { god: GodName::Moirai })
        }
    }

    /// Genera predicción inicial para un paciente
    async fn generate_initial_prediction(&self, patient_id: &str, clinical_data: &serde_json::Value) -> Result<(), ActorError> {
        let engine = self.prediction_engine.read().await;
        
        let prediction = engine.predict_outcome(
            patient_id,
            clinical_data,
            PredictionType::RiesgoMortality,
        )?;
        
        let mut history = self.prediction_history.write().await;
        if history.len() >= 1000 {
            history.remove(0);
        }
        history.push(prediction);
        
        Ok(())
    }

    /// Actualiza la predicción basada en nuevos datos
    async fn update_prediction(&self, patient_id: &str, clinical_data: &serde_json::Value) -> Result<(), ActorError> {
        let thread = self.get_thread(patient_id).await;
        
        if let Some(thread) = thread {
            let engine = self.prediction_engine.read().await;
            
            // Determinar qué tipo de predicción hacer basado en estado
            let prediction_type = if thread.status == ThreadStatus::Critical {
                PredictionType::DeteriorationRisk
            } else {
                PredictionType::RecoveryProbability
            };
            
            let prediction = engine.predict_outcome(
                patient_id,
                clinical_data,
                prediction_type,
            )?;
            
            let mut history = self.prediction_history.write().await;
            if history.len() >= 1000 {
                history.remove(0);
            }
            history.push(prediction);
        }
        
        Ok(())
    }

    /// Analiza la trayectoria clínica de un paciente
    async fn analyze_trajectory(&self, patient_id: &str) -> Result<TrajectoryPoint, ActorError> {
        let thread = self.get_thread(patient_id).await;
        
        if let Some(thread) = thread {
            let analyzer = self.trajectory_analyzer.read().await;
            let trajectory = analyzer.analyze(&thread.events)?;
            Ok(trajectory)
        } else {
            Err(ActorError::NotFound { god: GodName::Moirai })
        }
    }

    /// Obtiene predicciones para un paciente
    pub async fn get_predictions(&self, patient_id: &str) -> Vec<ClinicalPrediction> {
        let history = self.prediction_history.read().await;
        history.iter()
            .filter(|p| p.patient_id == patient_id)
            .cloned()
            .collect()
    }

    /// Obtiene el último riesgo calculado para un paciente
    pub async fn get_current_risk(&self, patient_id: &str) -> Option<RiskAssessment> {
        let predictions = self.get_predictions(patient_id).await;
        predictions.last().map(|p| p.risk_assessment.clone())
    }

    /// Predice tiempo estimado de estancia UCI
    pub async fn predict_los(&self, patient_id: &str) -> Result<Duration, ActorError> {
        let thread = self.get_thread(patient_id).await
            .ok_or(ActorError::NotFound { god: GodName::Moirai })?;
        
        let engine = self.prediction_engine.read().await;
        engine.predict_length_of_stay(&thread)
    }

    /// Obtiene casos similares históricos
    pub async fn get_similar_cases(&self, patient_id: &str, limit: usize) -> Result<Vec<String>, ActorError> {
        let thread = self.get_thread(patient_id).await
            .ok_or(ActorError::NotFound { god: GodName::Moirai })?;
        
        let engine = self.fate_engine.read().await;
        engine.find_similar_cases(&thread, limit).await
    }

    /// Genera recomendaciones basadas en predicciones
    pub async fn generate_recommendations(&self, patient_id: &str) -> Result<Vec<String>, ActorError> {
        let thread = self.get_thread(patient_id).await
            .ok_or(ActorError::NotFound { god: GodName::Moirai })?;
        
        let predictions = self.get_predictions(patient_id).await;
        let engine = self.prediction_engine.read().await;
        
        engine.generate_recommendations(&thread, &predictions)
    }

    /// Obtiene estadísticas de predicciones
    pub async fn get_prediction_statistics(&self) -> PredictionStatistics {
        let history = self.prediction_history.read().await;
        let threads = self.threads.read().await;
        
        let total_predictions = history.len();
        let mortality_predictions = history.iter()
            .filter(|p| p.prediction_type == PredictionType::RiesgoMortality)
            .count();
        
        let avg_confidence = if total_predictions > 0 {
            history.iter().map(|p| p.confidence).sum::<f64>() / total_predictions as f64
        } else {
            0.0
        };

        PredictionStatistics {
            total_threads: threads.len(),
            active_threads: threads.values().filter(|t| t.is_active()).count(),
            total_predictions,
            mortality_predictions,
            average_confidence: avg_confidence,
        }
    }
}

/// Estadísticas de predicciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStatistics {
    pub total_threads: usize,
    pub active_threads: usize,
    pub total_predictions: usize,
    pub mortality_predictions: usize,
    pub average_confidence: f64,
}

#[async_trait]
impl OlympianActor for Moirai {
    fn name(&self) -> GodName { 
        GodName::Moirai 
    }
    
    fn domain(&self) -> DivineDomain { 
        DivineDomain::Predictions 
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
        let threads = self.threads.read().await;
        serde_json::json!({
            "name": "Moirai",
            "messages": self.state.message_count,
            "active_threads": threads.values().filter(|t| t.is_active()).count(),
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
        let _threads = self.threads.read().await;
        
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
        info!("🧵 Moirai: Sistema de predicciones clínicas v15 iniciado");
        info!("🧵 Moirai: Ciclo de vida de pacientes, predicciones y trayectorias activos");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🧵 Moirai: Deteniendo sistema de predicciones...");
        info!("🧵 Moirai: {} threads activos cerrados", self.threads.read().await.len());
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

// Métodos privados para manejo de comandos y queries
impl Moirai {
    async fn handle_command(&self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str());
                
                match action {
                    Some("create_thread") => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let initial_data = data.get("initial_data").cloned().unwrap_or_default();
                        
                        let _thread = self.create_thread(patient_id, initial_data).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Thread creado para paciente {}", patient_id) 
                        })
                    }
                    Some("update_thread") => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let clinical_data = data.get("clinical_data").cloned().unwrap_or_default();
                        
                        self.update_thread(patient_id, clinical_data).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Thread actualizado para paciente {}", patient_id) 
                        })
                    }
                    Some("close_thread") => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidCommand {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let outcome = data.get("outcome")
                            .and_then(|v| serde_json::from_value::<FateOutcome>(v.clone()).ok())
                            .unwrap_or(FateOutcome::Undetermined);
                        
                        self.close_thread(patient_id, outcome).await?;
                        
                        Ok(ResponsePayload::Success { 
                            message: format!("Thread cerrado para paciente {}", patient_id) 
                        })
                    }
                    _ => Err(ActorError::InvalidCommand { 
                        god: GodName::Moirai, 
                        reason: format!("Acción '{}' no soportada", action.unwrap_or("unknown")) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidCommand { 
                god: GodName::Moirai, 
                reason: "Comando no soportado".to_string() 
            }),
        }
    }

    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Metrics => {
                let stats = self.get_prediction_statistics().await;
                Ok(ResponsePayload::Stats { 
                    data: serde_json::to_value(&stats).unwrap_or_default()
                })
            }
            QueryPayload::Custom(data) => {
                let query_type = data.get("query_type").and_then(|v| v.as_str()).unwrap_or("");
                
                match query_type {
                    "get_thread" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        if let Some(thread) = self.get_thread(patient_id).await {
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&thread).unwrap_or_default()
                            })
                        } else {
                            Err(ActorError::NotFound { god: GodName::Moirai })
                        }
                    }
                    "list_threads" => {
                        let status = data.get("status")
                            .and_then(|v| serde_json::from_value::<ThreadStatus>(v.clone()).ok());
                        let threads = self.list_threads(status).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "threads": threads,
                                "count": threads.len(),
                            })
                        })
                    }
                    "get_predictions" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let predictions = self.get_predictions(patient_id).await;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "predictions": predictions,
                                "count": predictions.len(),
                            })
                        })
                    }
                    "get_current_risk" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        if let Some(risk) = self.get_current_risk(patient_id).await {
                            Ok(ResponsePayload::Data { 
                                data: serde_json::to_value(&risk).unwrap_or_default()
                            })
                        } else {
                            Err(ActorError::NotFound { god: GodName::Moirai })
                        }
                    }
                    "predict_los" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let los = self.predict_los(patient_id).await?;
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "predicted_days": los.num_days(),
                                "predicted_hours": los.num_hours(),
                            })
                        })
                    }
                    "get_similar_cases" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let limit = data.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                        let similar = self.get_similar_cases(patient_id, limit).await?;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "similar_cases": similar,
                                "count": similar.len(),
                            })
                        })
                    }
                    "get_recommendations" => {
                        let patient_id = data.get("patient_id")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| ActorError::InvalidQuery {
                                god: GodName::Moirai,
                                reason: "patient_id requerido".to_string(),
                            })?;
                        
                        let recommendations = self.generate_recommendations(patient_id).await?;
                        
                        Ok(ResponsePayload::Data { 
                            data: serde_json::json!({
                                "recommendations": recommendations,
                            })
                        })
                    }
                    _ => Err(ActorError::InvalidQuery { 
                        god: GodName::Moirai, 
                        reason: format!("Query type '{}' no soportado", query_type) 
                    }),
                }
            }
            _ => Err(ActorError::InvalidQuery { 
                god: GodName::Moirai, 
                reason: "Query no soportado".to_string() 
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_moirai_initialization() -> Result<(), ActorError> {
        let mut moirai = Moirai::new().await;
        moirai.initialize().await?;
        
        assert_eq!(moirai.name(), GodName::Moirai);
        assert_eq!(moirai.domain(), DivineDomain::Predictions);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_create_thread() -> Result<(), ActorError> {
        let moirai = Moirai::new().await;
        
        let thread = moirai.create_thread("patient_001", serde_json::json!({
            "apache_ii": 15,
            "sofa": 8,
        })).await?;
        
        assert_eq!(thread.patient_id, "patient_001");
        assert!(thread.is_active());
        
        // Verificar que existe
        let retrieved = moirai.get_thread("patient_001").await;
        assert!(retrieved.is_some());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_threads() -> Result<(), ActorError> {
        let moirai = Moirai::new().await;
        
        moirai.create_thread("patient_001", serde_json::json!({})).await?;
        moirai.create_thread("patient_002", serde_json::json!({})).await?;
        
        let threads = moirai.list_threads(None).await;
        assert_eq!(threads.len(), 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_close_thread() -> Result<(), ActorError> {
        let moirai = Moirai::new().await;
        
        moirai.create_thread("patient_001", serde_json::json!({})).await?;
        moirai.close_thread("patient_001", FateOutcome::Heroic).await?;
        
        let thread = moirai.get_thread("patient_001").await.unwrap();
        assert!(!thread.is_active());
        assert_eq!(thread.outcome, Some(FateOutcome::Heroic));
        
        Ok(())
    }
}
