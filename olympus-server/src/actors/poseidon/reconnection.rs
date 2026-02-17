// src/actors/poseidon/reconnection.rs
// OLYMPUS v15 - Poseidon Reconnection Manager Real
// Exponential backoff, circuit breaker y estado persistente

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};


use crate::actors::DivineDomain;

/// Estado del circuit breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,    // Funcionamiento normal
    Open,      // Circuito abierto, rechazando conexiones
    HalfOpen,  // Probando si se puede restaurar
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "closed"),
            CircuitState::Open => write!(f, "open"),
            CircuitState::HalfOpen => write!(f, "half_open"),
        }
    }
}

/// Configuración de política de reconexión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionPolicy {
    /// Retardo inicial en ms
    pub initial_delay_ms: u64,
    /// Retardo máximo en ms
    pub max_delay_ms: u64,
    /// Número máximo de intentos
    pub max_attempts: u32,
    /// Multiplicador de backoff
    pub backoff_multiplier: f64,
    /// Jitter (0.0 - 1.0)
    pub jitter: f64,
    /// Umbral de fallos para abrir circuit breaker
    pub circuit_breaker_threshold: u32,
    /// Tiempo de recuperación del circuit breaker (segundos)
    pub circuit_breaker_recovery_secs: u64,
    /// Habilitar estado persistente
    pub persistent_state: bool,
    /// Guardar estado cada N segundos
    pub state_save_interval_secs: u64,
}

impl Default for ReconnectionPolicy {
    fn default() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            max_attempts: 10,
            backoff_multiplier: 2.0,
            jitter: 0.1,
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery_secs: 60,
            persistent_state: true,
            state_save_interval_secs: 30,
        }
    }
}

impl ReconnectionPolicy {
    /// Calcula el próximo retardo con exponential backoff y jitter
    pub fn next_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(self.initial_delay_ms);
        }

        let delay = self.initial_delay_ms as f64 * (self.backoff_multiplier.powi(attempt as i32));
        let delay = delay.min(self.max_delay_ms as f64);

        // Añadir jitter
        let jitter_amount = delay * self.jitter;
        let jittered = delay + (rand::random::<f64>() * jitter_amount * 2.0 - jitter_amount);

        Duration::from_millis((jittered as u64).max(self.initial_delay_ms))
    }

    /// Verifica si se debe reintentar
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    /// Verifica si el circuit breaker debería abrirse
    pub fn should_open_circuit(&self, consecutive_failures: u32) -> bool {
        consecutive_failures >= self.circuit_breaker_threshold
    }
}

/// Circuit breaker para prevenir loops de reconexión
pub struct CircuitBreaker {
    policy: ReconnectionPolicy,
    state: RwLock<CircuitState>,
    consecutive_failures: AtomicU32,
    consecutive_successes: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    last_state_change: Mutex<Instant>,
    half_open_attempts: AtomicU32,
}

impl CircuitBreaker {
    pub fn new(policy: ReconnectionPolicy) -> Self {
        Self {
            policy,
            state: RwLock::new(CircuitState::Closed),
            consecutive_failures: AtomicU32::new(0),
            consecutive_successes: AtomicU32::new(0),
            last_failure: Mutex::new(None),
            last_state_change: Mutex::new(Instant::now()),
            half_open_attempts: AtomicU32::new(0),
        }
    }

    /// Verifica si se puede intentar una conexión
    pub async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Verificar si es tiempo de intentar half-open
                let last_change = *self.last_state_change.lock().await;
                let recovery_duration = Duration::from_secs(self.policy.circuit_breaker_recovery_secs);

