// src/actors/ares/history.rs
// OLYMPUS v15 - History: Sistema de Historial de Conflictos para Ares

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::ares::{strategies::ResolutionResult, Conflict, ConflictSeverity, ConflictType};
use crate::actors::GodName;
use crate::errors::ActorError;

/// Historial de conflictos para Ares
#[derive(Debug, Clone)]
pub struct ConflictHistory {
    entries: Vec<HistoryEntry>,
    stats: ConflictStats,
    patterns: Vec<ConflictPattern>,
}

/// Entrada en el historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub conflict: Conflict,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub escalation_count: u32,
    pub resolution_result: Option<ResolutionResult>,
}

/// Estadísticas de conflictos
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConflictStats {
    pub total_detected: u32,
    pub total_resolved: u32,
    pub successful_resolutions: u32,
    pub failed_resolutions: u32,
    pub current_active: u32,
    pub total_escalated: u32,
    pub average_resolution_time_ms: f64,
    pub by_type: HashMap<ConflictType, TypeStats>,
    pub by_severity: HashMap<ConflictSeverity, SeverityStats>,
    pub by_strategy: HashMap<String, StrategyStats>,
    pub patterns: Vec<ConflictPattern>,
}

/// Estadísticas por tipo
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: u32,
    pub resolved_count: u32,
}

/// Estadísticas por severidad
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityStats {
    pub count: u32,
    pub escalation_rate: f64,
    pub resolution_rate: f64,
}

/// Estadísticas por estrategia
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyStats {
    pub usage_count: u32,
    pub success_rate: f64,
    pub average_resolution_time_ms: f64,
}

/// Patrón de conflicto detectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPattern {
    pub description: String,
    pub frequency: u32,
    pub last_seen: DateTime<Utc>,
    pub affected_actors: Vec<String>,
    pub suggested_action: String,
}

impl ConflictHistory {
    /// Crea un nuevo historial
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            stats: ConflictStats {
                total_detected: 0,
                total_resolved: 0,
                successful_resolutions: 0,
                failed_resolutions: 0,
                current_active: 0,
                total_escalated: 0,
                average_resolution_time_ms: 0.0,
                by_type: HashMap::new(),
                by_severity: HashMap::new(),
                by_strategy: HashMap::new(),
                patterns: Vec::new(),
            },
            patterns: Vec::new(),
        }
    }

    /// Agrega un nuevo conflicto al historial
    pub fn add_conflict(&mut self, conflict: Conflict) {
        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            conflict: conflict.clone(),
            created_at: Utc::now(),
            resolved_at: None,
            escalation_count: 0,
            resolution_result: None,
        };

        self.entries.push(entry);
        self.update_stats();
    }

    /// Marca un conflicto como resuelto
    pub fn mark_resolved(
        &mut self,
        conflict_id: &str,
        result: ResolutionResult,
    ) -> Result<(), ActorError> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == conflict_id) {
            entry.resolved_at = Some(Utc::now());
            entry.resolution_result = Some(result.clone());
            self.update_stats();
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Ares,
                message: format!("Conflicto no encontrado: {}", conflict_id),
            })
        }
    }

    /// Incrementa contador de escalación
    pub fn increment_escalation(&mut self, conflict_id: &str) -> Result<(), ActorError> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == conflict_id) {
            entry.escalation_count += 1;
            self.update_stats();
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Ares,
                message: format!("Conflicto no encontrado: {}", conflict_id),
            })
        }
    }

    /// Actualiza estadísticas
    fn update_stats(&mut self) {
        // Resetear estadísticas
        self.stats.total_detected = self.entries.len() as u32;
        self.stats.total_resolved = 0;
        self.stats.successful_resolutions = 0;
        self.stats.failed_resolutions = 0;
        self.stats.current_active = 0;
        self.stats.total_escalated = 0;

        let mut resolution_times = Vec::new();

        for entry in &self.entries {
            if entry.resolved_at.is_some() {
                self.stats.total_resolved += 1;

                if let Some(ref result) = entry.resolution_result {
                    if result.success {
                        self.stats.successful_resolutions += 1;
                    } else {
                        self.stats.failed_resolutions += 1;
                    }
                    resolution_times.push(result.resolution_time_ms);
                }
            } else {
                self.stats.current_active += 1;
            }

            if entry.escalation_count > 0 {
                self.stats.total_escalated += 1;
            }
        }

        // Calcular tiempo promedio
        if !resolution_times.is_empty() {
            self.stats.average_resolution_time_ms =
                resolution_times.iter().sum::<u64>() as f64 / resolution_times.len() as f64;
        }

        self.detect_patterns();
    }

    /// Detecta patrones en los conflictos
    fn detect_patterns(&mut self) {
        self.patterns.clear();

        // Simple detección de patrones por recurso
        let mut resource_counts: HashMap<String, u32> = HashMap::new();

        for entry in &self.entries {
            let resource = format!("Resource: {}", entry.conflict.resource);
            *resource_counts.entry(resource).or_insert(0) += 1;
        }

        for (resource, count) in resource_counts {
            if count >= 3 {
                self.patterns.push(ConflictPattern {
                    description: resource.clone(),
                    frequency: count,
                    last_seen: Utc::now(),
                    affected_actors: vec![],
                    suggested_action: "Considerar particionar este recurso".to_string(),
                });
            }
        }

        self.stats.patterns = self.patterns.clone();
    }

    /// Obtiene conflictos recientes
    pub fn get_recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    /// Obtiene estadísticas
    pub fn get_stats(&self) -> &ConflictStats {
        &self.stats
    }

    /// Exporta historial a JSON
    pub fn export_to_json(&self) -> Result<String, ActorError> {
        serde_json::to_string_pretty(&self.entries).map_err(|e| ActorError::Unknown {
            god: GodName::Ares,
            message: format!("Error exportando historial: {}", e),
        })
    }

    /// Importa historial desde JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<(), ActorError> {
        let imported: Vec<HistoryEntry> =
            serde_json::from_str(json).map_err(|e| ActorError::Unknown {
                god: GodName::Ares,
                message: format!("Error importando historial: {}", e),
            })?;

        for entry in imported {
            self.entries.push(entry);
        }

        self.update_stats();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_conflict() {
        let mut history = ConflictHistory::new();

        let conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );

        history.add_conflict(conflict);

        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.stats.total_detected, 1);
        assert_eq!(history.stats.current_active, 1);
    }

    #[test]
    fn test_mark_resolved() {
        let mut history = ConflictHistory::new();

        let conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );

        history.add_conflict(conflict.clone());
        let entry_id = history.entries[0].id.clone();

        let result = ResolutionResult::success(
            crate::actors::ares::strategies::ResolutionStrategy::Priority,
            "Test resolution".to_string(),
        );

        assert!(history.mark_resolved(&entry_id, result).is_ok());
        assert_eq!(history.stats.total_resolved, 1);
        assert_eq!(history.stats.successful_resolutions, 1);
        assert_eq!(history.stats.current_active, 0);
    }
}
