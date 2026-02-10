// src/actors/ares/detector.rs
// OLYMPUS v15 - Detector de Conflictos para Ares

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::GodName;
use crate::errors::ActorError;

/// Tipos de conflictos detectables
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    /// Conflicto por acceso a recursos compartidos
    Resource,
    /// Conflicto por acceso a datos
    Data,
    /// Conflicto por prioridades encontradas
    Priority,
    /// Conflicto por dependencias circulares
    Dependency,
    /// Conflicto por tiempo de ejecución
    Timing,
    /// Conflicto por comunicación
    Communication,
    /// Conflicto por estado inconsistente
    State,
}

/// Severidad del conflicto
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Estado de un conflicto
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStatus {
    Detected,
    Active,
    Resolving,
    Resolved,
    Escalated,
    Failed,
}

/// Conflicto detectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// ID único del conflicto
    pub id: String,
    /// Actores involucrados
    pub actors: (GodName, GodName),
    /// Tipo de conflicto
    pub conflict_type: ConflictType,
    /// Severidad
    pub severity: ConflictSeverity,
    /// Recurso o elemento en conflicto
    pub resource: String,
    /// Descripción del conflicto
    pub description: String,
    /// Estado actual
    pub status: ConflictStatus,
    /// Momento de detección
    pub detected_at: DateTime<Utc>,
    /// Última actualización
    pub updated_at: DateTime<Utc>,
    /// Veces escalado
    pub escalation_count: u32,
    /// Razón de escalado
    pub escalation_reason: Option<String>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
}

impl Conflict {
    /// Crea un nuevo conflicto
    pub fn new(
        actor_a: GodName,
        actor_b: GodName,
        resource: &str,
        conflict_type: ConflictType,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let severity = Self::calculate_severity(&conflict_type, resource);
        let description = Self::generate_description(&actor_a, &actor_b, resource, &conflict_type);

        Self {
            id,
            actors: (actor_a, actor_b),
            conflict_type,
            severity,
            resource: resource.to_string(),
            description,
            status: ConflictStatus::Detected,
            detected_at: Utc::now(),
            updated_at: Utc::now(),
            escalation_count: 0,
            escalation_reason: None,
            metadata: HashMap::new(),
        }
    }

    /// Calcula la severidad del conflicto
    fn calculate_severity(conflict_type: &ConflictType, resource: &str) -> ConflictSeverity {
        match conflict_type {
            ConflictType::State => ConflictSeverity::Critical,
            ConflictType::Dependency => ConflictSeverity::High,
            ConflictType::Resource => {
                if resource.contains("critical") || resource.contains("database") {
                    ConflictSeverity::High
                } else if resource.contains("cache") || resource.contains("temp") {
                    ConflictSeverity::Medium
                } else {
                    ConflictSeverity::Low
                }
            }
            ConflictType::Data => {
                if resource.contains("patient") || resource.contains("sensitive") {
                    ConflictSeverity::Critical
                } else {
                    ConflictSeverity::Medium
                }
            }
            ConflictType::Priority => ConflictSeverity::Medium,
            ConflictType::Timing => ConflictSeverity::Low,
            ConflictType::Communication => ConflictSeverity::Medium,
        }
    }

    /// Genera descripción del conflicto
    fn generate_description(
        actor_a: &GodName,
        actor_b: &GodName,
        resource: &str,
        conflict_type: &ConflictType,
    ) -> String {
        match conflict_type {
            ConflictType::Resource => format!(
                "Conflicto de recurso: {:?} y {:?} compiten por '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::Data => format!(
                "Conflicto de datos: {:?} y {:?} con acceso concurrente a '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::Priority => format!(
                "Conflicto de prioridades: {:?} y {:?} con prioridades encontradas en '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::Dependency => format!(
                "Conflicto de dependencia: {:?} y {:?} con dependencia circular en '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::Timing => format!(
                "Conflicto de tiempo: {:?} y {:?} con problemas de sincronización en '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::Communication => format!(
                "Conflicto de comunicación: {:?} y {:?} con fallos en '{}'",
                actor_a, actor_b, resource
            ),
            ConflictType::State => format!(
                "Conflicto de estado: {:?} y {:?} con estados inconsistentes en '{}'",
                actor_a, actor_b, resource
            ),
        }
    }

