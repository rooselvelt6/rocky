// src/actors/zeus/governance.rs
// OLYMPUS v15 - Zeus Governance Controller
// Decisiones de alto nivel del Olimpo con Circuit Breaker, Feature Flags y Rate Limiting

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, warn, error};

use super::GodName;

/// Decisión de gobernanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceDecision {
    NoAction,
    RestartActor { actor: GodName, strategy: super::RecoveryStrategy },
    RestartAll,
    EmergencyShutdown { reason: String },
    ScaleResources { actor: GodName, resources: serde_json::Value },
    NotifyStakeholders { message: String },
    EnableFeatureFlag { flag: String },
    DisableFeatureFlag { flag: String },
    OpenCircuitBreaker { component: String },
    CloseCircuitBreaker { component: String },
    RateLimit { actor: GodName, limit: u64 },
}

/// Situación de gobernanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceSituation {
    ActorUnhealthy { actor: GodName, error_count: u64 },
    MultipleActorsUnhealthy { count: usize, actors: Vec<GodName> },
    HighErrorRate { rate: f64 },
    PerformanceDegradation { latency_ms: u64, actor: GodName },
    ResourceExhaustion { resource: String, usage_percent: f64, actor: GodName },
    SecurityBreach { severity: SecuritySeverity, source: String },
    CircuitBreakerOpen { component: String },
    RateLimitExceeded { actor: GodName, current: u64, limit: u64 },
    FeatureFlagChanged { flag: String, enabled: bool },
    SystemHealthy,
}

/// Severidad de seguridad
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Controlador de gobernanza
#[derive(Debug, Clone)]
pub struct GovernanceController {
    decisions: Arc<RwLock<Vec<GovernanceRecord>>>,
    thresholds: Arc<RwLock<GovernanceThresholds>>,
    feature_flags: Arc<RwLock<FeatureFlagsManager>>,
    circuit_breaker: Arc<RwLock<CircuitBreakerManager>>,
    rate_limiter: Arc<RwLock<RateLimitManager>>,
    access_policies: Arc<RwLock<AccessPolicyManager>>,
    enabled: Arc<RwLock<bool>>,
}

/// Registro de decisiones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub situation: GovernanceSituation,
    pub decision: GovernanceDecision,
    pub actor: Option<GodName>,
    pub executed: bool,
    pub result: Option<String>,
}

/// Umbrales de gobernanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceThresholds {
    pub max_errors_before_restart: u64,
    pub max_unhealthy_actors: usize,
    pub max_error_rate: f64,
    pub max_latency_ms: u64,
    pub max_resource_usage_percent: f64,
    pub circuit_breaker_threshold: u64,
    pub rate_limit_requests_per_second: u64,
    pub rate_limit_burst_size: u64,
}

impl Default for GovernanceThresholds {
    fn default() -> Self {
        Self {
            max_errors_before_restart: 10,
            max_unhealthy_actors: 2,
            max_error_rate: 0.1,
            max_latency_ms: 1000,
            max_resource_usage_percent: 90.0,
            circuit_breaker_threshold: 5,
            rate_limit_requests_per_second: 1000,
            rate_limit_burst_size: 100,
        }
    }
}

/// Feature Flags Manager
#[derive(Debug, Clone, Default)]
pub struct FeatureFlagsManager {
    flags: HashMap<String, FeatureFlag>,
}

/// Feature Flag individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub modified_by: Option<String>,
    pub rollout_percentage: f64, // 0.0 - 100.0 para rollout gradual
}

impl FeatureFlag {
    pub fn new(name: &str, description: &str, enabled: bool) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            enabled,
            description: description.to_string(),
            created_at: now,
            modified_at: now,
            modified_by: None,
            rollout_percentage: if enabled { 100.0 } else { 0.0 },
        }
    }
    
    /// Verifica si está habilitado para un actor específico (para rollout gradual)
    pub fn is_enabled_for(&self, actor_id: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        if self.rollout_percentage >= 100.0 {
            return true;
        }
        
        // Hash simple para determinar si el actor está en el rollout
        let hash = self.simple_hash(actor_id);
        (hash % 100) < (self.rollout_percentage as u64)
    }
    
    fn simple_hash(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

/// Circuit Breaker Manager
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerManager {
    breakers: HashMap<String, CircuitBreaker>,
}

/// Circuit Breaker individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub component: String,
    pub state: CircuitState,
    pub failure_count: u64,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub threshold: u64,
    pub timeout_seconds: u64,
    pub half_open_max_calls: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,     // Funcionamiento normal
    Open,       // Circuito abierto, rechazando llamadas
    HalfOpen,   // Probando si se puede cerrar
}

