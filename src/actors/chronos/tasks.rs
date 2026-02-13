// src/actors/chronos/tasks.rs
// OLYMPUS v15 - Definición y gestión de tareas programadas

use crate::actors::GodName;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Definición de una tarea para ser programada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    /// Nombre descriptivo de la tarea
    pub name: String,
    /// Tipo de tarea
    #[serde(rename = "type")]
    pub task_type: TaskType,
    /// Expresión cron (opcional para one-shot)
    pub cron_expression: Option<String>,
    /// Payload de la tarea (JSON)
    pub payload: serde_json::Value,
    /// Actor creador de la tarea
    pub creator: Option<GodName>,
}

/// Tarea programada completa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// ID único de la tarea
    pub id: String,
    /// Nombre descriptivo
    pub name: String,
    /// Tipo de tarea
    #[serde(rename = "type")]
    pub task_type: TaskType,
    /// Expresión cron (si aplica)
    pub cron_expression: Option<String>,
    /// Payload de ejecución
    pub payload: serde_json::Value,
    /// Estado actual
    pub status: TaskStatus,
    /// Actor que creó la tarea
    pub creator: GodName,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Última actualización
    pub updated_at: DateTime<Utc>,
    /// Última ejecución (si aplica)
    pub last_execution: Option<DateTime<Utc>>,
    /// Contador de ejecuciones
    pub execution_count: u64,
    /// Resultado de la última ejecución
    pub last_result: Option<TaskResult>,
}

impl ScheduledTask {
    /// Crea una nueva tarea programada
    pub fn new(
        id: String,
        name: String,
        task_type: TaskType,
        cron_expression: Option<String>,
        payload: serde_json::Value,
        creator: GodName,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            task_type,
            cron_expression,
            payload,
            status: TaskStatus::Pending,
            creator,
            created_at: now,
            updated_at: now,
            last_execution: None,
            execution_count: 0,
            last_result: None,
        }
    }

    /// Verifica si la tarea debe ejecutarse ahora
    pub fn is_due(&self, _now: DateTime<Utc>) -> bool {
        if self.status == TaskStatus::Paused || self.status == TaskStatus::Cancelled {
            return false;
        }

        match &self.cron_expression {
            Some(_) => self.status == TaskStatus::Pending,
            None => self.status == TaskStatus::Pending && self.execution_count == 0,
        }
    }

    /// Marca la tarea como completada
    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Marca la tarea como fallida
    pub fn mark_failed(&mut self, _error: &str) {
        self.status = TaskStatus::Failed;
        self.updated_at = Utc::now();
    }
}

/// Tipos de tareas soportadas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Tarea única (one-shot)
    OneShot,
    /// Tarea recurrente con cron
    Recurring,
    /// Tarea con intervalo fijo en segundos
    Interval(u64),
}

impl TaskType {
    /// Verifica si es una tarea recurrente
    pub fn is_recurring(&self) -> bool {
        matches!(self, TaskType::Recurring | TaskType::Interval(_))
    }

    /// Obtiene el intervalo en segundos (si aplica)
    pub fn interval_seconds(&self) -> Option<u64> {
        match self {
            TaskType::Interval(seconds) => Some(*seconds),
            _ => None,
        }
    }
}

/// Estados de una tarea
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Esperando ejecución
    Pending,
    /// En ejecución
    Running,
    /// Completada exitosamente
    Completed,
    /// Falló en la ejecución
    Failed,
    /// Cancelada manualmente
    Cancelled,
    /// Pausada temporalmente
    Paused,
}

/// Resultado de la ejecución de una tarea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Si la ejecución fue exitosa
    pub success: bool,
    /// Mensaje descriptivo
    pub message: String,
    /// Timestamp de ejecución
    pub executed_at: DateTime<Utc>,
    /// Duración en milisegundos
    pub duration_ms: u64,
    /// Output de la tarea (JSON)
    pub output: serde_json::Value,
}

impl TaskResult {
    /// Crea un resultado exitoso
    pub fn success(message: &str, output: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            executed_at: Utc::now(),
            duration_ms: 0,
            output,
        }
    }

    /// Crea un resultado fallido
    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            executed_at: Utc::now(),
            duration_ms: 0,
            output: serde_json::json!({"error": message}),
        }
    }

    /// Construye el resultado con duración
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Tipos de payload de tareas comunes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum TaskPayload {
    /// Generar reporte
    GenerateReport { report_type: String, format: String },
    /// Limpiar datos antiguos
    CleanupData { older_than_days: u64 },
    /// Enviar notificación
    SendNotification { recipient: String, message: String },
    /// Backup de base de datos
    BackupDatabase { destination: String },
    /// Health check del sistema
    HealthCheck { components: Vec<String> },
    /// Reiniciar actor
    RestartActor { actor_name: String },
    /// Comando personalizado
    Custom(serde_json::Value),
}

/// Builder para crear definiciones de tareas
pub struct TaskDefinitionBuilder {
    name: String,
    task_type: TaskType,
    cron_expression: Option<String>,
    payload: serde_json::Value,
    creator: Option<GodName>,
}

impl TaskDefinitionBuilder {
    /// Inicia un builder para tarea one-shot
    pub fn one_shot(name: &str) -> Self {
        Self {
            name: name.to_string(),
            task_type: TaskType::OneShot,
            cron_expression: None,
            payload: serde_json::json!({}),
            creator: None,
        }
    }

    /// Inicia un builder para tarea recurrente
    pub fn recurring(name: &str, cron: &str) -> Self {
        Self {
            name: name.to_string(),
            task_type: TaskType::Recurring,
            cron_expression: Some(cron.to_string()),
            payload: serde_json::json!({}),
            creator: None,
        }
    }

    /// Inicia un builder para tarea con intervalo
    pub fn interval(name: &str, seconds: u64) -> Self {
        Self {
            name: name.to_string(),
            task_type: TaskType::Interval(seconds),
            cron_expression: None,
            payload: serde_json::json!({}),
            creator: None,
        }
    }

    /// Agrega payload a la tarea
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Establece el creador de la tarea
    pub fn with_creator(mut self, creator: GodName) -> Self {
        self.creator = Some(creator);
        self
    }

    /// Construye la definición de tarea
    pub fn build(self) -> TaskDefinition {
        TaskDefinition {
            name: self.name,
            task_type: self.task_type,
            cron_expression: self.cron_expression,
            payload: self.payload,
            creator: self.creator,
        }
    }
}
