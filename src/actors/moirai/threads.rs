// src/actors/moirai/threads.rs
// OLYMPUS v15 - Sistema de Threads (Hilos del Destino) para pacientes

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Thread (Hilo del Destino) para un paciente
/// Representa el ciclo de vida completo de un paciente en UCI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientThread {
    /// ID único del thread
    pub id: String,
    /// ID del paciente
    pub patient_id: String,
    /// Estado actual del thread
    pub status: ThreadStatus,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    /// Timestamp de cierre (si aplica)
    pub closed_at: Option<DateTime<Utc>>,
    /// Outcome final
    pub outcome: Option<FateOutcome>,
    /// Eventos del ciclo de vida
    pub events: Vec<ThreadEvent>,
    /// Datos clínicos acumulados
    pub clinical_data: HashMap<String, serde_json::Value>,
    /// Trayectoria clínica actual
    pub trajectory: Option<TrajectoryPoint>,
    /// Contador de actualizaciones
    pub update_count: u32,
    /// Última actualización
    pub last_update: DateTime<Utc>,
}

impl PatientThread {
    /// Crea un nuevo thread para un paciente
    pub fn new(patient_id: &str, initial_data: serde_json::Value) -> Self {
        let now = Utc::now();
        let id = format!("thread_{}_{}", patient_id, now.timestamp_millis());

        let mut clinical_data = HashMap::new();
        clinical_data.insert("initial".to_string(), initial_data);

        Self {
            id,
            patient_id: patient_id.to_string(),
            status: ThreadStatus::Active,
            created_at: now,
            closed_at: None,
            outcome: None,
            events: vec![ThreadEvent::Created { timestamp: now }],
            clinical_data,
            trajectory: None,
            update_count: 0,
            last_update: now,
        }
    }

    /// Agrega un evento al thread
    pub fn add_event(&mut self, event: ThreadEvent) {
        self.events.push(event);
        self.update_count += 1;
        self.last_update = Utc::now();
    }

    /// Actualiza datos clínicos
    pub fn update_clinical_data(&mut self, key: &str, data: serde_json::Value) {
        self.clinical_data.insert(key.to_string(), data);
        self.update_count += 1;
        self.last_update = Utc::now();
    }

    /// Cierra el thread con un outcome
    pub fn close(&mut self, outcome: FateOutcome) {
        self.status = ThreadStatus::Closed;
        self.outcome = Some(outcome.clone());
        self.closed_at = Some(Utc::now());
        self.add_event(ThreadEvent::Closed {
            timestamp: Utc::now(),
            outcome,
        });
    }

    /// Verifica si el thread está activo
    pub fn is_active(&self) -> bool {
        self.status == ThreadStatus::Active
    }

    /// Obtiene la duración del thread
    pub fn duration(&self) -> Duration {
        let end = self.closed_at.unwrap_or_else(Utc::now);
        end - self.created_at
    }

    /// Obtiene eventos de un tipo específico
    pub fn events_by_type(&self, event_type: &str) -> Vec<&ThreadEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type() == event_type)
            .collect()
    }

    /// Obtiene el último score APACHE II registrado
    pub fn latest_apache(&self) -> Option<i32> {
        self.clinical_data
            .get("apache_ii")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
    }

    /// Obtiene el último score SOFA registrado
    pub fn latest_sofa(&self) -> Option<i32> {
        self.clinical_data
            .get("sofa")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
    }

    /// Obtiene la última evaluación NEWS2
    pub fn latest_news2(&self) -> Option<i32> {
        self.clinical_data
            .get("news2")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
    }
}

/// Estados de un thread
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    /// Thread activo, paciente en UCI
    Active,
    /// Thread crítico, paciente en estado grave
    Critical,
    /// Thread estable, paciente mejorando
    Stable,
    /// Thread cerrado, paciente dado de alta o fallecido
    Closed,
}

