// src/actors/chronos/time_events.rs
// OLYMPUS v15 - Eventos temporales para el sistema

use crate::actors::GodName;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Evento temporal emitido por Chronos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEvent {
    /// ID del evento
    pub id: String,
    /// Tipo de evento temporal
    pub event_type: TimeEventType,
    /// Timestamp del evento
    pub timestamp: DateTime<Utc>,
    /// Actor relacionado (si aplica)
    pub actor: Option<GodName>,
    /// ID de la tarea relacionada (si aplica)
    pub task_id: Option<String>,
    /// Nombre de la tarea (si aplica)
    pub task_name: Option<String>,
    /// Datos adicionales del evento
    pub data: serde_json::Value,
}

impl TimeEvent {
    /// Crea un evento de inicio de ejecución de tarea
    pub fn task_started(task_id: &str, task_name: &str, actor: GodName) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::TaskStarted,
            timestamp: Utc::now(),
            actor: Some(actor),
            task_id: Some(task_id.to_string()),
            task_name: Some(task_name.to_string()),
            data: serde_json::json!({}),
        }
    }

    /// Crea un evento de fin de ejecución de tarea
    pub fn task_completed(task_id: &str, task_name: &str, success: bool, duration_ms: u64) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::TaskCompleted,
            timestamp: Utc::now(),
            actor: None,
            task_id: Some(task_id.to_string()),
            task_name: Some(task_name.to_string()),
            data: serde_json::json!({
                "success": success,
                "duration_ms": duration_ms,
            }),
        }
    }

    /// Crea un evento de tarea fallida
    pub fn task_failed(task_id: &str, task_name: &str, error: &str) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::TaskFailed,
            timestamp: Utc::now(),
            actor: None,
            task_id: Some(task_id.to_string()),
            task_name: Some(task_name.to_string()),
            data: serde_json::json!({
                "error": error,
            }),
        }
    }

    /// Crea un evento de tarea cancelada
    pub fn task_cancelled(task_id: &str, task_name: &str) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::TaskCancelled,
            timestamp: Utc::now(),
            actor: None,
            task_id: Some(task_id.to_string()),
            task_name: Some(task_name.to_string()),
            data: serde_json::json!({}),
        }
    }

    /// Crea un evento de tarea programada
    pub fn task_scheduled(task_id: &str, task_name: &str, scheduled_at: DateTime<Utc>) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::TaskScheduled,
            timestamp: Utc::now(),
            actor: None,
            task_id: Some(task_id.to_string()),
            task_name: Some(task_name.to_string()),
            data: serde_json::json!({
                "scheduled_at": scheduled_at,
            }),
        }
    }

    /// Crea un evento de scheduler iniciado
    pub fn scheduler_started() -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::SchedulerStarted,
            timestamp: Utc::now(),
            actor: None,
            task_id: None,
            task_name: None,
            data: serde_json::json!({}),
        }
    }

    /// Crea un evento de scheduler detenido
    pub fn scheduler_stopped() -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::SchedulerStopped,
            timestamp: Utc::now(),
            actor: None,
            task_id: None,
            task_name: None,
            data: serde_json::json!({}),
        }
    }

    /// Crea un evento de tick del scheduler (cada ciclo)
    pub fn scheduler_tick(tasks_checked: u64, tasks_executed: u64) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::SchedulerTick,
            timestamp: Utc::now(),
            actor: None,
            task_id: None,
            task_name: None,
            data: serde_json::json!({
                "tasks_checked": tasks_checked,
                "tasks_executed": tasks_executed,
            }),
        }
    }

    /// Crea un evento de alarma temporal
    pub fn alarm_triggered(alarm_id: &str, alarm_name: &str) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::AlarmTriggered,
            timestamp: Utc::now(),
            actor: None,
            task_id: Some(alarm_id.to_string()),
            task_name: Some(alarm_name.to_string()),
            data: serde_json::json!({}),
        }
    }

    /// Crea un evento de heartbeat del scheduler
    pub fn scheduler_heartbeat(active_tasks: u64, uptime_seconds: u64) -> Self {
        Self {
            id: format!("evt_{}", Utc::now().timestamp_millis()),
            event_type: TimeEventType::SchedulerHeartbeat,
            timestamp: Utc::now(),
            actor: None,
            task_id: None,
            task_name: None,
            data: serde_json::json!({
                "active_tasks": active_tasks,
                "uptime_seconds": uptime_seconds,
            }),
        }
    }
}

