// src/actors/chaos/mod.rs
// OLYMPUS v15 - Chaos: Dios de la Entropía y Pruebas Caos

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::info;

use crate::actors::{GodName, DivineDomain, ActorConfig, ActorState};
use crate::traits::{OlympianActor, CommandPayload, ResponsePayload};
use crate::errors::ActorError;

pub mod failure_injection;
pub mod experiments;
pub mod monitoring;
pub mod learning;
pub mod recovery;
pub mod injection;
pub mod impact;

use failure_injection::{FailureInjector, FailureType, FailureSeverity};
use experiments::{ExperimentManager, Experiment, ChaosStrategy};
use monitoring::{ChaosMonitor, ImpactAnalyzer, ImpactMetrics};
use learning::ChaosLearner;

/// Chaos - Dios de la Entropía y Pruebas Caos
#[derive(Debug, Clone)]
pub struct Chaos {
    name: GodName,
    domain: DivineDomain,
    state: ActorState,
    config: Arc<RwLock<ChaosConfig>>,
}

/// Configuración de Chaos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    pub base_failure_probability: f64,
    pub max_concurrent_experiments: usize,
    pub max_experiment_duration: u64,
    pub protected_actors: Vec<GodName>,
    pub allowed_strategies: Vec<ChaosStrategy>,
    pub auto_mode: bool,
    pub auto_experiment_interval: u64,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            base_failure_probability: 0.05,
            max_concurrent_experiments: 3,
            max_experiment_duration: 300,
            protected_actors: vec![GodName::Zeus],
            allowed_strategies: vec![
                ChaosStrategy::RandomFailure,
                ChaosStrategy::LatencyInjection,
                ChaosStrategy::NetworkPartition,
                ChaosStrategy::ResourceExhaustion,
                ChaosStrategy::CascadingFailure,
            ],
            auto_mode: false,
            auto_experiment_interval: 600,
        }
    }
}

/// Comandos específicos de Chaos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosCommand {
    StartExperiment {
        strategy: ChaosStrategy,
        target_actors: Vec<GodName>,
        duration: Option<u64>,
        intensity: f64,
    },
    StopExperiment {
        experiment_id: String,
    },
    InjectFailure {
        target: GodName,
        failure_type: FailureType,
        severity: FailureSeverity,
        duration: Option<u64>,
    },
    ConfigureChaos {
        config: ChaosConfig,
    },
    EnableAutoMode {
        enabled: bool,
    },
    GetStatus,
    GetExperimentMetrics,
    CleanupExperiments {
        older_than_hours: u64,
    },
    ExportResults {
        format: ExportFormat,
    },
}

/// Eventos de Chaos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosEvent {
    ExperimentStarted {
        experiment_id: String,
        strategy: ChaosStrategy,
        targets: Vec<GodName>,
    },
    ExperimentCompleted {
        experiment_id: String,
        results: ExperimentResults,
    },
    FailureInjected {
        target: GodName,
        failure_type: FailureType,
        severity: FailureSeverity,
    },
    ImpactDetected {
        experiment_id: String,
        metrics: ImpactMetrics,
    },
    RecoveryDetected {
        experiment_id: String,
        recovery_time: u64,
        success: bool,
    },
}

/// Resultados de experimento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub experiment_id: String,
    pub strategy: ChaosStrategy,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: u64,
    pub success: bool,
    pub impact_metrics: ImpactMetrics,
    pub failures_injected: u32,
    pub affected_actors: Vec<GodName>,
    pub observations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Formatos de exportación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Yaml,
    Prometheus,
}

#[async_trait]
impl OlympianActor for Chaos {
    fn name(&self) -> GodName {
        self.name
    }

    fn domain(&self) -> DivineDomain {
        self.domain
    }

    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🌀 Iniciando Chaos - Dios de la Entropía");
        self.state.status = crate::traits::ActorStatus::Healthy;
        Ok(())
    }

    async fn handle_message(&mut self, msg: crate::traits::message::ActorMessage) -> Result<ResponsePayload, ActorError> {
        // Implementación básica
        Ok(ResponsePayload::Success { message: "message_handled".to_string() })
    }

    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "state": self.state
        })
    }

    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        // Implementación básica
        Ok(())
    }

    fn heartbeat(&self) -> crate::traits::GodHeartbeat {
        crate::traits::GodHeartbeat {
            god: self.name,
            status: crate::traits::ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.2,
            memory_usage_mb: 45.0,
            uptime_seconds: 0,
        }
    }

    async fn health_check(&self) -> crate::traits::HealthStatus {
        crate::traits::HealthStatus::healthy(self.name)
    }

    fn config(&self) -> Option<&ActorConfig> {
        None
    }

    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🌀 Deteniendo Chaos - Finalizando experimentos activos");
        self.state.status = crate::traits::ActorStatus::Dead;
        Ok(())
    }

    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Chaos {
    /// Crea una nueva instancia de Chaos
    pub fn new() -> Self {
        let name = GodName::Chaos;
        let domain = DivineDomain::Testing;
        
        Self {
            name,
            domain,
            state: ActorState::new(name),
            config: Arc::new(RwLock::new(ChaosConfig::default())),
        }
    }
    
    /// Inicializa con configuración
    pub async fn with_config(config: ActorConfig) -> Result<Self, ActorError> {
        let chaos_config = serde_json::from_value(serde_json::json!({})).ok().unwrap_or_default();
        
        let name = GodName::Chaos;
        let chaos = Self {
            name,
            domain: DivineDomain::Testing,
            state: ActorState::new(name),
            config: Arc::new(RwLock::new(chaos_config)),
        };
        
        Ok(chaos)
    }
}

/// Queries específicos de Chaos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosQuery {
    GetExperiments,
    GetExperimentResults {
        experiment_id: String,
    },
    GetImpactMetrics,
    GetLearningInsights,
}