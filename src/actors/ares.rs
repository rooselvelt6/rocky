use chrono::{DateTime, Utc};
/// Ares v12 - Dios de la Guerra
/// Gestión de conflictos y resolución de problemas
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: String,
    pub involved_parties: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub status: ConflictStatus,
    pub resolution_strategy: Option<ResolutionStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    DataConflict,
    ResourceContention,
    AccessConflict,
    ClinicalDisagreement,
    SystemDeadlock,
    ConcurrentModification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStatus {
    Pending,
    InProgress,
    Resolved,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    FirstComeFirstServed,
    LastWriterWins,
    MergeResolution,
    ManualIntervention,
    OptimisticLocking,
    PessimisticLocking,
    TimestampOrdering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub strategy: ResolutionStrategy,
    pub resolution: String,
    pub resolved_by: String,
    pub resolved_at: DateTime<Utc>,
    pub outcome: ResolutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionOutcome {
    Successful,
    Partial,
    Failed,
    Escalated,
}

#[derive(Debug, Clone)]
pub struct AresV12 {
    active_conflicts: HashMap<String, Conflict>,
    resolution_history: Vec<ConflictResolution>,
    strategies: HashMap<ConflictType, ResolutionStrategy>,
}

impl AresV12 {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        // Estrategias por defecto por tipo de conflicto
        strategies.insert(
            ConflictType::DataConflict,
            ResolutionStrategy::MergeResolution,
        );
        strategies.insert(
            ConflictType::ResourceContention,
            ResolutionStrategy::FirstComeFirstServed,
        );
        strategies.insert(
            ConflictType::AccessConflict,
            ResolutionStrategy::TimestampOrdering,
        );
        strategies.insert(
            ConflictType::ClinicalDisagreement,
            ResolutionStrategy::ManualIntervention,
        );
        strategies.insert(
            ConflictType::SystemDeadlock,
            ResolutionStrategy::OptimisticLocking,
        );
        strategies.insert(
            ConflictType::ConcurrentModification,
            ResolutionStrategy::LastWriterWins,
        );

        Self {
            active_conflicts: HashMap::new(),
            resolution_history: Vec::new(),
            strategies,
        }
    }

    pub fn detect_conflict(
        &mut self,
        conflict_type: ConflictType,
        description: &str,
        involved_parties: Vec<String>,
    ) -> String {
        let conflict_id = uuid::Uuid::new_v4().to_string();

        let severity = self.assess_conflict_severity(&conflict_type, &involved_parties);

        let conflict = Conflict {
            id: conflict_id.clone(),
            conflict_type,
            severity,
            description: description.to_string(),
            involved_parties,
            timestamp: Utc::now(),
            status: ConflictStatus::Pending,
            resolution_strategy: self.strategies.get(&conflict_type).cloned(),
        };

        self.active_conflicts.insert(conflict_id.clone(), conflict);

        tracing::warn!(
            "⚔️ Ares: Conflicto detectado - {:?} - {}",
            conflict_type,
            description
        );

        conflict_id
    }

    pub fn resolve_conflict(
        &mut self,
        conflict_id: &str,
        resolution: &str,
        resolved_by: &str,
    ) -> Result<(), String> {
        if let Some(mut conflict) = self.active_conflicts.remove(conflict_id) {
            let strategy = conflict
                .resolution_strategy
                .clone()
                .unwrap_or(ResolutionStrategy::ManualIntervention);
            let outcome = self.evaluate_resolution(&conflict, resolution);

            let conflict_resolution = ConflictResolution {
                conflict_id: conflict_id.to_string(),
                strategy,
                resolution: resolution.to_string(),
                resolved_by: resolved_by.to_string(),
                resolved_at: Utc::now(),
                outcome,
            };

            self.resolution_history.push(conflict_resolution);

            tracing::info!(
                "⚔️ Ares: Conflicto {} resuelto usando {:?}",
                conflict_id,
                strategy
            );
            Ok(())
        } else {
            Err(format!("Conflicto {} no encontrado", conflict_id))
        }
    }

    pub fn escalate_conflict(&mut self, conflict_id: &str, reason: &str) -> Result<(), String> {
        if let Some(mut conflict) = self.active_conflicts.get_mut(conflict_id) {
            conflict.status = ConflictStatus::Escalated;

            tracing::error!("⚔️ Ares: Conflicto {} escalado - {}", conflict_id, reason);
            Ok(())
        } else {
            Err(format!("Conflicto {} no encontrado", conflict_id))
        }
    }

    pub fn get_conflict(&self, conflict_id: &str) -> Option<&Conflict> {
        self.active_conflicts.get(conflict_id)
    }

    pub fn get_active_conflicts(&self, conflict_type: Option<ConflictType>) -> Vec<&Conflict> {
        let mut conflicts: Vec<&Conflict> = self.active_conflicts.values().collect();

        if let Some(filter_type) = conflict_type {
            conflicts.retain(|c| c.conflict_type == filter_type);
        }

        // Ordenar por severidad y timestamp
        conflicts.sort_by(|a, b| match (a.severity.clone(), b.severity.clone()) {
            (ConflictSeverity::Critical, _) => std::cmp::Ordering::Less,
            (_, ConflictSeverity::Critical) => std::cmp::Ordering::Greater,
            (ConflictSeverity::High, _) => std::cmp::Ordering::Less,
            (_, ConflictSeverity::High) => std::cmp::Ordering::Greater,
            (ConflictSeverity::Medium, _) => std::cmp::Ordering::Less,
            (_, ConflictSeverity::Medium) => std::cmp::Ordering::Greater,
            (ConflictSeverity::Low, _) => std::cmp::Ordering::Less,
            (_, ConflictSeverity::Low) => std::cmp::Ordering::Greater,
            (ConflictSeverity::Low, ConflictSeverity::Low) => b.timestamp.cmp(&a.timestamp),
        });

        conflicts
    }

    pub fn auto_resolve_conflicts(&mut self) -> Vec<String> {
        let mut resolved = Vec::new();
        let conflicts_to_resolve: Vec<String> = self
            .active_conflicts
            .iter()
            .filter(|(_, c)| {
                matches!(
                    c.resolution_strategy,
                    Some(
                        ResolutionStrategy::FirstComeFirstServed
                            | ResolutionStrategy::LastWriterWins
                    )
                )
            })
            .map(|(id, _)| id.clone())
            .collect();

        for conflict_id in conflicts_to_resolve {
            if let Some(conflict) = self.active_conflicts.get(&conflict_id) {
                let resolution = match conflict.resolution_strategy.as_ref().unwrap() {
                    ResolutionStrategy::FirstComeFirstServed => "Primera llegada gana".to_string(),
                    ResolutionStrategy::LastWriterWins => "Última escritura gana".to_string(),
                    _ => "Resolución manual requerida".to_string(),
                };

                if self
                    .resolve_conflict(&conflict_id, &resolution, "Ares-Auto")
                    .is_ok()
                {
                    resolved.push(conflict_id);
                }
            }
        }

        resolved
    }

    fn assess_conflict_severity(
        &self,
        conflict_type: &ConflictType,
        involved_parties: &[String],
    ) -> ConflictSeverity {
        match conflict_type {
            ConflictType::SystemDeadlock => ConflictSeverity::Critical,
            ConflictType::DataConflict => {
                if involved_parties.len() > 3 {
                    ConflictSeverity::High
                } else {
                    ConflictSeverity::Medium
                }
            }
            ConflictType::AccessConflict => {
                if involved_parties.len() > 2 {
                    ConflictSeverity::High
                } else {
                    ConflictSeverity::Medium
                }
            }
            ConflictType::ClinicalDisagreement => ConflictSeverity::High,
            ConflictType::ResourceContention => {
                if involved_parties.len() > 5 {
                    ConflictSeverity::High
                } else {
                    ConflictSeverity::Low
                }
            }
            ConflictType::ConcurrentModification => ConflictSeverity::Medium,
        }
    }

    fn evaluate_resolution(&self, conflict: &Conflict, resolution: &str) -> ResolutionOutcome {
        match conflict.conflict_type {
            ConflictType::DataConflict => {
                if resolution.contains("merge") {
                    ResolutionOutcome::Successful
                } else if resolution.contains("manual") {
                    ResolutionOutcome::Partial
                } else {
                    ResolutionOutcome::Failed
                }
            }
            ConflictType::AccessConflict => {
                if resolution.contains("granted") || resolution.contains("denied") {
                    ResolutionOutcome::Successful
                } else {
                    ResolutionOutcome::Failed
                }
            }
            ConflictType::ClinicalDisagreement => {
                if resolution.contains("consensus") || resolution.contains("escalated") {
                    ResolutionOutcome::Successful
                } else {
                    ResolutionOutcome::Partial
                }
            }
            _ => {
                if !resolution.is_empty() {
                    ResolutionOutcome::Successful
                } else {
                    ResolutionOutcome::Failed
                }
            }
        }
    }

    pub fn get_conflict_statistics(&self) -> ConflictStatistics {
        let total_conflicts = self.active_conflicts.len();
        let mut type_counts = HashMap::new();
        let mut severity_counts = HashMap::new();
        let mut status_counts = HashMap::new();

        for conflict in self.active_conflicts.values() {
            *type_counts
                .entry(format!("{:?}", conflict.conflict_type))
                .or_insert(0) += 1;
            *severity_counts
                .entry(format!("{:?}", conflict.severity))
                .or_insert(0) += 1;
            *status_counts
                .entry(format!("{:?}", conflict.status))
                .or_insert(0) += 1;
        }

        ConflictStatistics {
            total_active_conflicts: total_conflicts,
            total_resolved: self.resolution_history.len(),
            conflict_type_counts: type_counts,
            severity_counts,
            status_counts,
            most_common_strategy: self.get_most_common_strategy(),
        }
    }

    fn get_most_common_strategy(&self) -> ResolutionStrategy {
        let mut strategy_counts = HashMap::new();

        for resolution in &self.resolution_history {
            *strategy_counts
                .entry(format!("{:?}", resolution.strategy))
                .or_insert(0) += 1;
        }

        strategy_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(strategy, _)| strategy)
            .unwrap_or(ResolutionStrategy::ManualIntervention)
    }

    pub fn set_strategy(&mut self, conflict_type: ConflictType, strategy: ResolutionStrategy) {
        self.strategies.insert(conflict_type, strategy);
        tracing::info!(
            "⚔️ Ares: Estrategia actualizada para {:?}: {:?}",
            conflict_type,
            strategy
        );
    }

    pub fn clear_resolved_conflicts(&mut self, older_than_hours: u64) {
        let cutoff_time = Utc::now() - chrono::Duration::hours(older_than_hours as i64);

        let initial_count = self.resolution_history.len();
        self.resolution_history
            .retain(|r| r.resolved_at > cutoff_time);

        let removed_count = initial_count - self.resolution_history.len();
        if removed_count > 0 {
            tracing::info!("⚔️ Ares: {} resoluciones antiguas limpiadas", removed_count);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConflictStatistics {
    pub total_active_conflicts: usize,
    pub total_resolved: usize,
    pub conflict_type_counts: HashMap<String, u32>,
    pub severity_counts: HashMap<String, u32>,
    pub status_counts: HashMap<String, u32>,
    pub most_common_strategy: ResolutionStrategy,
}

impl Default for AresV12 {
    fn default() -> Self {
        Self::new()
    }
}