    /// Escala el conflicto
    pub fn escalate(&mut self, reason: &str) {
        self.escalation_count += 1;
        self.escalation_reason = Some(reason.to_string());
        self.status = ConflictStatus::Escalated;
        self.updated_at = Utc::now();
    }

    /// Verifica si está escalado
    pub fn is_escalated(&self) -> bool {
        matches!(self.status, ConflictStatus::Escalated)
    }

    /// Verifica si está resuelto
    pub fn is_resolved(&self) -> bool {
        matches!(self.status, ConflictStatus::Resolved)
    }

    /// Marca como resuelto
    pub fn mark_resolved(&mut self) {
        self.status = ConflictStatus::Resolved;
        self.updated_at = Utc::now();
    }

    /// Agrega metadato
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }

    /// Obtiene metadato
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Detector de conflictos
#[derive(Debug, Clone)]
pub struct ConflictDetector {
    /// Contador de conflictos detectados
    detection_count: u64,
    /// Cache de detecciones recientes
    recent_detections: HashMap<String, DateTime<Utc>>,
    /// Umbrales de detección
    thresholds: DetectionThresholds,
}

/// Umbrales para detección de conflictos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionThresholds {
    /// Mínimo tiempo entre detecciones del mismo conflicto (segundos)
    min_detection_interval: i64,
    /// Máximo número de detecciones antes de escalar automáticamente
    max_detections_before_escalation: u32,
    /// Nivel de recursos críticos
    critical_resources: Vec<String>,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            min_detection_interval: 30, // 30 segundos
            max_detections_before_escalation: 3,
            critical_resources: vec![
                "database".to_string(),
                "patient_data".to_string(),
                "auth".to_string(),
                "payment".to_string(),
            ],
        }
    }
}

impl ConflictDetector {
    /// Crea un nuevo detector
    pub fn new() -> Self {
        Self {
            detection_count: 0,
            recent_detections: HashMap::new(),
            thresholds: DetectionThresholds::default(),
        }
    }

    /// Crea un detector con umbrales personalizados
    pub fn with_thresholds(thresholds: DetectionThresholds) -> Self {
        Self {
            detection_count: 0,
            recent_detections: HashMap::new(),
            thresholds,
        }
    }

    /// Detecta un conflicto entre dos actores
    pub fn detect(
        &mut self,
        actor_a: GodName,
        actor_b: GodName,
        resource: &str,
        conflict_type: ConflictType,
    ) -> Result<Conflict, ActorError> {
        self.detection_count += 1;

        // Verificar si es una detección duplicada
        let detection_key = format!(
            "{:?}_{:?}_{}_{}",
            actor_a,
            actor_b,
            resource,
            conflict_type.clone() as u8
        );

        if let Some(last_detection) = self.recent_detections.get(&detection_key) {
            let time_since_last = Utc::now() - *last_detection;
            if time_since_last.num_seconds() < self.thresholds.min_detection_interval {
                // Es una detección duplicada muy reciente
                return Err(ActorError::Unknown {
                    god: actor_a,
                    message: format!("Detección duplicada de conflicto: {}", detection_key),
                });
            }
        }

        // Actualizar timestamp de detección
        self.recent_detections.insert(detection_key, Utc::now());

        // Crear conflicto
        let mut conflict = Conflict::new(actor_a, actor_b, resource, conflict_type.clone());

        // Agregar metadatos de detección
        conflict.add_metadata("detection_count", &self.detection_count.to_string());

        // Verificar si debe escalarse automáticamente
        if self.should_auto_escalate(&conflict) {
            conflict.add_metadata("auto_escalation", "true");
        }

        Ok(conflict)
    }

    /// Verifica si un conflicto debe escalarse automáticamente
    fn should_auto_escalate(&self, conflict: &Conflict) -> bool {
        match conflict.severity {
            ConflictSeverity::Critical => true,
            ConflictSeverity::High => {
                // Escalar automáticamente si es un recurso crítico
                self.thresholds
                    .critical_resources
                    .iter()
                    .any(|cr| conflict.resource.contains(cr))
            }
            _ => false,
        }
    }

