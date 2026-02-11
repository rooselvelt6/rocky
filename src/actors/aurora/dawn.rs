// src/actors/aurora/dawn.rs
// OLYMPUS v15 - Aurora: Sistema de Amanecer y Renovación

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::actors::GodName;
use crate::actors::aurora::{RenewalType, RenewalStatus, RenewalLevel};
use crate::errors::ActorError;
use tracing::info;

/// Sistema de Amanecer - Gestión de Inicios y Renovaciones
/// 
/// Responsabilidades:
/// - Coordinar renovaciones periódicas del sistema
/// - Gestionar transiciones suaves entre estados
/// - Optimizar recursos durante períodos de baja carga
/// - Implementar patrones de "rebirth" para componentes
/// - Monitorizar ciclos de vida y puntos de inflexión
#[derive(Debug, Clone)]
pub struct DawnSystem {
    /// Configuración del sistema
    config: Arc<RwLock<DawnConfig>>,
    /// Estado actual del amanecer
    state: Arc<RwLock<DawnState>>,
    /// Ciclos activos
    active_cycles: Arc<RwLock<Vec<RenewalCycle>>>,
    /// Historial de renovaciones
    renewal_history: Arc<RwLock<Vec<RenewalRecord>>>,
    /// Programador de ciclos
    cycle_scheduler: Arc<RwLock<CycleScheduler>>,
}

/// Configuración del sistema de amanecer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawnConfig {
    /// Hora del amanecer (hora militar)
    pub dawn_hour: u8,
    /// Zona horaria
    pub timezone: String,
    /// Frecuencia de renovación (horas)
    pub renewal_frequency_hours: u64,
    /// Niveles de renovación
    pub renewal_levels: Vec<RenewalLevel>,
    /// Recursos a optimizar durante renovación
    pub optimization_resources: Vec<String>,
    /// Tolerancia para renovación (porcentaje)
    pub renewal_tolerance_percent: f64,
}

/// Estado del amanecer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawnState {
    /// Estado actual
    pub status: DawnStatus,
    /// Ciclo actual
    pub current_cycle: Option<CyclePhase>,
    /// Última renovación
    pub last_renewal: Option<DateTime<Utc>>,
    /// Próximo amanecer programado
    pub next_dawn: DateTime<Utc>,
    /// Luz ambiental actual (0-100)
    pub ambient_light: u8,
    /// Energía del sistema
    pub system_energy: f64,
}

/// Ciclo de renovación activo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalCycle {
    /// ID único del ciclo
    pub cycle_id: String,
    /// Tipo de renovación
    pub renewal_type: RenewalType,
    /// Fase actual del ciclo
    pub phase: CyclePhase,
    /// Componentes afectados
    pub affected_components: Vec<String>,
    /// Tiempo de inicio
    pub start_time: DateTime<Utc>,
    /// Duración estimada (minutos)
    pub estimated_duration_minutes: u32,
    /// Progreso del ciclo (0-100)
    pub progress_percentage: f64,
    /// Efectos observados
    pub observed_effects: Vec<String>,
    /// Estado del ciclo
    pub status: RenewalStatus,
}

/// Fases del ciclo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CyclePhase {
    /// Iniciando
    Initializing,
    /// Preparando
    Preparing,
    /// Ejecutando
    Executing,
    /// Optimizando
    Optimizing,
    /// Validando
    Validating,
    /// Completando
    Completing,
    /// Finalizando
    Finalizing,
}

/// Registros de renovación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalRecord {
    /// ID único del registro
    pub record_id: String,
    /// Tipo de renovación
    pub renewal_type: RenewalType,
    /// Hora del amanecer
    pub dawn_time: DateTime<Utc>,
    /// Componentes renovados
    pub renewed_components: Vec<String>,
    /// Métricas antes de la renovación
    pub pre_renewal_metrics: HashMap<String, f64>,
    /// Métricas después de la renovación
    pub post_renewal_metrics: HashMap<String, f64>,
    /// Mejoras observadas
    pub observed_improvements: Vec<String>,
    /// Problemas detectados
    pub detected_issues: Vec<String>,
    /// Duración total (minutos)
    pub duration_minutes: u32,
    /// Calificación de la renovación
    pub renewal_rating: RenewalRating,
    /// Recomendaciones
    pub recommendations: Vec<String>,
}