impl CircuitBreaker {
    pub fn new(component: &str, threshold: u64, timeout_seconds: u64) -> Self {
        Self {
            component: component.to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            threshold,
            timeout_seconds,
            half_open_max_calls: 3,
        }
    }
    
    /// Registra un fallo
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());
        
        if self.failure_count >= self.threshold {
            if self.state == CircuitState::Closed {
                self.state = CircuitState::Open;
                warn!("⚡ Zeus: Circuit breaker OPEN for {}", self.component);
            }
        }
    }
    
    /// Registra un éxito
    pub fn record_success(&mut self) {
        if self.state == CircuitState::HalfOpen {
            self.failure_count = 0;
            self.state = CircuitState::Closed;
            info!("⚡ Zeus: Circuit breaker CLOSED for {}", self.component);
        } else {
            // Reducir contador de fallos gradualmente
            if self.failure_count > 0 {
                self.failure_count -= 1;
            }
        }
    }
    
    /// Verifica si se debe intentar half-open
    pub fn check_half_open(&mut self) -> bool {
        if self.state == CircuitState::Open {
            if let Some(last_failure) = self.last_failure {
                let elapsed = (Utc::now() - last_failure).num_seconds();
                if elapsed >= self.timeout_seconds as i64 {
                    self.state = CircuitState::HalfOpen;
                    info!("⚡ Zeus: Circuit breaker HALF-OPEN for {}", self.component);
                    return true;
                }
            }
        }
        false
    }
    
    /// Verifica si se permite la llamada
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Verificar si debe pasar a half-open
                self.check_half_open();
                self.state != CircuitState::Open
            }
            CircuitState::HalfOpen => {
                // Permitir llamadas limitadas en half-open
                self.failure_count < self.half_open_max_calls
            }
        }
    }
}

/// Rate Limit Manager
#[derive(Debug, Clone, Default)]
pub struct RateLimitManager {
    limits: HashMap<GodName, ActorRateLimit>,
    global_limit: Option<GlobalRateLimit>,
}

/// Rate Limit por actor
#[derive(Debug, Clone)]
pub struct ActorRateLimit {
    pub actor: GodName,
    pub requests_per_second: u64,
    pub burst_size: u64,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub request_count: u64,
    pub total_requests: u64,
    pub limited_requests: u64,
}

impl ActorRateLimit {
    pub fn new(actor: GodName, requests_per_second: u64, burst_size: u64) -> Self {
        Self {
            actor,
            requests_per_second,
            burst_size,
            window_start: Utc::now(),
            request_count: 0,
            total_requests: 0,
            limited_requests: 0,
        }
    }
    
    /// Verifica si se permite la solicitud
    pub fn allow_request(&mut self) -> bool {
        let now = Utc::now();
        let elapsed = (now - self.window_start).num_seconds() as u64;
        
        // Reiniciar ventana si ha pasado 1 segundo
        if elapsed >= 1 {
            self.window_start = now;
            self.request_count = 0;
        }
        
        self.total_requests += 1;
        
        // Permitir hasta burst_size + requests_per_second por ventana
        let allowed = self.request_count < (self.requests_per_second + self.burst_size);
        
        if allowed {
            self.request_count += 1;
            true
        } else {
            self.limited_requests += 1;
            false
        }
    }
    
