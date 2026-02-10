// src/actors/demeter/alerts.rs
// OLYMPUS v15 - Sistema de alertas de recursos

use crate::actors::demeter::resources::ResourceType;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Nivel de alerta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    /// Advertencia - requiere atención
    Warning,
    /// Crítico - requiere acción inmediata
    Critical,
}

impl AlertLevel {
    /// Obtiene el nombre legible del nivel
    pub fn display_name(&self) -> &'static str {
        match self {
            AlertLevel::Warning => "Advertencia",
            AlertLevel::Critical => "Crítico",
        }
    }

    /// Obtiene el color asociado para UI
    pub fn color(&self) -> &'static str {
        match self {
            AlertLevel::Warning => "#FFA500",  // Naranja
            AlertLevel::Critical => "#FF0000", // Rojo
        }
    }

    /// Comprueba si este nivel es más severo que otro
    pub fn is_more_severe_than(&self, other: AlertLevel) -> bool {
        matches!((self, other), (AlertLevel::Critical, AlertLevel::Warning))
    }
}

/// Umbral de alerta configurado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    /// Tipo de recurso
    pub resource_type: ResourceType,
    /// Valor umbral (0.0 - 1.0)
    pub threshold: f64,
    /// Nivel de la alerta
    pub level: AlertLevel,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    /// Descripción opcional
    pub description: Option<String>,
}

impl AlertThreshold {
    /// Crea un nuevo umbral
    pub fn new(resource_type: ResourceType, threshold: f64, level: AlertLevel) -> Self {
        Self {
            resource_type,
            threshold: threshold.clamp(0.0, 1.0),
            level,
            created_at: Utc::now(),
            description: None,
        }
    }

    /// Crea un nuevo umbral con descripción
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Verifica si un valor supera este umbral
    pub fn is_triggered(&self, value: f64) -> bool {
        value >= self.threshold
    }
}

/// Alerta de recurso activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAlert {
    /// ID único de la alerta
    pub id: String,
    /// Tipo de recurso afectado
    pub resource_type: ResourceType,
    /// Nivel de la alerta
    pub level: AlertLevel,
    /// Umbral que se superó
    pub threshold: f64,
    /// Valor actual cuando se disparó
    pub current_value: f64,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    /// Timestamp de resolución (si aplica)
    pub resolved_at: Option<DateTime<Utc>>,
    /// Si está resuelta
    pub resolved: bool,
    /// Mensaje descriptivo
    pub message: String,
    /// Acciones recomendadas
    pub recommendations: Vec<String>,
    /// Contador de ocurrencias (para alertas recurrentes)
    pub occurrence_count: u32,
    /// Última vez que ocurrió
    pub last_occurrence: DateTime<Utc>,
}

impl ResourceAlert {
    /// Crea una nueva alerta
    pub fn new(
        resource_type: ResourceType,
        level: AlertLevel,
        threshold: f64,
        current_value: f64,
    ) -> Self {
        let now = Utc::now();
        let id = format!("alert_{}_{}", now.timestamp_millis(), std::process::id());

        let message = format!(
            "{} - Uso de {}: {:.1}% (umbral: {:.1}%)",
            level.display_name(),
            resource_type.display_name(),
            current_value * 100.0,
            threshold * 100.0
        );

        let recommendations = Self::generate_recommendations(resource_type, level);

        Self {
            id,
            resource_type,
            level,
            threshold,
            current_value,
            created_at: now,
            resolved_at: None,
            resolved: false,
            message,
            recommendations,
            occurrence_count: 1,
            last_occurrence: now,
        }
    }

    /// Genera recomendaciones basadas en el tipo de recurso y nivel
    fn generate_recommendations(resource_type: ResourceType, level: AlertLevel) -> Vec<String> {
        let mut recommendations = Vec::new();

        match resource_type {
            ResourceType::Cpu => {
                recommendations.push("Identificar procesos con alto consumo de CPU".to_string());
                recommendations
                    .push("Considerar escalado horizontal si es persistente".to_string());
                if level == AlertLevel::Critical {
                    recommendations
                        .push("Revisar posibles loops infinitos o deadlocks".to_string());
                    recommendations
                        .push("Considerar reinicio de servicios no críticos".to_string());
                }
            }
            ResourceType::Memory => {
                recommendations.push("Identificar posibles memory leaks".to_string());
                recommendations.push("Revisar cachés y buffers".to_string());
                if level == AlertLevel::Critical {
                    recommendations
                        .push("Considerar reinicio de servicios con alto uso".to_string());
                    recommendations.push("Verificar swap y memoria disponible".to_string());
                }
            }
            ResourceType::Storage => {
                recommendations.push("Limpiar archivos temporales y logs".to_string());
                recommendations.push("Comprimir o archivar datos antiguos".to_string());
                if level == AlertLevel::Critical {
                    recommendations
                        .push("Eliminar archivos innecesarios inmediatamente".to_string());
                    recommendations.push("Considerar expansión de storage".to_string());
                }
            }
            ResourceType::Network => {
                recommendations.push("Identificar fuente de tráfico alto".to_string());
                recommendations.push("Verificar posibles ataques DDoS".to_string());
                if level == AlertLevel::Critical {
                    recommendations.push("Implementar rate limiting".to_string());
                    recommendations.push("Considerar balanceo de carga".to_string());
                }
            }
        }

        recommendations
    }