                if last_change.elapsed() >= recovery_duration {
                    let mut state_guard = self.state.write().await;
                    *state_guard = CircuitState::HalfOpen;
                    *self.last_state_change.lock().await = Instant::now();
                    self.half_open_attempts.store(0, Ordering::Relaxed);
                    info!("🔓 Circuit breaker: Intentando half-open");
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Permitir intentos limitados en half-open
                let attempts = self.half_open_attempts.load(Ordering::Relaxed);
                if attempts < 3 {
                    self.half_open_attempts.fetch_add(1, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Registra un éxito
    pub async fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        let successes = self.consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;

        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen && successes >= 3 {
            let mut state_guard = self.state.write().await;
            *state_guard = CircuitState::Closed;
            *self.last_state_change.lock().await = Instant::now();
            info!("✅ Circuit breaker: Cerrado después de éxito en half-open");
        }
    }

    /// Registra un fallo
    pub async fn record_failure(&self) {
        self.consecutive_successes.store(0, Ordering::Relaxed);
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.lock().await = Some(Instant::now());

        if self.policy.should_open_circuit(failures) {
            let mut state_guard = self.state.write().await;
            *state_guard = CircuitState::Open;
            *self.last_state_change.lock().await = Instant::now();
            warn!(
                "🔒 Circuit breaker: Abierto después de {} fallos consecutivos",
                failures
            );
        }
    }

    /// Obtiene el estado actual
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Fuerza el cierre del circuito
    pub async fn force_close(&self) {
        let mut state_guard = self.state.write().await;
        *state_guard = CircuitState::Closed;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        *self.last_state_change.lock().await = Instant::now();
        info!("🔓 Circuit breaker: Forzado a cerrado");
    }

    /// Obtiene estadísticas del circuit breaker
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
            consecutive_successes: self.consecutive_successes.load(Ordering::Relaxed),
            half_open_attempts: self.half_open_attempts.load(Ordering::Relaxed),
        }
    }
}

/// Estadísticas del circuit breaker
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub half_open_attempts: u32,
}

/// Estado de una conexión en proceso de reconexión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionState {
    pub connection_id: String,
    pub url: String,
    pub domain: DivineDomain,
    pub current_attempt: u32,
    pub next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub total_attempts: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub circuit_state: CircuitState,
}

impl ReconnectionState {
    pub fn new(connection_id: String, url: String, domain: DivineDomain) -> Self {
        let now = chrono::Utc::now();
        Self {
            connection_id,
            url,
            domain,
            current_attempt: 0,
            next_attempt_at: None,
            last_error: None,
            total_attempts: 0,
            created_at: now,
            updated_at: now,
            circuit_state: CircuitState::Closed,
        }
    }
}

/// Eventos de reconexión para notificaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconnectionEvent {
    Reconnecting {
        connection_id: String,
        attempt: u32,
        delay_ms: u64,
    },
    Reconnected {
        connection_id: String,
        attempt: u32,
    },
    Failed {
        connection_id: String,
        error: String,
        final_attempt: bool,
    },
    CircuitBreakerOpened {
        connection_id: String,
        failures: u32,
    },
    CircuitBreakerClosed {
        connection_id: String,
    },
    StatePersisted {
        connection_id: String,
    },
}

/// Manager de reconexiones con circuit breaker y estado persistente
pub struct ReconnectionManager {
    policy: ReconnectionPolicy,
    states: Arc<RwLock<HashMap<String, ReconnectionState>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    event_sender: Option<mpsc::Sender<ReconnectionEvent>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    /// Persistencia (almacenamiento externo)
    persistence: Arc<Mutex<Option<Box<dyn ReconnectionPersistence>>>>,
}

/// Trait para persistencia de estado
#[async_trait::async_trait]
pub trait ReconnectionPersistence: Send + Sync {
    async fn save_state(&self, state: &ReconnectionState) -> Result<(), String>;
    async fn load_state(&self, connection_id: &str) -> Result<Option<ReconnectionState>, String>;
    async fn delete_state(&self, connection_id: &str) -> Result<(), String>;
    async fn list_states(&self) -> Result<Vec<ReconnectionState>, String>;
}

