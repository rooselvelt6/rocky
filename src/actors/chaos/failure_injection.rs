// src/actors/chaos/failure_injection.rs
// OLYMPUS v15 - Sistema de Inyección de Fallos para Chaos

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use tracing::{info, warn};

use crate::actors::GodName;
use crate::errors::ActorError;

/// Tipos de fallos que se pueden inyectar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureType {
    /// Fallo de red (latencia alta)
    NetworkLatency {
        target_god: GodName,
        latency_ms: u64,
    },
    
    /// Pérdida de paquetes
    PacketLoss {
        target_god: GodName,
        loss_percentage: f64,
    },
    
    /// Cuelgue del proceso
    ProcessHang {
        target_god: GodName,
        duration_seconds: u64,
    },
    
    /// Agotamiento de memoria
    MemoryExhaustion {
        target_god: GodName,
        target_mb: u64,
    },
    
    /// Alta CPU
    CPUPressure {
        target_god: GodName,
        target_percentage: f64,
    },
    
    /// Error de base de datos
    DatabaseError {
        target_god: GodName,
        error_type: String,
    },
    
    /// Partición de red
    NetworkPartition {
        target_actors: Vec<GodName>,
        duration_seconds: u64,
    },
    
    /// Corrupción de datos
    DataCorruption {
        target_god: GodName,
        corruption_rate: f64,
    },
    
    /// Timeouts
    Timeout {
        target_god: GodName,
        timeout_ms: u64,
    },
    
    /// Fallo de autenticación
    AuthenticationFailure {
        target_god: GodName,
        error_type: String,
    },
    
    /// Fallo aleatorio general
    RandomFailure {
        target_god: GodName,
        failure_description: String,
    },
}

/// Severidad de los fallos
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl FailureSeverity {
    /// Obtiene el valor numérico para comparación
    pub fn as_number(&self) -> u8 {
        match self {
            FailureSeverity::Low => 1,
            FailureSeverity::Medium => 2,
            FailureSeverity::High => 3,
            FailureSeverity::Critical => 4,
        }
    }
    
    /// Obtiene el color para visualización
    pub fn color(&self) -> &'static str {
        match self {
            FailureSeverity::Low => "🟢",
            FailureSeverity::Medium => "🟡",
            FailureSeverity::High => "🟠",
            FailureSeverity::Critical => "🔴",
        }
    }
}

/// Estado de un fallo inyectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureState {
    /// ID único del fallo
    pub id: String,
    /// Tipo de fallo
    pub failure_type: FailureType,
    /// Severidad
    pub severity: FailureSeverity,
    /// Momento de inyección
    pub injected_at: DateTime<Utc>,
    /// Momento de expiración (si aplica)
    pub expires_at: Option<DateTime<Utc>>,
    /// Si está actualmente activo
    pub active: bool,
    /// Veces que se ha activado
    pub activation_count: u32,
    /// Última vez que fue activado
    pub last_activated: Option<DateTime<Utc>>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
}

