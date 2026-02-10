// src/actors/moirai/fate.rs
// OLYMPUS v15 - Motor del Destino y Probabilidades

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::actors::moirai::threads::{PatientThread, FateOutcome, ThreadStatus};

/// Motor del destino
#[derive(Debug, Clone)]
pub struct FateEngine;

impl FateEngine {
    pub fn new() -> Self {
        Self
    }

    /// Encuentra casos similares históricos
    pub async fn find_similar_cases(&self, thread: &PatientThread, limit: usize) -> Result<Vec<String>, crate::errors::ActorError> {
        // Simulación: retornar IDs de pacientes similares basados en scores
        let mut similar = Vec::new();
        
        // Aquí iría la lógica real de búsqueda de similitud
        // Por ahora simulamos
        for i in 0..limit {
            similar.push(format!("similar_patient_{}", i));
        }
        
        Ok(similar)
    }

    /// Calcula probabilidad de un outcome específico
    pub fn calculate_outcome_probability(&self, thread: &PatientThread, outcome: FateOutcome) -> f64 {
        match outcome {
            FateOutcome::Heroic => {
                // Alta probabilidad si está estable y mejorando
                if thread.status == ThreadStatus::Stable {
                    0.7
                } else {
                    0.3
                }
            }
            FateOutcome::Tragic => {
                // Basado en scores
                if thread.status == ThreadStatus::Critical {
                    0.6
                } else {
                    0.1
                }
            }
            _ => 0.5,
        }
    }
}

impl Default for FateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculadora de probabilidades
#[derive(Debug, Clone)]
pub struct ProbabilityCalculator;

impl ProbabilityCalculator {
    /// Calcula probabilidad bayesiana
    pub fn bayesian_probability(prior: f64, likelihood: f64, evidence: f64) -> f64 {
        if evidence == 0.0 {
            return prior;
        }
        (likelihood * prior) / evidence
    }

    /// Calcula odds ratio
    pub fn odds_ratio(probability: f64) -> f64 {
        if probability >= 1.0 {
            return f64::INFINITY;
        }
        probability / (1.0 - probability)
    }

    /// Combina múltiples probabilidades
    pub fn combine_probabilities(probabilities: &[f64]) -> f64 {
        if probabilities.is_empty() {
            return 0.0;
        }
        
        let product: f64 = probabilities.iter().product();
        let inverse_product: f64 = probabilities.iter()
            .map(|p| 1.0 - p)
            .product();
        
        product / (product + inverse_product)
    }
}

/// Predicción de destino
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatePrediction {
    pub timestamp: DateTime<Utc>,
    pub predicted_outcome: FateOutcome,
    pub probability: f64,
    pub confidence: f64,
    pub factors: Vec<String>,
}