    /// Obtiene estadísticas
    pub fn get_stats(&self) -> RateLimitStats {
        let limit_rate = if self.total_requests > 0 {
            (self.limited_requests as f64 / self.total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        RateLimitStats {
            actor: self.actor,
            total_requests: self.total_requests,
            limited_requests: self.limited_requests,
            limit_rate_percent: limit_rate,
            current_requests_in_window: self.request_count,
        }
    }
}

/// Rate Limit global
#[derive(Debug, Clone)]
pub struct GlobalRateLimit {
    pub requests_per_second: u64,
    pub burst_size: u64,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub request_count: u64,
}

/// Estadísticas de rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStats {
    pub actor: GodName,
    pub total_requests: u64,
    pub limited_requests: u64,
    pub limit_rate_percent: f64,
    pub current_requests_in_window: u64,
}

/// Access Policy Manager
#[derive(Debug, Clone, Default)]
pub struct AccessPolicyManager {
    policies: HashMap<String, AccessPolicy>,
    role_permissions: HashMap<String, Vec<Permission>>,
}

/// Política de acceso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub name: String,
    pub resource: String,
    pub allowed_roles: Vec<String>,
    pub allowed_actors: Vec<GodName>,
    pub conditions: Vec<AccessCondition>,
    pub enabled: bool,
}

/// Condición de acceso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCondition {
    TimeWindow { start_hour: u8, end_hour: u8 },
    RateLimit { max_requests: u64, window_seconds: u64 },
    RequireAuthentication,
    RequireMfa,
    Custom(String),
}

/// Permiso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
    Custom(String),
}