impl FailureState {
    /// Crea un nuevo estado de fallo
    pub fn new(failure_type: FailureType, severity: FailureSeverity, duration: Option<u64>) -> Self {
        let now = Utc::now();
        let expires_at = duration.map(|d| now + chrono::Duration::seconds(d as i64));
        
        Self {
            id: Uuid::new_v4().to_string(),
            failure_type,
            severity,
            injected_at: now,
            expires_at,
            active: false,
            activation_count: 0,
            last_activated: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Verifica si el fallo ha expirado
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Activa el fallo
    pub fn activate(&mut self) {
        self.active = true;
        self.activation_count += 1;
        self.last_activated = Some(Utc::now());
    }
    
    /// Desactiva el fallo
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    
    /// Obtiene la descripción del fallo
    pub fn description(&self) -> String {
        match &self.failure_type {
            FailureType::NetworkLatency { latency_ms, .. } => 
                format!("Latencia de red: {}ms", latency_ms),
            FailureType::PacketLoss { loss_percentage, .. } => 
                format!("Pérdida de paquetes: {:.1}%", loss_percentage),
            FailureType::ProcessHang { duration_seconds, .. } => 
                format!("Cuelgue de proceso: {}s", duration_seconds),
            FailureType::MemoryExhaustion { target_mb, .. } => 
                format!("Agotamiento de memoria: {}MB", target_mb),
            FailureType::CPUPressure { target_percentage, .. } => 
                format!("Presión de CPU: {:.1}%", target_percentage),
            FailureType::DatabaseError { error_type, .. } => 
                format!("Error de BD: {}", error_type),
            FailureType::NetworkPartition { target_actors, duration_seconds, .. } => 
                format!("Partición de red entre {:?} por {}s", target_actors, duration_seconds),
            FailureType::DataCorruption { corruption_rate, .. } => 
                format!("Corrupción de datos: {:.1}%", corruption_rate),
            FailureType::Timeout { timeout_ms, .. } => 
                format!("Timeout: {}ms", timeout_ms),
            FailureType::AuthenticationFailure { error_type, .. } => 
                format!("Fallo de autenticación: {}", error_type),
            FailureType::RandomFailure { failure_description, .. } => 
                format!("Fallo aleatorio: {}", failure_description),
        }
    }
}

/// Inyector de fallos
#[derive(Debug, Clone)]
pub struct FailureInjector {
    /// Fallos activos actualmente
    active_failures: Arc<RwLock<HashMap<String, FailureState>>>,
    
    /// Historial de todos los fallos inyectados
    injection_history: Arc<RwLock<Vec<FailureState>>>,
    
    /// Estadísticas de inyección
    stats: Arc<RwLock<InjectionStats>>,
    
    /// Configuración del inyector
    config: Arc<RwLock<InjectorConfig>>,
}

/// Configuración del inyector de fallos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectorConfig {
    /// Máximo de fallos concurrentes
    pub max_concurrent_failures: usize,
    /// Máximo de fallos por minuto
    pub max_failures_per_minute: u64,
    /// Actores protegidos contra inyección
    pub protected_actors: Vec<GodName>,
    /// Modo de prueba (simulación vs real)
    pub dry_run_mode: bool,
    /// Umbral de severidad para aprobación automática
    pub auto_approve_max_severity: FailureSeverity,
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_failures: 10,
            max_failures_per_minute: 5,
            protected_actors: vec![GodName::Zeus], // Proteger al supervisor
            dry_run_mode: false,
            auto_approve_max_severity: FailureSeverity::Medium,
        }
    }
}

/// Estadísticas de inyección de fallos
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InjectionStats {
    /// Total de fallos inyectados
    pub total_injections: u64,
    /// Fallos activos actualmente
    pub current_active: u64,
    /// Fallos exitosos
    pub successful_injections: u64,
    /// Fallos fallidos
    pub failed_injections: u64,
    /// Fallos por tipo
    pub failures_by_type: HashMap<String, u64>,
    /// Fallos por severidad
    pub failures_by_severity: HashMap<String, u64>,
    /// Fallos por objetivo
    pub failures_by_target: HashMap<String, u64>,
    /// Tiempo promedio de inyección
    pub average_injection_time_ms: f64,
}