impl ThreadStatus {
    /// Verifica si el thread está en un estado activo
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            ThreadStatus::Active | ThreadStatus::Critical | ThreadStatus::Stable
        )
    }

    /// Obtiene el nombre legible del estado
    pub fn display_name(&self) -> &'static str {
        match self {
            ThreadStatus::Active => "Activo",
            ThreadStatus::Critical => "Crítico",
            ThreadStatus::Stable => "Estable",
            ThreadStatus::Closed => "Cerrado",
        }
    }

    /// Obtiene el color asociado para UI
    pub fn color(&self) -> &'static str {
        match self {
            ThreadStatus::Active => "#3498db",   // Azul
            ThreadStatus::Critical => "#e74c3c", // Rojo
            ThreadStatus::Stable => "#2ecc71",   // Verde
            ThreadStatus::Closed => "#95a5a6",   // Gris
        }
    }
}

/// Eventos del ciclo de vida de un thread
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ThreadEvent {
    /// Thread creado
    Created { timestamp: DateTime<Utc> },
    /// Actualización clínica
    ClinicalUpdate {
        timestamp: DateTime<Utc>,
        data: serde_json::Value,
    },
    /// Cambio de estado
    StatusChange {
        timestamp: DateTime<Utc>,
        from: ThreadStatus,
        to: ThreadStatus,
        reason: String,
    },
    /// Evaluación de escala
    ScaleEvaluation {
        timestamp: DateTime<Utc>,
        scale_type: String,
        score: i32,
    },
    /// Intervención médica
    Intervention {
        timestamp: DateTime<Utc>,
        intervention_type: String,
        description: String,
    },
    /// Complicación
    Complication {
        timestamp: DateTime<Utc>,
        complication_type: String,
        severity: String,
    },
    /// Thread cerrado
    Closed {
        timestamp: DateTime<Utc>,
        outcome: FateOutcome,
    },
}

impl ThreadEvent {
    /// Obtiene el tipo de evento como string
    pub fn event_type(&self) -> &'static str {
        match self {
            ThreadEvent::Created { .. } => "created",
            ThreadEvent::ClinicalUpdate { .. } => "clinical_update",
            ThreadEvent::StatusChange { .. } => "status_change",
            ThreadEvent::ScaleEvaluation { .. } => "scale_evaluation",
            ThreadEvent::Intervention { .. } => "intervention",
            ThreadEvent::Complication { .. } => "complication",
            ThreadEvent::Closed { .. } => "closed",
        }
    }

    /// Obtiene el timestamp del evento
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ThreadEvent::Created { timestamp } => *timestamp,
            ThreadEvent::ClinicalUpdate { timestamp, .. } => *timestamp,
            ThreadEvent::StatusChange { timestamp, .. } => *timestamp,
            ThreadEvent::ScaleEvaluation { timestamp, .. } => *timestamp,
            ThreadEvent::Intervention { timestamp, .. } => *timestamp,
            ThreadEvent::Complication { timestamp, .. } => *timestamp,
            ThreadEvent::Closed { timestamp, .. } => *timestamp,
        }
    }
}

/// Outcome final del destino
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FateOutcome {
    /// Recuperación heroica - superó expectativas
    Heroic,
    /// Trágico - fallecimiento
    Tragic,
    /// Legendario - caso excepcional
    Legendary,
    /// Olvidado - evolución sin incidentes
    Forgotten,
    /// Transformado - cambio significativo en condición
    Transformed,
    /// Indeterminado
    Undetermined,
}

impl FateOutcome {
    /// Verifica si es un outcome positivo
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            FateOutcome::Heroic | FateOutcome::Legendary | FateOutcome::Forgotten
        )
    }

    /// Verifica si es un outcome negativo
    pub fn is_negative(&self) -> bool {
        matches!(self, FateOutcome::Tragic)
    }

    /// Obtiene la descripción del outcome
    pub fn description(&self) -> &'static str {
        match self {
            FateOutcome::Heroic => "Recuperación excepcional",
            FateOutcome::Tragic => "Fallecimiento",
            FateOutcome::Legendary => "Caso de referencia",
            FateOutcome::Forgotten => "Evolución normal",
            FateOutcome::Transformed => "Cambio significativo",
            FateOutcome::Undetermined => "Pendiente",
        }
    }
}