    /// Marca la alerta como resuelta
    pub fn resolve(&mut self) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
    }

    /// Registra una nueva ocurrencia de la misma alerta
    pub fn record_occurrence(&mut self, current_value: f64) {
        self.occurrence_count += 1;
        self.last_occurrence = Utc::now();
        self.current_value = current_value;
    }

    /// Verifica si la alerta está expirada
    pub fn is_expired(&self, max_age: Duration) -> bool {
        if self.resolved {
            return false; // Las alertas resueltas no expiran
        }
        Utc::now() - self.created_at > max_age
    }

    /// Duración de la alerta
    pub fn duration(&self) -> Duration {
        let end = self.resolved_at.unwrap_or_else(Utc::now);
        end - self.created_at
    }

    /// Obtiene un resumen de la alerta
    pub fn summary(&self) -> AlertSummary {
        AlertSummary {
            id: self.id.clone(),
            resource_type: self.resource_type,
            level: self.level,
            message: self.message.clone(),
            resolved: self.resolved,
            age_seconds: self.duration().num_seconds(),
        }
    }
}

/// Resumen de alerta para UI/API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    /// ID de la alerta
    pub id: String,
    /// Tipo de recurso
    pub resource_type: ResourceType,
    /// Nivel
    pub level: AlertLevel,
    /// Mensaje
    pub message: String,
    /// Si está resuelta
    pub resolved: bool,
    /// Edad en segundos
    pub age_seconds: i64,
}

/// Manager de alertas
#[derive(Debug, Clone)]
pub struct AlertManager {
    /// Alertas activas y históricas
    alerts: Vec<ResourceAlert>,
    /// Máximo número de alertas a mantener
    max_alerts: usize,
    /// Duración máxima de alertas activas
    max_alert_age: Duration,
}

impl AlertManager {
    /// Crea un nuevo manager de alertas
    pub fn new() -> Self {
        Self {
            alerts: Vec::with_capacity(1000),
            max_alerts: 1000,
            max_alert_age: Duration::days(7),
        }
    }

    /// Agrega una nueva alerta o actualiza una existente
    pub fn add_alert(&mut self, alert: ResourceAlert) {
        // Buscar alerta similar no resuelta
        let existing_idx = self.alerts.iter_mut().position(|a| {
            a.resource_type == alert.resource_type && a.level == alert.level && !a.resolved
        });

        if let Some(idx) = existing_idx {
            // Actualizar alerta existente
            self.alerts[idx].record_occurrence(alert.current_value);
        } else {
            // Agregar nueva alerta
            if self.alerts.len() >= self.max_alerts {
                self.cleanup_old_alerts();
            }
            self.alerts.push(alert);
        }
    }

    /// Resuelve alertas para un recurso específico
    pub fn resolve_alerts_for_resource(&mut self, resource_type: ResourceType) {
        for alert in self.alerts.iter_mut() {
            if alert.resource_type == resource_type && !alert.resolved {
                alert.resolve();
            }
        }
    }

    /// Obtiene alertas activas
    pub fn active_alerts(&self) -> Vec<&ResourceAlert> {
        self.alerts.iter().filter(|a| !a.resolved).collect()
    }

    /// Obtiene alertas activas para un recurso específico
    pub fn active_alerts_for_resource(&self, resource_type: ResourceType) -> Vec<&ResourceAlert> {
        self.alerts
            .iter()
            .filter(|a| a.resource_type == resource_type && !a.resolved)
            .collect()
    }

    /// Obtiene alertas por nivel
    pub fn alerts_by_level(&self, level: AlertLevel) -> Vec<&ResourceAlert> {
        self.alerts.iter().filter(|a| a.level == level).collect()
    }

    /// Obtiene todas las alertas
    pub fn all_alerts(&self) -> &[ResourceAlert] {
        &self.alerts
    }