/// Tipos de eventos temporales
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeEventType {
    /// Tarea iniciada
    TaskStarted,
    /// Tarea completada
    TaskCompleted,
    /// Tarea fallida
    TaskFailed,
    /// Tarea cancelada
    TaskCancelled,
    /// Tarea programada
    TaskScheduled,
    /// Scheduler iniciado
    SchedulerStarted,
    /// Scheduler detenido
    SchedulerStopped,
    /// Tick del scheduler
    SchedulerTick,
    /// Heartbeat del scheduler
    SchedulerHeartbeat,
    /// Alarma disparada
    AlarmTriggered,
}

/// Colección de eventos temporales
#[derive(Debug, Clone, Default)]
pub struct TimeEventLog {
    events: Vec<TimeEvent>,
    max_size: usize,
}

impl TimeEventLog {
    /// Crea un nuevo log con capacidad máxima
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Agrega un evento al log
    pub fn add(&mut self, event: TimeEvent) {
        if self.events.len() >= self.max_size {
            self.events.remove(0); // Eliminar el más antiguo
        }
        self.events.push(event);
    }

    /// Obtiene los últimos N eventos
    pub fn last_n(&self, n: usize) -> Vec<&TimeEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Obtiene todos los eventos de un tipo específico
    pub fn by_type(&self, event_type: TimeEventType) -> Vec<&TimeEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Obtiene eventos de una tarea específica
    pub fn by_task(&self, task_id: &str) -> Vec<&TimeEvent> {
        self.events
            .iter()
            .filter(|e| e.task_id.as_ref().map_or(false, |id| id == task_id))
            .collect()
    }

    /// Limpia el log
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Cantidad de eventos
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Verifica si está vacío
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Sistema de alarmas temporales
#[derive(Debug, Clone)]
pub struct AlarmSystem {
    alarms: Vec<TemporalAlarm>,
}

impl AlarmSystem {
    /// Crea un nuevo sistema de alarmas
    pub fn new() -> Self {
        Self { alarms: Vec::new() }
    }

    /// Agrega una alarma
    pub fn add_alarm(&mut self, alarm: TemporalAlarm) {
        self.alarms.push(alarm);
    }

    /// Elimina una alarma por ID
    pub fn remove_alarm(&mut self, alarm_id: &str) {
        self.alarms.retain(|a| a.id != alarm_id);
    }

    /// Verifica alarmas que deben dispararse
    pub fn check_alarms(&self, now: DateTime<Utc>) -> Vec<&TemporalAlarm> {
        self.alarms
            .iter()
            .filter(|a| a.should_trigger(now))
            .collect()
    }

    /// Obtiene alarmas activas
    pub fn active_alarms(&self) -> Vec<&TemporalAlarm> {
        self.alarms.iter().filter(|a| a.is_active()).collect()
    }
}

impl Default for AlarmSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Alarma temporal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAlarm {
    /// ID único
    pub id: String,
    /// Nombre descriptivo
    pub name: String,
    /// Timestamp de disparo
    pub trigger_at: DateTime<Utc>,
    /// Si está activa
    pub active: bool,
    /// Si debe repetirse
    pub recurring: bool,
    /// Intervalo de repetición (segundos)
    pub repeat_interval: Option<u64>,
    /// Payload de la alarma
    pub payload: serde_json::Value,
}

impl TemporalAlarm {
    /// Crea una nueva alarma
    pub fn new(id: &str, name: &str, trigger_at: DateTime<Utc>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            trigger_at,
            active: true,
            recurring: false,
            repeat_interval: None,
            payload: serde_json::json!({}),
        }
    }

    /// Verifica si debe dispararse ahora
    pub fn should_trigger(&self, now: DateTime<Utc>) -> bool {
        self.active && now >= self.trigger_at
    }

    /// Verifica si está activa
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Desactiva la alarma
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Activa la alarma
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Establece como recurrente
    pub fn set_recurring(&mut self, interval_seconds: u64) {
        self.recurring = true;
        self.repeat_interval = Some(interval_seconds);
    }

    /// Actualiza el tiempo de disparo (para recurrentes)
    pub fn reschedule(&mut self) {
        if self.recurring {
            if let Some(interval) = self.repeat_interval {
                self.trigger_at = self.trigger_at + chrono::Duration::seconds(interval as i64);
            }
        }
    }
}