impl ReconnectionManager {
    pub fn new(policy: Option<ReconnectionPolicy>) -> Self {
        let policy = policy.unwrap_or_default();

        Self {
            policy,
            states: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            event_sender: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            persistence: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_event_channel(mut self, sender: mpsc::Sender<ReconnectionEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    pub async fn set_persistence(&self, persistence: Box<dyn ReconnectionPersistence>) {
        let mut guard = self.persistence.lock().await;
        *guard = Some(persistence);
    }

    /// Registra una nueva conexión para tracking de reconexión
    pub async fn register_connection(
        &self,
        connection_id: String,
        url: String,
        domain: DivineDomain,
    ) {
        let state = ReconnectionState::new(connection_id.clone(), url, domain);
        let breaker = Arc::new(CircuitBreaker::new(self.policy.clone()));

        {
            let mut states = self.states.write().await;
            states.insert(connection_id.clone(), state);
        }

        {
            let mut breakers = self.circuit_breakers.write().await;
            breakers.insert(connection_id.clone(), breaker);
        }

        // Persistir si está habilitado
        if self.policy.persistent_state {
            self.persist_state(&connection_id).await;
        }

        info!("📝 ReconnectionManager: Conexión {} registrada", connection_id);
    }

    /// Inicia el proceso de reconexión
    pub async fn start_reconnection(
        &self,
        connection_id: &str,
        error: Option<String>,
    ) -> Result<ReconnectionPlan, ReconnectionError> {
        // Actualizar estado
        {
            let mut states = self.states.write().await;
            if let Some(state) = states.get_mut(connection_id) {
                state.current_attempt += 1;
                state.total_attempts += 1;
                state.last_error = error;
                state.updated_at = chrono::Utc::now();

                // Verificar circuit breaker
                let breaker = {
                    let breakers = self.circuit_breakers.read().await;
                    breakers.get(connection_id).cloned()
                };

                if let Some(ref cb) = breaker {
                    if !cb.can_execute().await {
                        state.circuit_state = CircuitState::Open;
                        return Err(ReconnectionError::CircuitOpen);
                    }
                    state.circuit_state = cb.state().await;
                }

                // Calcular próximo intento
                let delay = self.policy.next_delay(state.current_attempt);
                state.next_attempt_at = Some(chrono::Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default());

                // Notificar
                if let Some(ref sender) = self.event_sender {
                    let _ = sender
                        .send(ReconnectionEvent::Reconnecting {
                            connection_id: connection_id.to_string(),
                            attempt: state.current_attempt,
                            delay_ms: delay.as_millis() as u64,
                        })
                        .await;
                }

                return Ok(ReconnectionPlan {
                    connection_id: connection_id.to_string(),
                    attempt: state.current_attempt,
                    delay,
                    max_attempts: self.policy.max_attempts,
                });
            }
        }

        Err(ReconnectionError::ConnectionNotFound)
    }

    /// Espera y ejecuta reconexión
    pub async fn execute_reconnection<F, Fut>(
        &self,
        connection_id: &str,
        connect_fn: F,
    ) -> Result<(), ReconnectionError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        // Obtener plan de reconexión
        let plan = self.start_reconnection(connection_id, None).await?;

        info!(
            "🔄 ReconnectionManager: Reintentando {} en {:?} (intento {}/{})",
            connection_id, plan.delay, plan.attempt, plan.max_attempts
        );

        // Esperar el retardo
        sleep(plan.delay).await;

        // Intentar conexión
        match connect_fn().await {
            Ok(()) => {
                self.handle_success(connection_id).await;
                Ok(())
            }
            Err(err) => {
                self.handle_failure(connection_id, &err).await;
                Err(ReconnectionError::ConnectionFailed(err))
            }
        }
    }

    /// Maneja un éxito de reconexión
    async fn handle_success(&self, connection_id: &str) {
        // Actualizar estado
        {
            let mut states = self.states.write().await;
            if let Some(state) = states.get_mut(connection_id) {
                state.current_attempt = 0;
                state.next_attempt_at = None;
                state.last_error = None;
                state.updated_at = chrono::Utc::now();
            }
        }

        // Notificar circuit breaker
        {
            let breakers = self.circuit_breakers.read().await;
            if let Some(breaker) = breakers.get(connection_id) {
                breaker.record_success().await;
            }
        }

        // Notificar evento
        if let Some(ref sender) = self.event_sender {
            let _ = sender
                .send(ReconnectionEvent::Reconnected {
                    connection_id: connection_id.to_string(),
                    attempt: 0,
                })
                .await;
        }

        // Persistir
        if self.policy.persistent_state {
            self.persist_state(connection_id).await;
        }

        info!("✅ ReconnectionManager: Conexión {} restaurada", connection_id);
    }

    /// Maneja un fallo de reconexión
    async fn handle_failure(&self, connection_id: &str, error: &str) {
        // Actualizar estado
        let final_attempt = {
            let mut states = self.states.write().await;
            if let Some(state) = states.get_mut(connection_id) {
                state.last_error = Some(error.to_string());
                state.updated_at = chrono::Utc::now();
                state.current_attempt >= self.policy.max_attempts
            } else {
                false
            }
        };

        // Notificar circuit breaker
        {
            let breakers = self.circuit_breakers.read().await;
            if let Some(breaker) = breakers.get(connection_id) {
                breaker.record_failure().await;
            }
        }

        // Notificar evento
        if let Some(ref sender) = self.event_sender {
            let _ = sender
                .send(ReconnectionEvent::Failed {
                    connection_id: connection_id.to_string(),
                    error: error.to_string(),
                    final_attempt,
                })
                .await;
        }

        // Persistir
        if self.policy.persistent_state {
            self.persist_state(connection_id).await;
        }

        warn!(
            "❌ ReconnectionManager: Fallo en {}: {}",
            connection_id, error
        );
    }

    /// Registra una desconexión y prepara para reconexión
    pub async fn handle_disconnection(
        &self,
        connection_id: &str,
        error: Option<String>,
    ) -> bool {
        let should_reconnect = {
            let states = self.states.read().await;
            if let Some(state) = states.get(connection_id) {
                state.current_attempt < self.policy.max_attempts
            } else {
                false
            }
        };

        if should_reconnect {
            // Actualizar estado
            {
                let mut states = self.states.write().await;
                if let Some(state) = states.get_mut(connection_id) {
                    state.last_error = error.clone();
                    state.updated_at = chrono::Utc::now();
                }
            }

            info!(
                "🔌 ReconnectionManager: Conexión {} desconectada, preparando reconexión",
                connection_id
            );
            true
        } else {
            warn!(
                "🔌 ReconnectionManager: Conexión {} desconectada, máximos intentos alcanzados",
                connection_id
            );
            false
        }
    }

    /// Obtiene el estado de una conexión
    pub async fn get_state(&self, connection_id: &str) -> Option<ReconnectionState> {
        let states = self.states.read().await;
        states.get(connection_id).cloned()
    }

    /// Obtiene todos los estados
    pub async fn get_all_states(&self) -> Vec<ReconnectionState> {
        let states = self.states.read().await;
        states.values().cloned().collect()
    }

    /// Obtiene el estado del circuit breaker
    pub async fn get_circuit_state(&self, connection_id: &str) -> Option<CircuitState> {
        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(connection_id) {
            Some(breaker.state().await)
        } else {
            None
        }
    }

    /// Fuerza el cierre del circuit breaker
    pub async fn force_close_circuit(&self, connection_id: &str) {
        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(connection_id) {
            breaker.force_close().await;
        }
    }

    /// Elimina una conexión del tracking
    pub async fn unregister_connection(&self, connection_id: &str) {
        {
            let mut states = self.states.write().await;
            states.remove(connection_id);
        }

        {
            let mut breakers = self.circuit_breakers.write().await;
            breakers.remove(connection_id);
        }

        // Eliminar de persistencia
        if self.policy.persistent_state {
            self.delete_persistent_state(connection_id).await;
        }

        info!("🗑️ ReconnectionManager: Conexión {} eliminada", connection_id);
    }

    /// Persiste el estado
    async fn persist_state(&self, connection_id: &str) {
        let persistence = self.persistence.lock().await;
        if let Some(ref store) = *persistence {
            if let Some(state) = self.get_state(connection_id).await {
                if let Err(e) = store.save_state(&state).await {
                    error!("💾 Error persistiendo estado de {}: {}", connection_id, e);
                } else {
                    debug!("💾 Estado de {} persistido", connection_id);

                    if let Some(ref sender) = self.event_sender {
                        let _ = sender
                            .send(ReconnectionEvent::StatePersisted {
                                connection_id: connection_id.to_string(),
                            })
                            .await;
                    }
                }
            }
        }
    }

    /// Elimina estado persistente
    async fn delete_persistent_state(&self, connection_id: &str) {
        let persistence = self.persistence.lock().await;
        if let Some(ref store) = *persistence {
            if let Err(e) = store.delete_state(connection_id).await {
                error!("💾 Error eliminando estado de {}: {}", connection_id, e);
            }
        }
    }

    /// Carga estados persistentes
    pub async fn load_persistent_states(&self) -> Vec<ReconnectionState> {
        let persistence = self.persistence.lock().await;
        if let Some(ref store) = *persistence {
            match store.list_states().await {
                Ok(states) => {
                    // Restaurar en memoria
                    let mut mem_states = self.states.write().await;
                    let mut mem_breakers = self.circuit_breakers.write().await;

                    for state in &states {
                        let breaker = Arc::new(CircuitBreaker::new(self.policy.clone()));
                        mem_states.insert(state.connection_id.clone(), state.clone());
                        mem_breakers.insert(state.connection_id.clone(), breaker);
                    }

                    info!("💾 {} estados persistentes cargados", states.len());
                    states
                }
                Err(e) => {
                    error!("💾 Error cargando estados: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    /// Inicia el loop de persistencia automática
    pub async fn start_persistence_loop(&self) {
        if !self.policy.persistent_state {
            return;
        }

        let states = self.states.clone();
        let persistence = self.persistence.clone();
        let running = self.running.clone();
        let interval_secs = self.policy.state_save_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            while running.load(Ordering::Relaxed) {
                ticker.tick().await;

                let persistence_guard = persistence.lock().await;
                if let Some(ref store) = *persistence_guard {
                    let states_guard = states.read().await;
                    for (id, state) in states_guard.iter() {
                        if let Err(e) = store.save_state(state).await {
                            error!("💾 Error auto-guardando {}: {}", id, e);
                        }
                    }
                }
            }
        });
    }

    /// Detiene el manager
    pub async fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);

        // Persistir todos los estados finales
        if self.policy.persistent_state {
            let states = self.states.read().await;
            for id in states.keys() {
                self.persist_state(id).await;
            }
        }

        info!("🛑 ReconnectionManager detenido");
    }

    /// Verifica si está ejecutándose
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Clone for ReconnectionManager {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone(),
            states: self.states.clone(),
            circuit_breakers: self.circuit_breakers.clone(),
            event_sender: None, // Los clones no heredan el sender
            running: self.running.clone(),
            persistence: self.persistence.clone(),
        }
    }
}

/// Plan de reconexión
#[derive(Debug, Clone)]
pub struct ReconnectionPlan {
    pub connection_id: String,
    pub attempt: u32,
    pub delay: Duration,
    pub max_attempts: u32,
}

/// Errores del Reconnection Manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconnectionError {
    ConnectionNotFound,
    CircuitOpen,
    ConnectionFailed(String),
    MaxAttemptsReached,
    InvalidState,
}

impl std::fmt::Display for ReconnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconnectionError::ConnectionNotFound => write!(f, "Connection not found"),
            ReconnectionError::CircuitOpen => write!(f, "Circuit breaker is open"),
            ReconnectionError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            ReconnectionError::MaxAttemptsReached => write!(f, "Maximum reconnection attempts reached"),
            ReconnectionError::InvalidState => write!(f, "Invalid reconnection state"),
        }
    }
}