impl FailureInjector {
    /// Crea un nuevo inyector de fallos
    pub fn new() -> Self {
        Self {
            active_failures: Arc::new(RwLock::new(HashMap::new())),
            injection_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(InjectionStats::default())),
            config: Arc::new(RwLock::new(InjectorConfig::default())),
        }
    }
    
    /// Inyecta un fallo específico
    pub async fn inject_failure(
        &mut self,
        target: GodName,
        failure_type: FailureType,
        severity: FailureSeverity,
        duration: Option<u64>,
    ) -> Result<String, ActorError> {
        let start_time = std::time::Instant::now();
        
        // Verificar límites
        self.check_limits(&target, &severity).await?;
        
        // Crear estado del fallo
        let mut failure_state = FailureState::new(failure_type.clone(), severity.clone(), duration);
        
        // Agregar metadatos específicos
        failure_state.metadata.insert("target".to_string(), format!("{:?}", target));
        failure_state.metadata.insert("severity".to_string(), format!("{:?}", severity));
        failure_state.metadata.insert("injection_method".to_string(), "manual".to_string());
        
        // Activar el fallo
        failure_state.activate();
        
        // Ejecutar la inyección según el tipo
        self.execute_injection(&mut failure_state).await?;
        
        // Guardar el fallo
        let failure_id = failure_state.id.clone();
        {
            let mut active_failures = self.active_failures.write().await;
            active_failures.insert(failure_id.clone(), failure_state.clone());
            
            let mut history = self.injection_history.write().await;
            history.push(failure_state.clone());
        }
        
        // Actualizar estadísticas
        self.update_stats(&failure_type, &severity, &target, start_time.elapsed().as_millis() as f64, true).await;
        
        info!("🌀 Fallo inyectado {}: {:?} en {:?}", severity.color(), failure_type, target);
        
        Ok(failure_id)
    }
    
    /// Inyecta un fallo aleatorio
    pub async fn inject_random_failure(
        &mut self,
        target: GodName,
        severity: FailureSeverity,
    ) -> Result<String, ActorError> {
        let failure_types = vec![
            FailureType::NetworkLatency { target_god: target, latency_ms: 1000 + (rand::random::<u64>() % 2000) },
            FailureType::PacketLoss { target_god: target, loss_percentage: rand::random::<f64>() * 20.0 },
            FailureType::Timeout { target_god: target, timeout_ms: 5000 + (rand::random::<u64>() % 10000) },
            FailureType::DatabaseError { target_god: target, error_type: "Connection Timeout".to_string() },
            FailureType::RandomFailure { 
                target_god: target, 
                failure_description: "Fallo aleatorio de prueba".to_string() 
            },
        ];
        
        let failure_type = failure_types[rand::random::<usize>() % failure_types.len()].clone();
        
        let duration = match severity {
            FailureSeverity::Low => Some(30),
            FailureSeverity::Medium => Some(60),
            FailureSeverity::High => Some(120),
            FailureSeverity::Critical => Some(300),
        };
        
        self.inject_failure(target, failure_type, severity, duration).await
    }
    
    /// Detiene un fallo específico
    pub async fn stop_failure(&mut self, failure_id: &str) -> Result<(), ActorError> {
        let mut active_failures = self.active_failures.write().await;
        
        if let Some(mut failure_state) = active_failures.remove(failure_id) {
            failure_state.deactivate();
            
            // Revertir los efectos del fallo
            self.revert_injection(&failure_state).await?;
            
            info!("🌀 Fallo detenido: {} ({})", failure_id, failure_state.description());
            Ok(())
        } else {
            Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Fallo no encontrado: {}", failure_id),
            })
        }
    }
    
    /// Limpia todos los fallos activos
    pub async fn cleanup(&mut self) -> Result<(), ActorError> {
        let mut active_failures = self.active_failures.write().await;
        
        for (failure_id, mut failure_state) in active_failures.drain() {
            failure_state.deactivate();
            
            // Revertir los efectos
            if let Err(e) = self.revert_injection(&failure_state).await {
                warn!("🌀 Error revirtiendo fallo {}: {}", failure_id, e);
            }
        }
        
        info!("🌀 Todos los fallos activos han sido limpiados");
        Ok(())
    }
    
    /// Obtiene estado actual de todos los fallos
    pub async fn get_active_failures(&self) -> HashMap<String, FailureState> {
        self.active_failures.read().await.clone()
    }
    
    /// Obtiene estadísticas de inyección
    pub async fn get_stats(&self) -> InjectionStats {
        self.stats.read().await.clone()
    }
    
    /// Verifica los límites antes de inyectar
    async fn check_limits(&self, target: &GodName, severity: &FailureSeverity) -> Result<(), ActorError> {
        let config = self.config.read().await;
        let active_failures = self.active_failures.read().await;
        
        // Verificar si el actor está protegido
        if config.protected_actors.contains(target) {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Actor protegido contra inyección: {:?}", target),
            });
        }
        
        // Verificar límite de fallos concurrentes
        if active_failures.len() >= config.max_concurrent_failures {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: "Límite de fallos concurrentes alcanzado".to_string(),
            });
        }
        
        // Verificar severidad máxima para aprobación automática
        if *severity > config.auto_approve_max_severity {
            return Err(ActorError::Unknown {
                god: GodName::Chaos,
                message: format!("Severidad requiere aprobación manual: {:?}", severity),
            });
        }
        
        Ok(())
    }
    
    /// Ejecuta la inyección específica según el tipo
    async fn execute_injection(&self, failure_state: &mut FailureState) -> Result<(), ActorError> {
        let config = self.config.read().await;
        
        if config.dry_run_mode {
            info!("🌀 [DRY RUN] Simulando inyección: {}", failure_state.description());
            return Ok(());
        }
        
        match &failure_state.failure_type {
            FailureType::NetworkLatency { latency_ms, target_god } => {
                self.inject_network_latency(*target_god, *latency_ms).await?;
            },
            FailureType::PacketLoss { loss_percentage, target_god } => {
                self.inject_packet_loss(*target_god, *loss_percentage).await?;
            },
            FailureType::ProcessHang { duration_seconds, target_god } => {
                self.inject_process_hang(*target_god, *duration_seconds).await?;
            },
            FailureType::MemoryExhaustion { target_mb, target_god } => {
                self.inject_memory_exhaustion(*target_god, *target_mb).await?;
            },
            FailureType::CPUPressure { target_percentage, target_god } => {
                self.inject_cpu_pressure(*target_god, *target_percentage).await?;
            },
            FailureType::DatabaseError { error_type, target_god } => {
                self.inject_database_error(*target_god, error_type.clone()).await?;
            },
            FailureType::NetworkPartition { target_actors, duration_seconds } => {
                self.inject_network_partition(target_actors.clone(), *duration_seconds).await?;
            },
            FailureType::DataCorruption { corruption_rate, target_god } => {
                self.inject_data_corruption(*target_god, *corruption_rate).await?;
            },
            FailureType::Timeout { timeout_ms, target_god } => {
                self.inject_timeout(*target_god, *timeout_ms).await?;
            },
            FailureType::AuthenticationFailure { error_type, target_god } => {
                self.inject_authentication_failure(*target_god, error_type.clone()).await?;
            },
            FailureType::RandomFailure { target_god, .. } => {
                self.inject_random_generic_failure(*target_god).await?;
            },
        }
        
        Ok(())
    }
    
    /// Implementación de inyección de latencia de red
    async fn inject_network_latency(&self, target: GodName, latency_ms: u64) -> Result<(), ActorError> {
        info!("🌀 Inyectando latencia de red: {:?} -> {}ms", target, latency_ms);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de pérdida de paquetes
    async fn inject_packet_loss(&self, target: GodName, loss_percentage: f64) -> Result<(), ActorError> {
        info!("🌀 Inyectando pérdida de paquetes: {:?} -> {:.1}%", target, loss_percentage);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de cuelgue de proceso
    async fn inject_process_hang(&self, target: GodName, duration_seconds: u64) -> Result<(), ActorError> {
        info!("🌀 Inyectando cuelgue de proceso: {:?} -> {}s", target, duration_seconds);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de agotamiento de memoria
    async fn inject_memory_exhaustion(&self, target: GodName, target_mb: u64) -> Result<(), ActorError> {
        info!("🌀 Inyectando agotamiento de memoria: {:?} -> {}MB", target, target_mb);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de presión de CPU
    async fn inject_cpu_pressure(&self, target: GodName, target_percentage: f64) -> Result<(), ActorError> {
        info!("🌀 Inyectando presión de CPU: {:?} -> {:.1}%", target, target_percentage);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de error de base de datos
    async fn inject_database_error(&self, target: GodName, error_type: String) -> Result<(), ActorError> {
        info!("🌀 Inyectando error de BD: {:?} -> {}", target, error_type);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de partición de red
    async fn inject_network_partition(&self, target_actors: Vec<GodName>, duration_seconds: u64) -> Result<(), ActorError> {
        info!("🌀 Inyectando partición de red: {:?} -> {}s", target_actors, duration_seconds);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de corrupción de datos
    async fn inject_data_corruption(&self, target: GodName, corruption_rate: f64) -> Result<(), ActorError> {
        info!("🌀 Inyectando corrupción de datos: {:?} -> {:.1}%", target, corruption_rate);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de timeout
    async fn inject_timeout(&self, target: GodName, timeout_ms: u64) -> Result<(), ActorError> {
        info!("🌀 Inyectando timeout: {:?} -> {}ms", target, timeout_ms);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de inyección de fallo de autenticación
    async fn inject_authentication_failure(&self, target: GodName, error_type: String) -> Result<(), ActorError> {
        info!("🌀 Inyectando fallo de autenticación: {:?} -> {}", target, error_type);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Implementación de fallo aleatorio genérico
    async fn inject_random_generic_failure(&self, target: GodName) -> Result<(), ActorError> {
        info!("🌀 Inyectando fallo aleatorio: {:?}", target);
        // Aquí se implementaría la lógica real de inyección
        Ok(())
    }
    
    /// Revierte los efectos de una inyección
    async fn revert_injection(&self, failure_state: &FailureState) -> Result<(), ActorError> {
        info!("🌀 Revirtiendo inyección: {}", failure_state.description());
        // Aquí se implementaría la lógica para revertir los efectos
        Ok(())
    }
    
    /// Actualiza estadísticas de inyección
    async fn update_stats(
        &self,
        failure_type: &FailureType,
        severity: &FailureSeverity,
        target: &GodName,
        injection_time_ms: f64,
        success: bool,
    ) {
        let mut stats = self.stats.write().await;
        
        stats.total_injections += 1;
        
        if success {
            stats.successful_injections += 1;
        } else {
            stats.failed_injections += 1;
        }
        
        // Actualizar promedio de tiempo
        let total_time = stats.average_injection_time_ms * (stats.total_injections - 1) as f64 + injection_time_ms;
        stats.average_injection_time_ms = total_time / stats.total_injections as f64;
        
        // Actualizar contadores por tipo
        let type_key = format!("{:?}", failure_type);
        *stats.failures_by_type.entry(type_key).or_insert(0) += 1;
        
        // Actualizar contadores por severidad
        let severity_key = format!("{:?}", severity);
        *stats.failures_by_severity.entry(severity_key).or_insert(0) += 1;
        
        // Actualizar contadores por objetivo
        let target_key = format!("{:?}", target);
        *stats.failures_by_target.entry(target_key).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::GodName;

    #[test]
    fn test_failure_state_creation() {
        let failure_type = FailureType::NetworkLatency {
            target_god: GodName::Zeus,
            latency_ms: 1000,
        };
        
        let failure_state = FailureState::new(
            failure_type.clone(),
            FailureSeverity::High,
            Some(60)
        );
        
        assert_eq!(failure_state.failure_type, failure_type);
        assert_eq!(failure_state.severity, FailureSeverity::High);
        assert!(failure_state.expires_at.is_some());
        assert!(!failure_state.active);
        assert_eq!(failure_state.activation_count, 0);
    }

    #[test]
    fn test_failure_severity_ordering() {
        assert!(FailureSeverity::Low < FailureSeverity::Medium);
        assert!(FailureSeverity::Medium < FailureSeverity::High);
        assert!(FailureSeverity::High < FailureSeverity::Critical);
        
        assert_eq!(FailureSeverity::Low.as_number(), 1);
        assert_eq!(FailureSeverity::Critical.as_number(), 4);
    }

    #[test]
    fn test_injector_creation() {
        let injector = FailureInjector::new();
        
        // El inyector debe inicializarse correctamente
        let active_failures = tokio::task::block_inplace(injector.get_active_failures());
        assert!(active_failures.is_empty());
        
        let stats = tokio::task::block_inplace(injector.get_stats());
        assert_eq!(stats.total_injections, 0);
    }

    #[tokio::test]
    async fn test_failure_injection() {
        let mut injector = FailureInjector::new();
        
        let result = injector.inject_failure(
            GodName::Hades,
            FailureType::NetworkLatency {
                target_god: GodName::Hades,
                latency_ms: 500,
            },
            FailureSeverity::Medium,
            Some(30)
        ).await;
        
        assert!(result.is_ok());
        
        let active_failures = injector.get_active_failures().await;
        assert_eq!(active_failures.len(), 1);
    }

    #[test]
    fn test_failure_description() {
        let failure_state = FailureState::new(
            FailureType::PacketLoss {
                target_god: GodName::Athena,
                loss_percentage: 10.5,
            },
            FailureSeverity::Low,
            None
        );
        
        let description = failure_state.description();
        assert!(description.contains("10.5%"));
        assert!(description.contains("pérdida"));
    }
}