    /// Verifica espera circular (deadlock potencial)
    pub fn check_circular_wait(&self, actors: &[GodName]) -> Option<String> {
        if actors.len() < 2 {
            return None;
        }

        // Implementación simple: verificar si hay dependencia circular
        // En una implementación real, esto sería más sofisticado
        let actor_map: HashMap<GodName, usize> = actors
            .iter()
            .enumerate()
            .map(|(i, actor)| (*actor, i))
            .collect();

        // Buscar patrones circulares
        for (i, actor) in actors.iter().enumerate() {
            let next = actors[(i + 1) % actors.len()];

            // Si el último depende del primero -> ciclo
            if i == actors.len() - 1 {
                let first = actors[0];
                if self.check_dependency(next, first) {
                    return Some(format!(
                        "Deadlock detectado: {:?} -> {:?} (ciclo cerrado)",
                        next, first
                    ));
                }
            } else if self.check_dependency(*actor, next) {
                // Continuar verificando la cadena
                continue;
            }
        }

        None
    }

    /// Verifica si un actor depende de otro (implementación simplificada)
    fn check_dependency(&self, _from: GodName, _to: GodName) -> bool {
        // Implementación simplificada - en realidad esto consultaría
        // el estado actual del sistema para determinar dependencias
        false
    }

    /// Limpia detecciones antiguas
    pub fn cleanup_old_detections(&mut self, older_than: chrono::Duration) {
        let cutoff = Utc::now() - older_than;

        self.recent_detections
            .retain(|_, timestamp| *timestamp > cutoff);
    }

    /// Obtiene estadísticas de detección
    pub fn get_detection_stats(&self) -> DetectionStats {
        DetectionStats {
            total_detections: self.detection_count,
            recent_detections: self.recent_detections.len(),
            threshold_config: self.thresholds.clone(),
        }
    }
}

/// Estadísticas de detección
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionStats {
    pub total_detections: u64,
    pub recent_detections: usize,
    pub threshold_config: DetectionThresholds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_creation() {
        let conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "database_connection",
            ConflictType::Resource,
        );

        assert!(!conflict.id.is_empty());
        assert_eq!(conflict.actors, (GodName::Zeus, GodName::Hades));
        assert_eq!(conflict.conflict_type, ConflictType::Resource);
        assert_eq!(conflict.status, ConflictStatus::Detected);
    }

    #[test]
    fn test_conflict_escalation() {
        let mut conflict = Conflict::new(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        );

        assert!(!conflict.is_escalated());
        assert_eq!(conflict.escalation_count, 0);

        conflict.escalate("Test escalation");

        assert!(conflict.is_escalated());
        assert_eq!(conflict.escalation_count, 1);
        assert_eq!(
            conflict.escalation_reason,
            Some("Test escalation".to_string())
        );
    }

    #[test]
    fn test_severity_calculation() {
        // Recurso crítico debe ser alta severidad
        let conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "database_connection",
            ConflictType::Resource,
        );
        assert_eq!(conflict.severity, ConflictSeverity::High);

        // Datos sensibles deben ser críticos
        let conflict = Conflict::new(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        );
        assert_eq!(conflict.severity, ConflictSeverity::Critical);
    }

    #[test]
    fn test_conflict_detector() {
        let mut detector = ConflictDetector::new();

        // Primera detección debe funcionar
        let result1 = detector.detect(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );
        assert!(result1.is_ok());

        // Detección duplicada inmediata debe fallar
        let result2 = detector.detect(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );
        assert!(result2.is_err());
    }

    #[test]
    fn test_metadata_operations() {
        let mut conflict = Conflict::new(
            GodName::Zeus,
            GodName::Hades,
            "test_resource",
            ConflictType::Resource,
        );

        conflict.add_metadata("test_key", "test_value");
        assert_eq!(
            conflict.get_metadata("test_key"),
            Some(&"test_value".to_string())
        );
        assert_eq!(conflict.get_metadata("nonexistent"), None);
    }
}