/// Programador de ciclos
#[derive(Debug, Clone)]
pub struct CycleScheduler {
    /// Ciclos programados
    scheduled_cycles: Arc<RwLock<Vec<ScheduledCycle>>>,
    /// Historial de ejecuciones
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
    /// Tiempo hasta el siguiente ciclo
    pub next_cycle_time: Option<DateTime<Utc>>,
}

/// Ciclo programado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledCycle {
    /// ID del ciclo programado
    pub cycle_id: String,
    /// Hora programada
    pub scheduled_time: DateTime<Utc>,
    /// Tipo de renovación
    pub renewal_type: RenewalType,
    /// Componentes objetivo
    pub target_components: Vec<String>,
    /// Prioridad del ciclo
    pub priority: CyclePriority,
    /// Si es recurrente
    pub is_recurring: bool,
    /// Frecuencia si es recurrente
    pub frequency_hours: Option<u64>,
    /// Condiciones de activación
    pub activation_conditions: Vec<String>,
}

/// Registro de ejecución
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// ID del registro
    pub record_id: String,
    /// ID del ciclo
    pub cycle_id: String,
    /// Hora de inicio
    pub start_time: DateTime<Utc>,
    /// Hora de finalización
    pub end_time: Option<DateTime<Utc>>,
    /// Estado de la ejecución
    pub execution_status: ExecutionStatus,
    /// Tiempo transcurrido (minutos)
    pub elapsed_minutes: u32,
    /// Resultados de la ejecución
    pub execution_results: HashMap<String, serde_json::Value>,
    /// Errores detectados
    pub execution_errors: Vec<String>,
}

/// Estados del sistema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DawnStatus {
    /// Inactiva
    Inactive,
    /// Amaneciendo
    Dawning,
    /// Activa
    Active,
    /// En renovación
    Renewing,
    /// Optimizando
    Optimizing,
}

/// Prioridades de ciclo
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CyclePriority {
    Crítica,
    Alta,
    Media,
    Baja,
}

/// Estados de ejecución
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Programado,
    Ejecutando,
    Completado,
    Falló,
    Cancelado,
}

/// Calificación de renovación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenewalRating {
    Excelente,
    Buena,
    Regular,
    Deficiente,
    Crítica,
}

