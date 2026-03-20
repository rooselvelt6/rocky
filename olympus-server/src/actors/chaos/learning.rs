// src/actors/chaos/learning.rs
// OLYMPUS v16 - Chaos Learning Module

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChaosLearner {
    experiment_history: Arc<RwLock<Vec<LearntExperiment>>>,
    strategy_knowledge: Arc<RwLock<HashMap<String, StrategyKnowledge>>>,
    insights: Arc<RwLock<Vec<ChaosInsight>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub min_experiments_for_insights: usize,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct LearntExperiment {
    pub id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct StrategyKnowledge {
    pub success_rate: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub pattern_id: String,
}

#[derive(Debug, Clone)]
pub struct ChaosInsight {
    pub insight_id: String,
    pub description: String,
}

impl ChaosLearner {
    pub fn new() -> Self {
        Self {
            experiment_history: Arc::new(RwLock::new(Vec::new())),
            strategy_knowledge: Arc::new(RwLock::new(HashMap::new())),
            insights: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
