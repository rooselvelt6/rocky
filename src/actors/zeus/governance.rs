// src/actors/zeus/governance.rs
// OLYMPUS v13 - Zeus Governance Controller
// Decisiones de alto nivel del Olimpo

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Decisión de gobernanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceDecision {
    NoAction,
    RestartActor { actor: super::GodName, strategy: super::RecoveryStrategy },
    RestartAll,
    EmergencyShutdown { reason: String },
    ScaleResources { actor: super::GodName, resources: serde_json::Value },
    NotifyStakeholders { message: String },
}

/// Situación de gobernanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceSituation {
    ActorUnhealthy { actor: super::GodName, error_count: u64 },
    MultipleActorsUnhealthy { count: usize },
    HighErrorRate { rate: f64 },
    PerformanceDegradation { latency_ms: u64 },
    ResourceExhaustion { resource: String, usage_percent: f64 },
    SecurityBreach { severity: SecuritySeverity },
    SystemHealthy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Controlador de gobernanza
#[derive(Debug, Clone)]
pub struct GovernanceController {
    decisions: Arc<RwLock<Vec<GovernanceRecord>>>,
    thresholds: GovernanceThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub situation: GovernanceSituation,
    pub decision: GovernanceDecision,
    pub actor: Option<super::GodName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceThresholds {
    pub max_errors_before_restart: u64,
    pub max_unhealthy_actors: usize,
    pub max_error_rate: f64,
    pub max_latency_ms: u64,
    pub max_resource_usage_percent: f64,
}

impl Default for GovernanceThresholds {
    fn default() -> Self {
        Self {
            max_errors_before_restart: 10,
            max_unhealthy_actors: 2,
            max_error_rate: 0.1,
            max_latency_ms: 1000,
            max_resource_usage_percent: 90.0,
        }
    }
}

impl GovernanceController {
    pub fn new() -> Self {
        Self {
            decisions: Arc::new(RwLock::new(Vec::new())),
            thresholds: GovernanceThresholds::default(),
        }
    }
    
    pub async fn make_decision(&self, situation: &GovernanceSituation) -> GovernanceDecision {
        match situation {
            GovernanceSituation::ActorUnhealthy { actor, error_count } => {
                if *error_count >= self.thresholds.max_errors_before_restart {
                    GovernanceDecision::RestartActor {
                        actor: *actor,
                        strategy: super::RecoveryStrategy::OneForOne,
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::MultipleActorsUnhealthy { count } => {
                if *count >= self.thresholds.max_unhealthy_actors {
                    GovernanceDecision::RestartAll
                } else {
                    GovernanceDecision::NotifyStakeholders {
                        message: format!("{} actors are unhealthy", count),
                    }
                }
            }
            GovernanceSituation::SecurityBreach { severity } => {
                if *severity == SecuritySeverity::Critical {
                    GovernanceDecision::EmergencyShutdown {
                        reason: "Critical security breach".to_string(),
                    }
                } else {
                    GovernanceDecision::NotifyStakeholders {
                        message: format!("Security breach detected: {:?}", severity),
                    }
                }
            }
            GovernanceSituation::SystemHealthy => GovernanceDecision::NoAction,
            _ => GovernanceDecision::NoAction,
        }
    }
    
    pub async fn record_decision(&self, situation: GovernanceSituation, decision: GovernanceDecision, actor: Option<super::GodName>) {
        let mut decisions = self.decisions.write().await;
        decisions.push(GovernanceRecord {
            timestamp: chrono::Utc::now(),
            situation,
            decision,
            actor,
        });
        
        // Keep only last 1000 decisions
        if decisions.len() > 1000 {
            decisions.remove(0);
        }
    }
    
    pub async fn get_history(&self, limit: usize) -> Vec<GovernanceRecord> {
        let decisions = self.decisions.read().await;
        decisions.iter().rev().take(limit).cloned().collect()
    }
}