    /// Limpia alertas antiguas
    pub fn cleanup_old_alerts(&mut self) {
        let now = Utc::now();

        // Remover alertas resueltas muy antiguas
        self.alerts.retain(|a| {
            if let Some(resolved_at) = a.resolved_at {
                now - resolved_at < Duration::days(30) // Mantener 30 días
            } else {
                // Mantener alertas activas que no han expirado
                !a.is_expired(self.max_alert_age)
            }
        });

        // Si aún hay demasiadas, remover las más antiguas
        if self.alerts.len() >= self.max_alerts {
            self.alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            self.alerts.truncate(self.max_alerts);
        }
    }

    /// Obtiene estadísticas de alertas
    pub fn statistics(&self) -> AlertStatistics {
        let active = self.active_alerts();
        let warning_count = active
            .iter()
            .filter(|a| a.level == AlertLevel::Warning)
            .count();
        let critical_count = active
            .iter()
            .filter(|a| a.level == AlertLevel::Critical)
            .count();

        AlertStatistics {
            total_alerts: self.alerts.len(),
            active_alerts: active.len(),
            warning_alerts: warning_count,
            critical_alerts: critical_count,
            resolved_alerts: self.alerts.iter().filter(|a| a.resolved).count(),
        }
    }

    /// Obtiene alertas para exportar
    pub fn export_alerts(&self, since: DateTime<Utc>) -> Vec<&ResourceAlert> {
        self.alerts
            .iter()
            .filter(|a| a.created_at >= since)
            .collect()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de alertas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    /// Total de alertas
    pub total_alerts: usize,
    /// Alertas activas
    pub active_alerts: usize,
    /// Alertas de advertencia
    pub warning_alerts: usize,
    /// Alertas críticas
    pub critical_alerts: usize,
    /// Alertas resueltas
    pub resolved_alerts: usize,
}

/// Builder para crear umbrales
pub struct AlertThresholdBuilder {
    resource_type: ResourceType,
    threshold: f64,
    level: AlertLevel,
    description: Option<String>,
}

impl AlertThresholdBuilder {
    /// Inicia un builder para umbral de warning
    pub fn warning(resource_type: ResourceType, threshold: f64) -> Self {
        Self {
            resource_type,
            threshold,
            level: AlertLevel::Warning,
            description: None,
        }
    }

    /// Inicia un builder para umbral crítico
    pub fn critical(resource_type: ResourceType, threshold: f64) -> Self {
        Self {
            resource_type,
            threshold,
            level: AlertLevel::Critical,
            description: None,
        }
    }

    /// Agrega descripción
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Construye el umbral
    pub fn build(self) -> AlertThreshold {
        let mut threshold = AlertThreshold::new(self.resource_type, self.threshold, self.level);
        if let Some(desc) = self.description {
            threshold.description = Some(desc);
        }
        threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_threshold() {
        let threshold = AlertThreshold::new(ResourceType::Cpu, 0.8, AlertLevel::Warning);

        assert!(threshold.is_triggered(0.85));
        assert!(!threshold.is_triggered(0.75));
        assert_eq!(threshold.resource_type, ResourceType::Cpu);
    }

    #[test]
    fn test_resource_alert() {
        let alert = ResourceAlert::new(ResourceType::Memory, AlertLevel::Critical, 0.9, 0.95);

        assert!(!alert.resolved);
        assert!(alert.message.contains("Memoria"));
        assert!(!alert.recommendations.is_empty());

        let mut alert = alert;
        alert.resolve();
        assert!(alert.resolved);
        assert!(alert.resolved_at.is_some());
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();

        let alert1 = ResourceAlert::new(ResourceType::Cpu, AlertLevel::Warning, 0.8, 0.85);
        let alert2 = ResourceAlert::new(ResourceType::Memory, AlertLevel::Critical, 0.9, 0.95);

        manager.add_alert(alert1);
        manager.add_alert(alert2);

        assert_eq!(manager.active_alerts().len(), 2);
        assert_eq!(manager.alerts_by_level(AlertLevel::Critical).len(), 1);

        manager.resolve_alerts_for_resource(ResourceType::Cpu);
        assert_eq!(manager.active_alerts().len(), 1);

        let stats = manager.statistics();
        assert_eq!(stats.total_alerts, 2);
        assert_eq!(stats.resolved_alerts, 1);
    }

    #[test]
    fn test_alert_threshold_builder() {
        let threshold = AlertThresholdBuilder::warning(ResourceType::Storage, 0.85)
            .with_description("Storage getting full")
            .build();

        assert_eq!(threshold.level, AlertLevel::Warning);
        assert_eq!(threshold.threshold, 0.85);
        assert_eq!(
            threshold.description,
            Some("Storage getting full".to_string())
        );
    }
}
