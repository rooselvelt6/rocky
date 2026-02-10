// src/actors/ares/history.rs
// OLYMPUS v15 - Historial de Conflictos para Ares

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::detector::ConflictStatus;
use super::strategies::{ResolutionResult, ResolutionStrategy};
use super::{Conflict, ConflictSeverity, ConflictType};

/// Historial de conflictos
#[derive(Debug, Clone)]
pub struct ConflictHistory {
    /// Entradas de historial (limitado a MAX_HISTORY_SIZE)
    entries: VecDeque<HistoryEntry>,
    /// Tamaño máximo del historial
    max_size: usize,
    /// Estadísticas agregadas
    stats: ConflictStats,
}

/// Entrada de historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Conflicto original
    pub conflict: Conflict,
    /// Resultado de resolución (si aplica)
    pub resolution_result: Option<ResolutionResult>,
    /// Timestamp de creación de la entrada
    pub created_at: DateTime<Utc>,
    /// Categoría para análisis
    pub category: HistoryCategory,
}

/// Categorías para análisis del historial
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryCategory {
    /// Resolución exitosa rápida
    QuickResolution,
    /// Resolución exitosa después de escalado
    EscalatedResolution,
    /// Resolución fallida
    FailedResolution,
    /// Resuelto automáticamente
    AutoResolution,
    /// Requirió intervención manual
    ManualIntervention,
    /// Aún activo
    StillActive,
}

/// Estadísticas de conflictos
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConflictStats {
    /// Total de conflictos detectados
    pub total_detected: u64,
    /// Conflictos actualmente activos
    pub current_active: u64,
    /// Total de conflictos resueltos
    pub total_resolved: u64,
    /// Resoluciones exitosas
    pub successful_resolutions: u64,
    /// Resoluciones fallidas
    pub failed_resolutions: u64,
    /// Conflictos escalados
    pub total_escalated: u64,
    /// Tiempo promedio de resolución (ms)
    pub average_resolution_time_ms: f64,
    /// Estadísticas por tipo
    pub by_type: HashMap<ConflictType, TypeStats>,
    /// Estadísticas por severidad
    pub by_severity: HashMap<ConflictSeverity, SeverityStats>,
    /// Estadísticas por estrategia
    pub by_strategy: HashMap<ResolutionStrategy, StrategyStats>,
    /// Patrones detectados
    pub patterns: Vec<ConflictPattern>,
}

/// Estadísticas por tipo de conflicto
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: u64,
    pub resolved_count: u64,
    pub average_resolution_time_ms: f64,
    pub most_common_strategy: Option<ResolutionStrategy>,
}

/// Estadísticas por severidad
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityStats {
    pub count: u64,
    pub escalation_rate: f64,
    pub resolution_rate: f64,
}

/// Estadísticas por estrategia
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyStats {
    pub usage_count: u64,
    pub success_rate: f64,
    pub average_resolution_time_ms: f64,
}