/// Punto de trayectoria clínica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Tendencia
    pub trend: TrajectoryTrend,
    /// Velocidad de cambio
    pub velocity: f64,
    /// Dirección (mejorando, estable, empeorando)
    pub direction: String,
    /// Score de severidad normalizado (0-1)
    pub severity_score: f64,
    /// Factores de riesgo identificados
    pub risk_factors: Vec<String>,
}

/// Tendencias de trayectoria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrajectoryTrend {
    /// Mejorando rápidamente
    ImprovingFast,
    /// Mejorando lentamente
    ImprovingSlow,
    /// Estable
    Stable,
    /// Empeorando lentamente
    DeterioratingSlow,
    /// Empeorando rápidamente
    DeterioratingFast,
    /// Crítico
    Critical,
}

impl TrajectoryTrend {
    /// Obtiene el índice numérico de la tendencia
    pub fn index(&self) -> i32 {
        match self {
            TrajectoryTrend::ImprovingFast => 2,
            TrajectoryTrend::ImprovingSlow => 1,
            TrajectoryTrend::Stable => 0,
            TrajectoryTrend::DeterioratingSlow => -1,
            TrajectoryTrend::DeterioratingFast => -2,
            TrajectoryTrend::Critical => -3,
        }
    }

    /// Verifica si la trayectoria es positiva
    pub fn is_positive(&self) -> bool {
        self.index() > 0
    }

    /// Verifica si la trayectoria es negativa
    pub fn is_negative(&self) -> bool {
        self.index() < 0
    }

    /// Obtiene el color para visualización
    pub fn color(&self) -> &'static str {
        match self {
            TrajectoryTrend::ImprovingFast => "#27ae60",
            TrajectoryTrend::ImprovingSlow => "#2ecc71",
            TrajectoryTrend::Stable => "#f39c12",
            TrajectoryTrend::DeterioratingSlow => "#e67e22",
            TrajectoryTrend::DeterioratingFast => "#e74c3c",
            TrajectoryTrend::Critical => "#c0392b",
        }
    }
}

/// Manager de threads
#[derive(Debug, Clone)]
pub struct ThreadManager {
    threads: HashMap<String, PatientThread>,
}

impl ThreadManager {
    /// Crea un nuevo manager
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
        }
    }

    /// Crea un thread
    pub fn create_thread(
        &mut self,
        patient_id: &str,
        initial_data: serde_json::Value,
    ) -> PatientThread {
        let thread = PatientThread::new(patient_id, initial_data);
        self.threads.insert(patient_id.to_string(), thread.clone());
        thread
    }

    /// Obtiene un thread
    pub fn get_thread(&self, patient_id: &str) -> Option<&PatientThread> {
        self.threads.get(patient_id)
    }

    /// Obtiene un thread mutable
    pub fn get_thread_mut(&mut self, patient_id: &str) -> Option<&mut PatientThread> {
        self.threads.get_mut(patient_id)
    }

    /// Lista threads por estado
    pub fn list_by_status(&self, status: ThreadStatus) -> Vec<&PatientThread> {
        self.threads
            .values()
            .filter(|t| t.status == status)
            .collect()
    }

    /// Cierra un thread
    pub fn close_thread(&mut self, patient_id: &str, outcome: FateOutcome) -> Option<()> {
        self.threads.get_mut(patient_id).map(|t| t.close(outcome))
    }

    /// Obtiene estadísticas
    pub fn statistics(&self) -> ThreadStatistics {
        ThreadStatistics {
            total: self.threads.len(),
            active: self
                .threads
                .values()
                .filter(|t| t.status == ThreadStatus::Active)
                .count(),
            critical: self
                .threads
                .values()
                .filter(|t| t.status == ThreadStatus::Critical)
                .count(),
            stable: self
                .threads
                .values()
                .filter(|t| t.status == ThreadStatus::Stable)
                .count(),
            closed: self
                .threads
                .values()
                .filter(|t| t.status == ThreadStatus::Closed)
                .count(),
        }
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStatistics {
    pub total: usize,
    pub active: usize,
    pub critical: usize,
    pub stable: usize,
    pub closed: usize,
}