impl std::error::Error for ReconnectionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnection_policy_default() {
        let policy = ReconnectionPolicy::default();
        assert_eq!(policy.max_attempts, 10);
        assert!(policy.jitter >= 0.0 && policy.jitter <= 1.0);
    }

    #[test]
    fn test_next_delay_exponential() {
        let policy = ReconnectionPolicy::default();

        let delay0 = policy.next_delay(0);
        let delay1 = policy.next_delay(1);
        let delay2 = policy.next_delay(2);

        assert_eq!(delay0.as_millis() as u64, policy.initial_delay_ms);
        assert!(delay1 > delay0);
        assert!(delay2 > delay1);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let policy = ReconnectionPolicy {
            circuit_breaker_threshold: 3,
            ..Default::default()
        };

        let cb = CircuitBreaker::new(policy);

        assert!(cb.can_execute().await);
        assert_eq!(cb.state().await, CircuitState::Closed);

        // Registrar fallos
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.can_execute().await);

        cb.record_failure().await; // Tercer fallo abre el circuito
        assert_eq!(cb.state().await, CircuitState::Open);
        assert!(!cb.can_execute().await);

        // Forzar cierre
        cb.force_close().await;
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_reconnection_state() {
        let state = ReconnectionState::new(
            "test-123".to_string(),
            "ws://localhost:8080".to_string(),
            DivineDomain::DataFlow,
        );

        assert_eq!(state.connection_id, "test-123");
        assert_eq!(state.current_attempt, 0);
        assert_eq!(state.circuit_state, CircuitState::Closed);
    }
}