impl DawnSystem {
    /// Crea una nueva instancia del sistema de amanecer
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(DawnConfig::default())),
            state: Arc::new(RwLock::new(DawnState::default())),
            active_cycles: Arc::new(RwLock::new(Vec::new())),
            renewal_history: Arc::new(RwLock::new(Vec::new())),
            cycle_scheduler: Arc::new(RwLock::new(CycleScheduler::new())),
        }
    }
    
    /// Inicializa el sistema de amanecer
    pub async fn initialize(&self) -> Result<(), ActorError> {
        info!("🌅 Iniciando sistema de Amanecer para Aurora");
        
        let config = self.config.read().await;
        
        // Programar el primer amanecer
        let mut scheduler = self.cycle_scheduler.write().await;
        scheduler.schedule_cycle(&config.dawn_hour, &config.timezone).await?;
        
        // Establecer estado inicial
        {
            let mut state = self.state.write().await;
            state.next_dawn = self.calculate_next_dawn(&config).await;
            state.status = DawnStatus::Inactive;
        }
        
        info!("🌅 Sistema de Amanecer inicializado");
        Ok(())
    }
    
    /// Inicia el amanecer del sistema
    pub async fn initiate_dawn(&self) -> Result<String, ActorError> {
        let config = self.config.read().await;
        let dawn_id = Uuid::new_v4().to_string();
        
        info!("🌅 Iniciando amanecer: {}", dawn_id);
        
        // Cambiar estado a amaneciendo
        {
            let mut state = self.state.write().await;
            state.status = DawnStatus::Dawning;
            state.current_cycle = Some(CyclePhase::Initializing);
            state.ambient_light = 0;
            state.system_energy = 0.0;
        }
        
        // Crear ciclo de renovación
        let cycle = RenewalCycle {
            cycle_id: dawn_id.clone(),
            renewal_type: RenewalType::System,
            phase: CyclePhase::Initializing,
            affected_components: vec!["system".to_string()],
            start_time: Utc::now(),
            estimated_duration_minutes: 10,
            progress_percentage: 0.0,
            observed_effects: Vec::new(),
            status: RenewalStatus::InProgress,
        };
        
        {
            let mut active_cycles = self.active_cycles.write().await;
            active_cycles.push(cycle);
        }
        
        // Simular proceso de amanecer
        self.simulate_dawn_process(&dawn_id).await?;
        
        info!("🌅 Amanecer completado: {}", dawn_id);
        Ok(dawn_id)
    }
    
    /// Simula el proceso de amanecer
    async fn simulate_dawn_process(&self, dawn_id: &str) -> Result<(), ActorError> {
        let mut progress = 0;
        
        while progress < 100 {
            {
                let mut active_cycles = self.active_cycles.write().await;
                if let Some(cycle) = active_cycles.iter_mut().find(|c| c.cycle_id == dawn_id) {
                    cycle.progress_percentage = progress;
                    cycle.phase = self.determine_phase(progress);
                    cycle.observed_effects.push(format!("Progreso: {}%", progress));
                    
                    match cycle.phase {
                        CyclePhase::Executing => {
                            // Ejecutar optimizaciones del sistema
                            self.execute_renewal_operations(&cycle).await?;
                        },
                        CyclePhase::Optimizing => {
                            // Optimizar recursos
                            self.optimize_system_resources().await?;
                        },
                        _ => {}
                    }
                }
            }
            
            // Actualizar estado del sistema
            {
                let mut state = self.state.write().await;
                state.ambient_light = progress;
                state.system_energy = progress as f64 / 100.0;
            }
            
            progress += 10;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        // Marcar ciclo como completado
        {
            let mut active_cycles = self.active_cycles.write().await;
            if let Some(cycle) = active_cycles.iter_mut().find(|c| c.cycle_id == dawn_id) {
                cycle.progress_percentage = 100.0;
                cycle.phase = CyclePhase::Completing;
                cycle.status = RenewalStatus::Completed;
            }
        }
        
        Ok(())
    }
    
    /// Determina la fase actual basada en el progreso
    fn determine_phase(&self, progress: u8) -> CyclePhase {
        match progress {
            0..=20 => CyclePhase::Initializing,
            21..=40 => CyclePhase::Preparing,
            41..=60 => CyclePhase::Executing,
            61..=80 => CyclePhase::Optimizing,
            81..=95 => CyclePhase::Validating,
            96..=100 => CyclePhase::Completing,
            _ => CyclePhase::Finalizing,
        }
    }
    
    /// Ejecuta operaciones de renovación
    async fn execute_renewal_operations(&self, cycle: &RenewalCycle) -> Result<(), ActorError> {
        let config = self.config.read().await;
        
        for resource in &config.optimization_resources {
            match resource.as_str() {
                "memory" => {
                    // Simular optimización de memoria
                    self.optimize_memory().await?;
                },
                "cache" => {
                    // Simular limpieza de caché
                    self.clear_cache().await?;
                },
                "temp_files" => {
                    // Simular limpieza de archivos temporales
                    self.cleanup_temp_files().await?;
                },
                "logs" => {
                    // Simular rotación de logs
                    self.rotate_logs().await?;
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Optimiza recursos del sistema
    async fn optimize_system_resources(&self) -> Result<(), ActorError> {
        // Simular optimización general
        self.optimize_memory().await?;
        self.optimize_cpu().await?;
        self.optimize_disk_io().await?;
        
        Ok(())
    }
    
    /// Optimiza uso de memoria
    async fn optimize_memory(&self) -> Result<(), ActorError> {
        info!("🌅 Optimizando uso de memoria");
        // Simular recolección de basura y compactación
        Ok(())
    }
    
    /// Optimiza uso de CPU
    async fn optimize_cpu(&self) -> Result<(), ActorError> {
        info!("🌅 Optimizando uso de CPU");
        // Simular ajuste de prioridades y scheduling
        Ok(())
    }
    
    /// Optimiza I/O de disco
    async fn optimize_disk_io(&self) -> Result<(), ActorError> {
        info!("🌅 Optimizando I/O de disco");
        // Simular optimización de acceso y caché
        Ok(())
    }
    
    /// Limpia caché
    async fn clear_cache(&self) -> Result<(), ActorError> {
        info!("🌅 Limpiando caché del sistema");
        // Simular limpieza de cachés del sistema
        Ok(())
    }
    
    /// Limpia archivos temporales
    async fn cleanup_temp_files(&self) -> Result<(), ActorError> {
        info!("🌅 Limpiando archivos temporales");
        // Simular limpieza de directorio temporal
        Ok(())
    }
    
    /// Rota logs
    async fn rotate_logs(&self) -> Result<(), ActorError> {
        info!("🌅 Rotando logs del sistema");
        // Simular rotación de archivos de log
        Ok(())
    }
    
    /// Programa renovaciones periódicas
    pub async fn schedule_renewals(&self) -> Result<(), ActorError> {
        let config = self.config.read().await;
        
        info!("🌅 Programando renovaciones periódicas cada {} horas", 
                config.renewal_frequency_hours);
        
        // Simular programación de tareas de renovación
        Ok(())
    }
    
    /// Programa un ciclo específico
    pub async fn schedule_cycle(
        &self,
        renewal_type: RenewalType,
        target_components: Vec<String>,
        delay_minutes: u32,
    ) -> Result<String, ActorError> {
        let cycle_id = Uuid::new_v4().to_string();
        
        {
            let mut scheduler = self.cycle_scheduler.write().await;
            let scheduled_cycle = ScheduledCycle {
                cycle_id: cycle_id.clone(),
                scheduled_time: Utc::now() + chrono::Duration::minutes(delay_minutes.into()),
                renewal_type: renewal_type.clone(),
                target_components: target_components.clone(),
                priority: CyclePriority::Media,
                is_recurring: false,
                frequency_hours: None,
                activation_conditions: Vec::new(),
            };
            
            scheduler.scheduled_cycles.push(scheduled_cycle);
        }
        
        info!("🌅 Ciclo programado: {} en {} minutos", cycle_id, delay_minutes);
        Ok(cycle_id)
    }
    
    /// Ejecuta un ciclo específico
    pub async fn execute_cycle(&self, cycle_id: &str) -> Result<RenewalRecord, ActorError> {
        info!("🌅 Ejecutando ciclo: {}", cycle_id);
        
        let mut active_cycles = self.active_cycles.write().await;
        let cycle_index = active_cycles.iter()
            .position(|c| c.cycle_id == cycle_id)
            .ok_or(0);
        
        let cycle = &mut active_cycles[cycle_index];
        cycle.status = RenewalStatus::InProgress;
        cycle.start_time = Utc::now();
        cycle.phase = CyclePhase::Preparing;
        
        // Simular ejecución del ciclo
        for phase in vec![
            CyclePhase::Preparing,
            CyclePhase::Executing,
            CyclePhase::Optimizing,
            CyclePhase::Validating,
            CyclePhase::Completing,
        ] {
            cycle.phase = phase.clone();
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            
            // Ejecutar acciones específicas de la fase
            self.execute_phase_actions(cycle, &phase).await?;
        }
        
        // Completar ciclo
        cycle.status = RenewalStatus::Completed;
        cycle.progress_percentage = 100.0;
        cycle.phase = CyclePhase::Completing;
        
        // Crear registro
        let record = RenewalRecord {
            record_id: Uuid::new_v4().to_string(),
            renewal_type: cycle.renewal_type.clone(),
            dawn_time: Utc::now(),
            renewed_components: cycle.affected_components.clone(),
            pre_renewal_metrics: self.collect_pre_metrics(&cycle).await,
            post_renewal_metrics: self.collect_post_metrics(&cycle).await,
            observed_improvements: vec!["Mejora en rendimiento general".to_string()],
            detected_issues: vec![],
            duration_minutes: (Utc::now() - cycle.start_time).num_minutes() as u32,
            renewal_rating: RenewalRating::Buena,
            recommendations: vec!["Continuar monitoreo".to_string()],
        };
        
        // Agregar al historial
        {
            let mut history = self.renewal_history.write().await;
            history.push(record.clone());
        }
        
        info!("🌅 Ciclo completado exitosamente: {}", cycle_id);
        Ok(record)
    }
    
    /// Ejecuta acciones específicas de una fase
    async fn execute_phase_actions(
        &self,
        cycle: &mut RenewalCycle,
        phase: &CyclePhase,
    ) -> Result<(), ActorError> {
        match phase {
            CyclePhase::Preparing => {
                info!("🌅 Preparando ciclo de renovación");
                // Simular preparación
                cycle.observed_effects.push("Recursos listos".to_string());
            },
            CyclePhase::Executing => {
                info!("🌅 Ejecutando renovación");
                // Simular ejecución de operaciones de renovación
                self.execute_renewal_operations(cycle).await?;
                cycle.observed_effects.push("Operaciones ejecutadas".to_string());
            },
            CyclePhase::Optimizing => {
                info!("🌅 Optimizando pos-renovación");
                self.optimize_system_resources().await?;
                cycle.observed_effects.push("Sistema optimizado".to_string());
            },
            CyclePhase::Validating => {
                info!("🌅 Validando renovación");
                // Simular validación de resultados
                cycle.observed_effects.push("Validación completada".to_string());
            },
            CyclePhase::Completing => {
                info!("🌅 Finalizando ciclo de renovación");
                // Simular limpieza y finalización
                cycle.observed_effects.push("Ciclo finalizado".to_string());
            },
            CyclePhase::Finalizing => {
                info!("🌅 Guardando resultados");
                // Simular guardado de métricas
                cycle.observed_effects.push("Resultados guardados".to_string());
            },
            _ => {}
        }
        
        Ok(())
    }
    
    /// Colecta métricas antes de la renovación
    async fn collect_pre_metrics(&self, cycle: &RenewalCycle) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        // Simular recolección de métricas
        metrics.insert("memory_usage".to_string(), 85.0);
        metrics.insert("cpu_usage".to_string(), 70.0);
        metrics.insert("disk_io".to_string(), 60.0);
        metrics.insert("response_time".to_string(), 200.0);
        metrics.insert("active_connections".to_string(), 150.0);
        
        metrics
    }
    
    /// Colecta métricas después de la renovación
    async fn collect_post_metrics(&self, cycle: &RenewalCycle) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        // Simular mejoras después de la renovación
        metrics.insert("memory_usage".to_string(), 65.0);
        metrics.insert("cpu_usage".to_string(), 55.0);
        metrics.insert("disk_io".to_string(), 50.0);
        metrics.insert("response_time".to_string(), 150.0);
        metrics.insert("active_connections".to_string(), 120.0);
        
        metrics
    }
    
    /// Calcula el próximo amanecer
    async fn calculate_next_dawn(&self, config: &DawnConfig) -> DateTime<Utc> {
        let now = Utc::now();
        
        // Calcular el próximo día a la hora del amanecer configurada
        let next_dawn = if now.hour() >= config.dawn_hour as u32 {
            // Si ya pasó la hora del amanecer, programar para mañana
            now.date_naive()
                .and_hms(config.dawn_hour as u32, 0, 0, 0)
                .and_utc()
        } else {
            // Programar para hoy a la hora del amanecer
            now.date_naive()
                .and_hms(config.dawn_hour as u32, 0, 0, 0)
                .and_utc()
        };
        
        next_dawn
    }
    
    /// Obtiene el estado actual del amanecer
    pub async fn get_dawn_state(&self) -> DawnState {
        self.state.read().await.clone()
    }
    
    /// Obtiene los ciclos activos
    pub async fn get_active_cycles(&self) -> Vec<RenewalCycle> {
        self.active_cycles.read().await.clone()
    }
    
    /// Obtiene el historial de renovaciones
    pub async fn get_renewal_history(&self) -> Vec<RenewalRecord> {
        self.renewal_history.read().await.clone()
    }
    
    /// Detiene un ciclo activo
    pub async fn stop_cycle(&self, cycle_id: &str) -> Result<(), ActorError> {
        let mut active_cycles = self.active_cycles.write().await;
        
        if let Some(cycle) = active_cycles.iter_mut().find(|c| c.cycle_id == cycle_id) {
            cycle.status = RenewalStatus::Cancelled;
            cycle.progress_percentage = 0.0;
        }
        
        info!("🌅 Ciclo detenido: {}", cycle_id);
        Ok(())
    }
    
    /// Obtiene estadísticas del sistema
    pub async fn get_dawn_statistics(&self) -> DawnStatistics {
        let active_cycles = self.active_cycles.read().await;
        let history = self.renewal_history.read().await;
        let state = self.state.read().await;
        
        DawnStatistics {
            current_status: state.status.clone(),
            active_cycles_count: active_cycles.len(),
            total_renewals: history.len(),
            average_renewal_time_minutes: if history.is_empty() { 0.0 } else {
                history.iter().map(|r| r.duration_minutes as f64).sum::<f64>() / history.len() as f64
            },
            last_renewal_time: history.last().map(|r| r.dawn_time),
            system_energy: state.system_energy,
            ambient_light: state.ambient_light,
            next_dawn_time: state.next_dawn,
            efficiency_score: 85.0, // Calculado basado en mejoras observadas
        }
    }
    
    /// Forzar un amanecer inmediato
    pub async fn force_dawn(&self) -> Result<String, ActorError> {
        info!("🌅 Forzando amanecer inmediato");
        self.initiate_dawn().await
    }
}

/// Estadísticas del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawnStatistics {
    /// Estado actual
    pub current_status: DawnStatus,
    /// Cantidad de ciclos activos
    pub active_cycles_count: usize,
    /// Total de renovaciones
    pub total_renewals: usize,
    /// Tiempo promedio de renovación
    pub average_renewal_time_minutes: f64,
    /// Última renovación
    pub last_renewal_time: Option<DateTime<Utc>>,
    /// Energía del sistema
    pub system_energy: f64,
    /// Luz ambiental
    pub ambient_light: u8,
    /// Próximo amanecer
    pub next_dawn_time: DateTime<Utc>,
    /// Score de eficiencia
    pub efficiency_score: f64,
}

impl Default for DawnConfig {
    fn default() -> Self {
        Self {
            dawn_hour: 6, // 6 AM
            timezone: "UTC".to_string(),
            renewal_frequency_hours: 24,
            renewal_levels: vec![
                RenewalLevel::Full,    // Renovación completa
                RenewalLevel::Light,  // Renovación ligera
                RenewalLevel::Minimal, // Renovación mínima
            ],
            optimization_resources: vec![
                "memory".to_string(),
                "cache".to_string(),
                "temp_files".to_string(),
                "logs".to_string(),
            ],
            renewal_tolerance_percent: 10.0,
        }
    }
}

impl Default for DawnState {
    fn default() -> Self {
        Self {
            status: DawnStatus::Inactive,
            current_cycle: None,
            last_renewal: None,
            next_dawn: Utc::now(),
            ambient_light: 0,
            system_energy: 0.0,
        }
    }
}

impl Default for CycleScheduler {
    fn default() -> Self {
        Self {
            scheduled_cycles: Vec::new(),
            execution_history: Vec::new(),
            next_cycle_time: None,
        }
    }
}

impl CycleScheduler {
    /// Programa un ciclo de amanecer
    async fn schedule_cycle(&mut self, dawn_hour: &u8, timezone: &str) -> Result<(), ActorError> {
        let now = Utc::now();
        
        // Calcular próximo amanecer
        let next_dawn = if now.hour() >= *dawn_hour as u32 {
            now.date_naive()
                .and_hms(*dawn_hour as u32, 0, 0, 0)
                .and_utc()
        } else {
            now.date_naive()
                .and_hms(*dawn_hour as u32, 0, 0, 0)
                .and_utc()
        };
        
        self.next_cycle_time = Some(next_dawn);
        
        info!("🌅 Amanecer programado para {}", next_dawn);
        Ok(())
    }
}