/// Patrones de conflictos detectados
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
            entries: VecDeque::new(),
            max_size: 1000, // Máximo 1000 entradas
            stats: ConflictStats::default(),
        }
    }

    /// Crea un historial con tamaño personalizado
    pub fn with_size(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
            stats: ConflictStats::default(),
        }
    }

    /// Agrega un conflicto resuelto al historial
    pub fn add_resolved(&mut self, conflict: Conflict, result: ResolutionResult) {
        let category = self.categorize_entry(&conflict, Some(&result));
        let entry = HistoryEntry {
            conflict,
            resolution_result: Some(result),
            created_at: Utc::now(),
            category,
        };

        self.add_entry(entry);
    }

    /// Agrega un conflicto activo al historial
    pub fn add_active(&mut self, conflict: Conflict) {
        let category = HistoryCategory::StillActive;
        let entry = HistoryEntry {
            conflict,
            resolution_result: None,
            created_at: Utc::now(),
            category,
        };

        self.add_entry(entry);
    }

    /// Agrega una entrada al historial
    fn add_entry(&mut self, entry: HistoryEntry) {
        // Si el historial está lleno, remover la entrada más antigua
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
        
        // Actualizar estadísticas incrementalmente (no resetear todo)
        self.stats.total_detected += 1;
        
        if entry.conflict.is_resolved() {
            self.stats.total_resolved += 1;
        } else {
            self.stats.current_active += 1;
        }
        
        if let Some(ref result) = entry.resolution_result {
            if result.success {
                self.stats.successful_resolutions += 1;
            } else {
                self.stats.failed_resolutions += 1;
            }
        }
        
        if entry.conflict.is_escalated() {
            self.stats.total_escalated += 1;
        }
    }

    /// Categoriza una entrada de historial
    fn categorize_entry(
        &self,
        conflict: &Conflict,
        result: Option<&ResolutionResult>,
    ) -> HistoryCategory {
        match result {
            Some(res) if res.success => {
                if res.resolution_time_ms < 1000 {
                    HistoryCategory::QuickResolution
                } else if conflict.is_escalated() {
                    HistoryCategory::EscalatedResolution
                } else {
                    HistoryCategory::AutoResolution
                }
            }
            Some(_) => HistoryCategory::FailedResolution,
            None => {
                if conflict.is_escalated() {
                    HistoryCategory::ManualIntervention
                } else {
                    HistoryCategory::StillActive
                }
            }
        }
    }

    /// Actualiza estadísticas agregadas
    fn update_stats(&mut self) {
        // Resetear estadísticas base pero mantener HashMaps
        self.stats.total_detected = 0;
        self.stats.total_resolved = 0;
        self.stats.successful_resolutions = 0;
        self.stats.failed_resolutions = 0;
        self.stats.current_active = 0;
        self.stats.total_escalated = 0;
        self.stats.average_resolution_time_ms = 0.0;
        self.stats.by_type.clear();
        self.stats.by_severity.clear();
        self.stats.by_strategy.clear();
        self.stats.patterns.clear();
        
        let mut resolution_times = Vec::new();
        
        for entry in &self.entries {
            // Estadísticas generales
            self.stats.total_detected += 1;
            
            if entry.conflict.is_resolved() {
                self.stats.total_resolved += 1;
            } else {
                self.stats.current_active += 1;
            }
            
            if let Some(ref result) = entry.resolution_result {
                if result.success {
                    self.stats.successful_resolutions += 1;
                } else {
                    self.stats.failed_resolutions += 1;
                }
                resolution_times.push(result.resolution_time_ms);
            }
            
            if entry.conflict.is_escalated() {
                self.stats.total_escalated += 1;
            }
            
            // Estadísticas por tipo
            let type_stats = self.stats.by_type
                .entry(entry.conflict.conflict_type.clone())
                .or_default();
            type_stats.count += 1;
            if entry.conflict.is_resolved() {
                type_stats.resolved_count += 1;
            }
            
            // Estadísticas por severidad
            let severity_stats = self.stats.by_severity
                .entry(entry.conflict.severity.clone())
                .or_default();
            severity_stats.count += 1;
            if entry.conflict.is_escalated() {
                severity_stats.escalation_rate += 1.0;
            }
            if entry.conflict.is_resolved() {
                severity_stats.resolution_rate += 1.0;
            }
            
            // Estadísticas por estrategia
            if let Some(ref result) = entry.resolution_result {
                let strategy_stats = self.stats.by_strategy
                    .entry(result.strategy.clone())
                    .or_default();
                strategy_stats.usage_count += 1;
                if result.success {
                    strategy_stats.success_rate += 1.0;
                }
                strategy_stats.average_resolution_time_ms += result.resolution_time_ms as f64;
            }
        }

            if let Some(ref result) = entry.resolution_result {
                if result.success {
                    self.stats.successful_resolutions += 1;
                } else {
                    self.stats.failed_resolutions += 1;
                }
                resolution_times.push(result.resolution_time_ms);
            }

            if entry.conflict.is_escalated() {
                self.stats.total_escalated += 1;
            }

            // Estadísticas por tipo
            let type_stats = self
                .stats
                .by_type
                .entry(entry.conflict.conflict_type.clone())
                .or_default();
            type_stats.count += 1;

            if entry.conflict.is_resolved() {
                type_stats.resolved_count += 1;
            }

            // Estadísticas por severidad
            let severity_stats = self
                .stats
                .by_severity
                .entry(entry.conflict.severity.clone())
                .or_default();
            severity_stats.count += 1;

            if entry.conflict.is_escalated() {
                severity_stats.escalation_rate += 1.0;
            }

            if entry.conflict.is_resolved() {
                severity_stats.resolution_rate += 1.0;
            }

            // Estadísticas por estrategia
            if let Some(ref result) = entry.resolution_result {
                let strategy_stats = self
                    .stats
                    .by_strategy
                    .entry(result.strategy.clone())
                    .or_default();
                strategy_stats.usage_count += 1;

                if result.success {
                    strategy_stats.success_rate += 1.0;
                }

                strategy_stats.average_resolution_time_ms += result.resolution_time_ms as f64;
            }
        }

        // Calcular promedios
        if !resolution_times.is_empty() {
            self.stats.average_resolution_time_ms =
                resolution_times.iter().sum::<u64>() as f64 / resolution_times.len() as f64;
        }

        // Normalizar tasas
        for (_, severity_stats) in &mut self.stats.by_severity {
            if severity_stats.count > 0 {
                severity_stats.escalation_rate =
                    (severity_stats.escalation_rate / severity_stats.count as f64) * 100.0;
                severity_stats.resolution_rate =
                    (severity_stats.resolution_rate / severity_stats.count as f64) * 100.0;
            }
        }

        for (_, strategy_stats) in &mut self.stats.by_strategy {
            if strategy_stats.usage_count > 0 {
                strategy_stats.success_rate =
                    (strategy_stats.success_rate / strategy_stats.usage_count as f64) * 100.0;
                strategy_stats.average_resolution_time_ms /= strategy_stats.usage_count as f64;
            }
        }

        // Detectar patrones
        self.detect_patterns();
    }

    /// Detecta patrones en los conflictos
    fn detect_patterns(&mut self) {
        let mut patterns: HashMap<String, (u32, DateTime<Utc>, Vec<String>)> = HashMap::new();

        for entry in &self.entries {
            // Buscar patrones por recurso
            let resource_pattern = format!("Resource: {}", entry.conflict.resource);
            let entry_data = patterns.entry(resource_pattern.clone()).or_insert((
                0,
                entry.created_at,
                Vec::new(),
            ));

            entry_data.0 += 1;
            if entry.created_at > entry_data.1 {
                entry_data.1 = entry.created_at;
            }

            if !entry_data
                .2
                .contains(&format!("{:?}", entry.conflict.actors.0))
            {
                entry_data.2.push(format!("{:?}", entry.conflict.actors.0));
            }
            if !entry_data
                .2
                .contains(&format!("{:?}", entry.conflict.actors.1))
            {
                entry_data.2.push(format!("{:?}", entry.conflict.actors.1));
            }

            // Buscar patrones por tipo
            let type_pattern = format!("Type: {:?}", entry.conflict.conflict_type);
            let entry_data =
                patterns
                    .entry(type_pattern.clone())
                    .or_insert((0, entry.created_at, Vec::new()));

            entry_data.0 += 1;
            if entry.created_at > entry_data.1 {
                entry_data.1 = entry.created_at;
            }
        }

        // Convertir patrones a ConflictPattern
        self.stats.patterns = patterns
            .into_iter()
            .filter(|(_, (freq, _, actors))| *freq >= 3 && actors.len() >= 2)
            .map(|(description, (frequency, last_seen, affected_actors))| {
                let suggested_action = if description.contains("Resource") {
                    "Considerar particionar o replicar este recurso".to_string()
                } else {
                    "Investigar causa raíz de este tipo de conflicto".to_string()
                };

                ConflictPattern {
                    description,
                    frequency,
                    last_seen,
                    affected_actors,
                    suggested_action,
                }
            })
            .collect();
    }

    /// Obtiene los conflictos más recientes
    pub fn get_recent(&self, limit: usize) -> Vec<Conflict> {
        self.entries
            .iter()
            .rev()
            .take(limit)
            .map(|entry| entry.conflict.clone())
            .collect()
    }

    /// Filtra conflictos por período
    pub fn filter_by_period(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.created_at >= from && entry.created_at <= to)
            .collect()
    }

    /// Filtra conflictos por tipo
    pub fn filter_by_type(&self, conflict_type: &ConflictType) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.conflict.conflict_type == *conflict_type)
            .collect()
    }

    /// Filtra conflictos por severidad
    pub fn filter_by_severity(&self, severity: &ConflictSeverity) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.conflict.severity == *severity)
            .collect()
    }

    /// Obtiene conflictos que requieren atención
    pub fn get_attention_required(&self) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                match entry.category {
                    HistoryCategory::FailedResolution => true,
                    HistoryCategory::ManualIntervention => true,
                    HistoryCategory::StillActive => {
                        // Si está activo por más de 5 minutos
                        Utc::now() - entry.created_at > Duration::minutes(5)
                    }
                    _ => false,
                }
            })
            .collect()
    }

    /// Obtiene estadísticas actuales
    pub fn get_stats(&self) -> &ConflictStats {
        &self.stats
    }

    /// Limpia entradas antiguas
    pub fn cleanup_old(&mut self, older_than: Duration) {
        let cutoff = Utc::now() - older_than;

        self.entries.retain(|entry| entry.created_at >= cutoff);
        self.update_stats();
    }

    /// Exporta historial a formato JSON
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }

    /// Importa historial desde JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let imported: Vec<HistoryEntry> = serde_json::from_str(json)?;

        for entry in imported {
            self.add_entry(entry);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::ares::detector::ConflictType;
    use crate::actors::GodName;

    #[test]
    fn test_history_creation() {
        let history = ConflictHistory::new();
        assert_eq!(history.max_size, 1000);
        assert!(history.entries.is_empty());
    }

    #[test]
    fn test_add_resolved_conflict() {
        let mut history = ConflictHistory::new();

        let conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );

        let result = super::super::strategies::ResolutionResult::success(
            super::super::strategies::ResolutionStrategy::Priority,
            "Test resolution".to_string(),
        );

        history.add_resolved(conflict, result);

        assert_eq!(history.entries.len(), 1);
        assert!(history.entries[0].resolution_result.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut history = ConflictHistory::new();

        let conflict = Conflict::new(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        );

        let result = super::super::strategies::ResolutionResult::success(
            super::super::strategies::ResolutionStrategy::Priority,
            "Test resolution".to_string(),
        );

        history.add_resolved(conflict, result);

        let stats = history.get_stats();
        assert_eq!(stats.total_detected, 1);
        assert_eq!(stats.total_resolved, 1);
        assert_eq!(stats.successful_resolutions, 1);
    }

    #[test]
    fn test_filter_by_type() {
        let mut history = ConflictHistory::new();

        let conflict1 = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );

        let conflict2 = Conflict::new(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        );

        let result = super::super::strategies::ResolutionResult::success(
            super::super::strategies::ResolutionStrategy::Priority,
            "Test resolution".to_string(),
        );

        history.add_resolved(conflict1, result.clone());
        history.add_resolved(conflict2, result);

        let resource_conflicts = history.filter_by_type(&ConflictType::Resource);
        assert_eq!(resource_conflicts.len(), 1);

        let data_conflicts = history.filter_by_type(&ConflictType::Data);
        assert_eq!(data_conflicts.len(), 1);
    }

    #[test]
    fn test_recent_conflicts() {
        let mut history = ConflictHistory::new();

        for i in 0..5 {
            let conflict = Conflict::new(
                GodName::Zeus,
                GodName::Hades,
                &format!("resource_{}", i),
                ConflictType::Resource,
            );

            let result = super::super::strategies::ResolutionResult::success(
                super::super::strategies::ResolutionStrategy::Priority,
                "Test resolution".to_string(),
            );

            history.add_resolved(conflict, result);
        }

        let recent = history.get_recent(3);
        assert_eq!(recent.len(), 3);
    }
}