impl GovernanceController {
    pub fn new() -> Self {
        Self {
            decisions: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(GovernanceThresholds::default())),
            feature_flags: Arc::new(RwLock::new(FeatureFlagsManager::default())),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreakerManager::default())),
            rate_limiter: Arc::new(RwLock::new(RateLimitManager::default())),
            access_policies: Arc::new(RwLock::new(AccessPolicyManager::default())),
            enabled: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Habilita/deshabilita gobernanza
    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
        info!("⚡ Zeus: Governance {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Verifica si la gobernanza está habilitada
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
    
    /// Toma una decisión de gobernanza
    pub async fn make_decision(&self, situation: &GovernanceSituation) -> GovernanceDecision {
        if !self.is_enabled().await {
            return GovernanceDecision::NoAction;
        }
        
        let thresholds = self.thresholds.read().await;
        
        let decision = match situation {
            GovernanceSituation::ActorUnhealthy { actor, error_count } => {
                if *error_count >= thresholds.max_errors_before_restart {
                    GovernanceDecision::RestartActor {
                        actor: *actor,
                        strategy: super::RecoveryStrategy::OneForOne,
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::MultipleActorsUnhealthy { count, actors } => {
                if *count >= thresholds.max_unhealthy_actors {
                    // Si son Trinity members, escalar inmediatamente
                    let has_trinity = actors.iter().any(|a| {
                        matches!(a, GodName::Zeus | GodName::Hades | GodName::Poseidon)
                    });
                    
                    if has_trinity {
                        GovernanceDecision::EmergencyShutdown {
                            reason: "Multiple Trinity members unhealthy".to_string(),
                        }
                    } else {
                        GovernanceDecision::RestartAll
                    }
                } else {
                    GovernanceDecision::NotifyStakeholders {
                        message: format!("{} actors are unhealthy", count),
                    }
                }
            }
            GovernanceSituation::HighErrorRate { rate } => {
                if *rate >= thresholds.max_error_rate {
                    GovernanceDecision::NotifyStakeholders {
                        message: format!("High error rate detected: {:.2}%", rate * 100.0),
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::PerformanceDegradation { latency_ms, actor } => {
                if *latency_ms >= thresholds.max_latency_ms {
                    GovernanceDecision::ScaleResources {
                        actor: *actor,
                        resources: serde_json::json!({"scale_up": true}),
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::ResourceExhaustion { resource, usage_percent, actor } => {
                if *usage_percent >= thresholds.max_resource_usage_percent {
                    GovernanceDecision::ScaleResources {
                        actor: *actor,
                        resources: serde_json::json!({
                            "resource": resource,
                            "scale_up": true,
                            "usage_percent": usage_percent
                        }),
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::SecurityBreach { severity, source } => {
                match severity {
                    SecuritySeverity::Critical => {
                        GovernanceDecision::EmergencyShutdown {
                            reason: format!("Critical security breach from {}", source),
                        }
                    }
                    SecuritySeverity::High => {
                        GovernanceDecision::NotifyStakeholders {
                            message: format!("High severity security breach from {}", source),
                        }
                    }
                    _ => GovernanceDecision::NoAction,
                }
            }
            GovernanceSituation::CircuitBreakerOpen { component } => {
                GovernanceDecision::OpenCircuitBreaker {
                    component: component.clone(),
                }
            }
            GovernanceSituation::RateLimitExceeded { actor, current, limit } => {
                if *current > limit * 2 {
                    // Más del doble del límite
                    GovernanceDecision::NotifyStakeholders {
                        message: format!("Actor {:?} severely exceeding rate limit: {}/{}", actor, current, limit),
                    }
                } else {
                    GovernanceDecision::NoAction
                }
            }
            GovernanceSituation::SystemHealthy => GovernanceDecision::NoAction,
            _ => GovernanceDecision::NoAction,
        };
        
        // Registrar decisión
        self.record_decision(situation.clone(), decision.clone(), None).await;
        
        decision
    }
    
    /// Registra una decisión
    pub async fn record_decision(
        &self, 
        situation: GovernanceSituation, 
        decision: GovernanceDecision, 
        actor: Option<GodName>
    ) {
        let mut decisions = self.decisions.write().await;
        decisions.push(GovernanceRecord {
            timestamp: Utc::now(),
            situation,
            decision,
            actor,
            executed: false,
            result: None,
        });
        
        // Mantener solo las últimas 1000 decisiones
        if decisions.len() > 1000 {
            decisions.remove(0);
        }
    }
    
    /// Marca una decisión como ejecutada
    pub async fn mark_executed(&self, timestamp: chrono::DateTime<chrono::Utc>, result: String) {
        let mut decisions = self.decisions.write().await;
        if let Some(decision) = decisions.iter_mut().find(|d| d.timestamp == timestamp) {
            decision.executed = true;
            decision.result = Some(result);
        }
    }
    
    /// Obtiene historial de decisiones
    pub async fn get_history(&self, limit: usize) -> Vec<GovernanceRecord> {
        let decisions = self.decisions.read().await;
        decisions.iter().rev().take(limit).cloned().collect()
    }
    
    /// Obtiene umbrales actuales
    pub async fn get_thresholds(&self) -> GovernanceThresholds {
        self.thresholds.read().await.clone()
    }
    
    /// Actualiza umbrales
    pub async fn set_thresholds(&self, thresholds: GovernanceThresholds) {
        let mut t = self.thresholds.write().await;
        *t = thresholds;
        info!("⚡ Zeus: Governance thresholds updated");
    }
    
    // ==================== Feature Flags ====================
    
    /// Crea un feature flag
    pub async fn create_feature_flag(
        &self, 
        name: &str, 
        description: &str, 
        enabled: bool
    ) -> Result<(), String> {
        let mut flags = self.feature_flags.write().await;
        
        if flags.flags.contains_key(name) {
            return Err(format!("Feature flag '{}' already exists", name));
        }
        
        flags.flags.insert(
            name.to_string(), 
            FeatureFlag::new(name, description, enabled)
        );
        
        info!("⚡ Zeus: Feature flag '{}' created (enabled: {})", name, enabled);
        Ok(())
    }
    
    /// Habilita un feature flag
    pub async fn enable_feature_flag(&self, name: &str, modified_by: Option<&str>) -> Result<(), String> {
        let mut flags = self.feature_flags.write().await;
        
        if let Some(flag) = flags.flags.get_mut(name) {
            flag.enabled = true;
            flag.rollout_percentage = 100.0;
            flag.modified_at = Utc::now();
            flag.modified_by = modified_by.map(|s| s.to_string());
            
            info!("⚡ Zeus: Feature flag '{}' enabled by {:?}", name, modified_by);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", name))
        }
    }
    
    /// Deshabilita un feature flag
    pub async fn disable_feature_flag(&self, name: &str, modified_by: Option<&str>) -> Result<(), String> {
        let mut flags = self.feature_flags.write().await;
        
        if let Some(flag) = flags.flags.get_mut(name) {
            flag.enabled = false;
            flag.rollout_percentage = 0.0;
            flag.modified_at = Utc::now();
            flag.modified_by = modified_by.map(|s| s.to_string());
            
            info!("⚡ Zeus: Feature flag '{}' disabled by {:?}", name, modified_by);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", name))
        }
    }
    
    /// Establece rollout gradual
    pub async fn set_rollout_percentage(
        &self, 
        name: &str, 
        percentage: f64,
        modified_by: Option<&str>
    ) -> Result<(), String> {
        let mut flags = self.feature_flags.write().await;
        
        if let Some(flag) = flags.flags.get_mut(name) {
            flag.rollout_percentage = percentage.clamp(0.0, 100.0);
            flag.modified_at = Utc::now();
            flag.modified_by = modified_by.map(|s| s.to_string());
            
            info!("⚡ Zeus: Feature flag '{}' rollout set to {:.1}%", name, percentage);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", name))
        }
    }
    
    /// Verifica si un feature flag está habilitado
    pub async fn is_feature_enabled(&self, name: &str) -> bool {
        let flags = self.feature_flags.read().await;
        flags.flags.get(name).map(|f| f.enabled).unwrap_or(false)
    }
    
    /// Verifica si un feature flag está habilitado para un actor específico
    pub async fn is_feature_enabled_for(&self, name: &str, actor_id: &str) -> bool {
        let flags = self.feature_flags.read().await;
        flags.flags.get(name).map(|f| f.is_enabled_for(actor_id)).unwrap_or(false)
    }
    
    /// Obtiene todos los feature flags
    pub async fn get_all_feature_flags(&self) -> Vec<FeatureFlag> {
        let flags = self.feature_flags.read().await;
        flags.flags.values().cloned().collect()
    }
    
    /// Elimina un feature flag
    pub async fn delete_feature_flag(&self, name: &str) -> Result<(), String> {
        let mut flags = self.feature_flags.write().await;
        
        if flags.flags.remove(name).is_some() {
            info!("⚡ Zeus: Feature flag '{}' deleted", name);
            Ok(())
        } else {
            Err(format!("Feature flag '{}' not found", name))
        }
    }
    
    // ==================== Circuit Breaker ====================
    
    /// Crea un circuit breaker
    pub async fn create_circuit_breaker(
        &self, 
        component: &str, 
        threshold: u64,
        timeout_seconds: u64
    ) -> Result<(), String> {
        let mut breakers = self.circuit_breaker.write().await;
        
        if breakers.breakers.contains_key(component) {
            return Err(format!("Circuit breaker for '{}' already exists", component));
        }
        
        breakers.breakers.insert(
            component.to_string(),
            CircuitBreaker::new(component, threshold, timeout_seconds)
        );
        
        info!("⚡ Zeus: Circuit breaker created for '{}' (threshold: {})", component, threshold);
        Ok(())
    }
    
    /// Registra un fallo en un circuit breaker
    pub async fn record_circuit_failure(&self, component: &str) {
        let mut breakers = self.circuit_breaker.write().await;
        
        if let Some(breaker) = breakers.breakers.get_mut(component) {
            breaker.record_failure();
        }
    }
    
    /// Registra un éxito en un circuit breaker
    pub async fn record_circuit_success(&self, component: &str) {
        let mut breakers = self.circuit_breaker.write().await;
        
        if let Some(breaker) = breakers.breakers.get_mut(component) {
            breaker.record_success();
        }
    }
    
    /// Verifica si se permite una llamada
    pub async fn allow_circuit_request(&self, component: &str) -> bool {
        let mut breakers = self.circuit_breaker.write().await;
        
        if let Some(breaker) = breakers.breakers.get_mut(component) {
            breaker.allow_request()
        } else {
            // Si no existe, permitir
            true
        }
    }
    
    /// Obtiene estado de un circuit breaker
    pub async fn get_circuit_state(&self, component: &str) -> Option<CircuitState> {
        let breakers = self.circuit_breaker.read().await;
        breakers.breakers.get(component).map(|b| b.state.clone())
    }
    
    /// Obtiene todos los circuit breakers
    pub async fn get_all_circuit_breakers(&self) -> Vec<CircuitBreaker> {
        let breakers = self.circuit_breaker.read().await;
        breakers.breakers.values().cloned().collect()
    }
    
    /// Abre manualmente un circuit breaker
    pub async fn open_circuit(&self, component: &str) -> Result<(), String> {
        let mut breakers = self.circuit_breaker.write().await;
        
        if let Some(breaker) = breakers.breakers.get_mut(component) {
            breaker.state = CircuitState::Open;
            breaker.last_failure = Some(Utc::now());
            warn!("⚡ Zeus: Circuit breaker manually OPENED for {}", component);
            Ok(())
        } else {
            Err(format!("Circuit breaker for '{}' not found", component))
        }
    }
    
    /// Cierra manualmente un circuit breaker
    pub async fn close_circuit(&self, component: &str) -> Result<(), String> {
        let mut breakers = self.circuit_breaker.write().await;
        
        if let Some(breaker) = breakers.breakers.get_mut(component) {
            breaker.state = CircuitState::Closed;
            breaker.failure_count = 0;
            info!("⚡ Zeus: Circuit breaker manually CLOSED for {}", component);
            Ok(())
        } else {
            Err(format!("Circuit breaker for '{}' not found", component))
        }
    }
    
    // ==================== Rate Limiting ====================
    
    /// Configura rate limit para un actor
    pub async fn set_actor_rate_limit(
        &self, 
        actor: GodName, 
        requests_per_second: u64,
        burst_size: u64
    ) {
        let mut limits = self.rate_limiter.write().await;
        limits.limits.insert(
            actor,
            ActorRateLimit::new(actor, requests_per_second, burst_size)
        );
        
        info!("⚡ Zeus: Rate limit configured for {:?} (rps: {}, burst: {})", 
            actor, requests_per_second, burst_size);
    }
    
    /// Verifica si se permite una solicitud
    pub async fn allow_rate_limited_request(&self, actor: GodName) -> bool {
        let mut limits = self.rate_limiter.write().await;
        
        if let Some(limit) = limits.limits.get_mut(&actor) {
            limit.allow_request()
        } else {
            // Si no hay límite configurado, permitir
            true
        }
    }
    
    /// Verifica rate limit global
    pub async fn check_global_rate_limit(&self) -> bool {
        let mut limits = self.rate_limiter.write().await;
        
        if let Some(ref mut global) = limits.global_limit {
            let now = Utc::now();
            let elapsed = (now - global.window_start).num_seconds() as u64;
            
            if elapsed >= 1 {
                global.window_start = now;
                global.request_count = 0;
            }
            
            let allowed = global.request_count < (global.requests_per_second + global.burst_size);
            
            if allowed {
                global.request_count += 1;
            }
            
            allowed
        } else {
            true
        }
    }
    
    /// Configura rate limit global
    pub async fn set_global_rate_limit(&self, requests_per_second: u64, burst_size: u64) {
        let mut limits = self.rate_limiter.write().await;
        limits.global_limit = Some(GlobalRateLimit {
            requests_per_second,
            burst_size,
            window_start: Utc::now(),
            request_count: 0,
        });
        
        info!("⚡ Zeus: Global rate limit configured (rps: {}, burst: {})", 
            requests_per_second, burst_size);
    }
    
    /// Obtiene estadísticas de rate limiting
    pub async fn get_rate_limit_stats(&self) -> Vec<RateLimitStats> {
        let limits = self.rate_limiter.read().await;
        limits.limits.values().map(|l| l.get_stats()).collect()
    }
    
    // ==================== Access Policies ====================
    
    /// Crea una política de acceso
    pub async fn create_access_policy(&self, policy: AccessPolicy) -> Result<(), String> {
        let mut policies = self.access_policies.write().await;
        
        if policies.policies.contains_key(&policy.name) {
            return Err(format!("Policy '{}' already exists", policy.name));
        }
        
        policies.policies.insert(policy.name.clone(), policy.clone());
        info!("⚡ Zeus: Access policy '{}' created", policy.name);
        Ok(())
    }
    
    /// Verifica acceso
    pub async fn check_access(
        &self, 
        resource: &str, 
        actor: GodName, 
        role: &str
    ) -> AccessDecision {
        let policies = self.access_policies.read().await;
        
        for policy in policies.policies.values() {
            if policy.resource == resource && policy.enabled {
                // Verificar si el actor está permitido
                if policy.allowed_actors.contains(&actor) {
                    return AccessDecision::Allow;
                }
                
                // Verificar si el rol está permitido
                if policy.allowed_roles.contains(&role.to_string()) {
                    return AccessDecision::Allow;
                }
            }
        }
        
        AccessDecision::Deny
    }
    
    /// Obtiene todas las políticas
    pub async fn get_all_policies(&self) -> Vec<AccessPolicy> {
        let policies = self.access_policies.read().await;
        policies.policies.values().cloned().collect()
    }
}

impl Default for GovernanceController {
    fn default() -> Self {
        Self::new()
    }
}

/// Decisión de acceso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny,
    Conditional(serde_json::Value),
}
