// src/actors/chronos/scheduler.rs
// OLYMPUS v15 - Task Scheduler con parser cron

use crate::actors::chronos::tasks::{ScheduledTask, TaskStatus};
use crate::actors::GodName;
use crate::errors::ActorError;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Scheduler principal de tareas
#[derive(Debug, Clone)]
pub struct TaskScheduler {
    /// Cola ordenada de ejecuciones: timestamp -> [task_ids]
    execution_queue: BTreeMap<DateTime<Utc>, Vec<String>>,
    /// Cron parser para calcular próximas ejecuciones
    cron_parser: CronParser,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            execution_queue: BTreeMap::new(),
            cron_parser: CronParser::new(),
        }
    }

    /// Programa una tarea en el scheduler
    pub fn schedule_task(&mut self, task: &ScheduledTask) -> Result<(), ActorError> {
        // Calcular próxima ejecución basada en la expresión cron
        let next_execution = if let Some(ref cron) = task.cron_expression {
            self.cron_parser
                .next_execution(cron, Utc::now())
                .ok_or_else(|| ActorError::InvalidCommand {
                    god: GodName::Chronos,
                    reason: format!("Expresión cron inválida: {}", cron),
                })?
        } else {
            // Si no hay cron, es one-shot inmediato
            Utc::now()
        };

        // Agregar a la cola
        self.execution_queue
            .entry(next_execution)
            .or_default()
            .push(task.id.clone());

        Ok(())
    }

    /// Cancela una tarea del scheduler
    pub fn cancel_task(&mut self, task_id: &str) -> Result<(), ActorError> {
        // Remover de todas las entradas de la cola
        let mut to_remove = Vec::new();

        for (timestamp, task_ids) in &mut self.execution_queue {
            task_ids.retain(|id| id != task_id);
            if task_ids.is_empty() {
                to_remove.push(*timestamp);
            }
        }

        // Limpiar timestamps vacíos
        for timestamp in to_remove {
            self.execution_queue.remove(&timestamp);
        }

        Ok(())
    }

    /// Reprograma una tarea recurrente
    pub fn reschedule_task(&mut self, task: &ScheduledTask) -> Result<(), ActorError> {
        // Primero cancelar la programación anterior
        self.cancel_task(&task.id)?;

        // Si es recurrente, calcular nueva ejecución
        if task.task_type.is_recurring() && task.status != TaskStatus::Cancelled {
            self.schedule_task(task)?;
        }

        Ok(())
    }

    /// Obtiene las tareas que deben ejecutarse ahora o antes
    pub fn get_due_tasks(&self, now: DateTime<Utc>) -> Vec<String> {
        let mut due_tasks = Vec::new();

        for (timestamp, task_ids) in &self.execution_queue {
            if *timestamp <= now {
                due_tasks.extend(task_ids.clone());
            } else {
                break; // La cola está ordenada, podemos parar
            }
        }

        due_tasks
    }

    /// Obtiene las próximas N ejecuciones programadas
    pub fn get_next_executions(&self, limit: usize) -> Vec<(String, DateTime<Utc>)> {
        let mut executions = Vec::new();

        for (timestamp, task_ids) in &self.execution_queue {
            for task_id in task_ids {
                executions.push((task_id.clone(), *timestamp));
                if executions.len() >= limit {
                    return executions;
                }
            }
        }

        executions
    }

    /// Devuelve la cantidad de tareas programadas
    pub fn task_count(&self) -> usize {
        self.execution_queue.values().map(|v| v.len()).sum()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser de expresiones cron
/// Soporta formato: "sec min hour day month weekday"
/// Ejemplos:
/// - "0 0 * * * *" = Cada hora en punto
/// - "0 0 12 * * *" = Todos los días a las 12:00
/// - "0 */15 * * * *" = Cada 15 minutos
#[derive(Debug, Clone)]
pub struct CronParser;

impl CronParser {
    pub fn new() -> Self {
        Self
    }

    /// Calcula la próxima ejecución basada en una expresión cron
    /// Formato: "second minute hour day month weekday"
    /// Campos soportados:
    /// - * = cualquier valor
    /// - n = valor exacto
    /// - */n = cada n unidades
    /// - n-m = rango
    /// - n,m = lista de valores
    pub fn next_execution(&self, cron: &str, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let parts: Vec<&str> = cron.split_whitespace().collect();

        if parts.len() != 6 {
            return None; // Formato inválido
        }

        // Parsear cada campo
        let seconds = self.parse_field(parts[0], 0, 59)?;
        let minutes = self.parse_field(parts[1], 0, 59)?;
        let hours = self.parse_field(parts[2], 0, 23)?;
        let days = self.parse_field(parts[3], 1, 31)?;
        let months = self.parse_field(parts[4], 1, 12)?;
        let weekdays = self.parse_field(parts[5], 0, 6)?; // 0=Domingo

        // Buscar la próxima ejecución
        let mut candidate = from + chrono::Duration::seconds(1); // Empezar en el siguiente segundo

        // Limitar búsqueda a 4 años para evitar loops infinitos
        let max_date = from + chrono::Duration::days(365 * 4);

        while candidate <= max_date {
            // Verificar mes
            if !months.contains(&(candidate.month() as i32)) {
                candidate = self.advance_to_next_month(candidate);
                continue;
            }

            // Verificar día del mes
            if !days.contains(&(candidate.day() as i32)) {
                candidate = candidate + chrono::Duration::days(1);
                candidate = candidate.with_hour(0)?.with_minute(0)?.with_second(0)?;
                continue;
            }

            // Verificar día de la semana
            if !weekdays.contains(&(candidate.weekday().num_days_from_sunday() as i32)) {
                candidate = candidate + chrono::Duration::days(1);
                candidate = candidate.with_hour(0)?.with_minute(0)?.with_second(0)?;
                continue;
            }

            // Verificar hora
            if !hours.contains(&(candidate.hour() as i32)) {
                candidate = candidate + chrono::Duration::hours(1);
                candidate = candidate.with_minute(0)?.with_second(0)?;
                continue;
            }

            // Verificar minuto
            if !minutes.contains(&(candidate.minute() as i32)) {
                candidate = candidate + chrono::Duration::minutes(1);
                candidate = candidate.with_second(0)?;
                continue;
            }

            // Verificar segundo
            if !seconds.contains(&(candidate.second() as i32)) {
                candidate = candidate + chrono::Duration::seconds(1);
                continue;
            }

            // ¡Coincidencia encontrada!
            return Some(candidate);
        }

        None // No se encontró próxima ejecución en el rango
    }

    /// Parsea un campo cron y devuelve los valores permitidos
    fn parse_field(&self, field: &str, min: i32, max: i32) -> Option<Vec<i32>> {
        let mut values = Vec::new();

        // Dividir por coma (listas)
        for part in field.split(',') {
            if part == "*" {
                // Todos los valores
                for v in min..=max {
                    values.push(v);
                }
            } else if part.starts_with("*/") {
                // Cada N unidades
                let step: i32 = part[2..].parse().ok()?;
                let mut v = min;
                while v <= max {
                    values.push(v);
                    v += step;
                }
            } else if part.contains('-') {
                // Rango
                let range: Vec<&str> = part.split('-').collect();
                if range.len() == 2 {
                    let start: i32 = range[0].parse().ok()?;
                    let end: i32 = range[1].parse().ok()?;
                    for v in start..=end {
                        if v >= min && v <= max {
                            values.push(v);
                        }
                    }
                }
            } else {
                // Valor exacto
                let v: i32 = part.parse().ok()?;
                if v >= min && v <= max {
                    values.push(v);
                }
            }
        }

        // Eliminar duplicados y ordenar
        values.sort_unstable();
        values.dedup();

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }

    /// Avanza al primer día del siguiente mes
    fn advance_to_next_month(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        let year = date.year();
        let month = date.month();

        let (new_year, new_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        date.with_year(new_year)
            .and_then(|d| d.with_month(new_month))
            .and_then(|d| d.with_day(1))
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .unwrap_or(date)
    }
}

impl Default for CronParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Representa una expresión cron parseada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExpression {
    pub expression: String,
    pub description: String,
}

impl CronExpression {
    pub fn new(expression: &str) -> Self {
        Self {
            expression: expression.to_string(),
            description: Self::describe(expression),
        }
    }

    /// Genera una descripción legible de la expresión cron
    fn describe(expression: &str) -> String {
        // Descripciones básicas
        match expression {
            "0 * * * * *" => "Cada minuto".to_string(),
            "0 0 * * * *" => "Cada hora".to_string(),
            "0 0 0 * * *" => "Cada día a medianoche".to_string(),
            "0 0 12 * * *" => "Cada día al mediodía".to_string(),
            "0 0 * * * 0" => "Cada domingo".to_string(),
            "0 0 1 * * *" => "El primero de cada mes".to_string(),
            _ => format!("Cron: {}", expression),
        }
    }
}
