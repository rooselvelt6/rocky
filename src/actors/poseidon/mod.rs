// src/actors/poseidon/mod.rs
// OLYMPUS v15 - Poseidon: Señor del Flujo de Datos
// WebSocket real con tokio-tungstenite, flow control y reconnection

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tracing::{debug, error, info, warn};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, ResponsePayload, QueryPayload, EventPayload};
use crate::infrastructure::ValkeyStore;
use crate::errors::ActorError;

pub mod websocket;
pub mod buffer;
pub mod async_writer;
pub mod flow_control;
pub mod reconnection;

pub use websocket::{
    WebSocketManager, ConnectionInfo, ConnectionStatus, WebSocketConfig, WebSocketStats,
    ConnectionEvent, WebSocketError, MessageCallback
};
pub use buffer::EmergencyBuffer;
pub use async_writer::AsyncWriter;
pub use flow_control::{
    FlowController, FlowMetrics, FlowConfig, FlowError, FlowPermit, RateLimitAlgorithm
};
pub use reconnection::{
    ReconnectionManager, ReconnectionPolicy, ReconnectionState, ReconnectionEvent,
    CircuitState, CircuitBreakerStats, ReconnectionError, ReconnectionPlan
};

/// Comandos de Poseidon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonCommand {
    /// Conectar a un WebSocket remoto
    Connect { url: String, domain: DivineDomain },
    /// Desconectar un WebSocket específico
    Disconnect { connection_id: String },
    /// Enviar mensaje a una conexión específica
    Send { connection_id: String, message: String },
    /// Enviar mensaje binario
    SendBinary { connection_id: String, data: Vec<u8> },
    /// Broadcast a todas las conexiones de un dominio
    Broadcast { domain: DivineDomain, message: String },
    /// Broadcast a todas las conexiones
    BroadcastAll { message: String },
    /// Flush del buffer de emergencia
    FlushBuffer,
    /// Configurar flow control
    ConfigureFlow { config: FlowConfig },
    /// Forzar cierre de circuit breaker
    ForceCloseCircuit { connection_id: String },
    /// Limpiar buffer
    ClearBuffer,
    /// Cerrar todas las conexiones
    CloseAll,
}

/// Queries de Poseidon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonQuery {
    /// Obtener todas las conexiones
    GetConnections,
    /// Obtener conexiones de un dominio específico
    GetConnectionsByDomain { domain: DivineDomain },
    /// Obtener detalles de una conexión específica
    GetConnectionDetails { connection_id: String },
    /// Obtener estadísticas del WebSocket Manager
    GetWebSocketStats,
    /// Obtener estadísticas de Flow Control
    GetFlowStats,
    /// Obtener estado de reconexión
    GetReconnectionState { connection_id: String },
    /// Obtener todos los estados de reconexión
    GetAllReconnectionStates,
    /// Obtener estado del circuit breaker
    GetCircuitState { connection_id: String },
    /// Obtener métricas combinadas
    GetAllMetrics,
    /// Verificar health check
    HealthCheck,
}

/// Eventos de Poseidon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonEvent {
    ConnectionOpened { connection_id: String, url: String },
    ConnectionClosed { connection_id: String, reason: String },
    MessageReceived { connection_id: String, size: usize },
    MessageSent { connection_id: String, size: usize },
    BackpressureActivated { level: f64 },
    CircuitBreakerOpened { connection_id: String },
    ReconnectionStarted { connection_id: String, attempt: u32 },
    ReconnectionFailed { connection_id: String, error: String },
    Error { error: String },
}

/// Poseidon: Dios del Flujo de Datos
pub struct Poseidon {
    name: GodName,
    state: ActorState,
    config: ActorConfig,
    
    // Core components
    ws_manager: Arc<RwLock<WebSocketManager>>,
    buffer: Arc<EmergencyBuffer>,
    async_writer: Arc<AsyncWriter>,
    flow_controller: Arc<FlowController>,
    reconnection_manager: Arc<ReconnectionManager>,
    
    // Valkey for buffer
    valkey: Arc<ValkeyStore>,
    
    // Channels
    command_tx: mpsc::Sender<PoseidonCommand>,
    command_rx: Option<mpsc::Receiver<PoseidonCommand>>,
    
    // Message callback
    message_callback: Arc<RwLock<Option<Arc<dyn Fn(String, String) + Send + Sync>>>>,
    
    // Running state
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl Poseidon {
    pub async fn new(valkey: Arc<ValkeyStore>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(1000);
        
        // Crear WebSocket manager
        let ws_manager = Arc::new(RwLock::new(WebSocketManager::new(None)));
        
        // Crear flow controller
        let flow_controller = Arc::new(FlowController::new(None));
        flow_controller.start_metrics_loop();
        
        // Crear reconnection manager
        let reconnection_manager = Arc::new(ReconnectionManager::new(None));
        reconnection_manager.start_persistence_loop().await;
        
        Self {
            name: GodName::Poseidon,
            state: ActorState::new(GodName::Poseidon),
            config: ActorConfig::default(),
            
            ws_manager,
            buffer: Arc::new(EmergencyBuffer::new(valkey.clone())),
            async_writer: Arc::new(AsyncWriter::new()),
            flow_controller,
            reconnection_manager,
            
            valkey,
            
            command_tx,
            command_rx: Some(command_rx),
            
            message_callback: Arc::new(RwLock::new(None)),
            
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Obtiene el sender de comandos
    pub fn command_sender(&self) -> mpsc::Sender<PoseidonCommand> {
        self.command_tx.clone()
    }

    /// Configura el callback para mensajes entrantes
    pub async fn set_message_callback<F>(&self, callback: F)
    where
        F: Fn(String, String) + Send + Sync + 'static,
    {
        let callback = Arc::new(callback);
        let mut guard = self.message_callback.write().await;
        *guard = Some(callback.clone());

        // Configurar callback en WebSocket manager
        let ws_manager = self.ws_manager.read().await;
        let msg_callback: MessageCallback = Arc::new(move |conn_id, msg| {
            if let TungsteniteMessage::Text(text) = msg {
                callback(conn_id, text);
            }
        });
        ws_manager.set_message_callback(msg_callback).await;
    }

    /// Conecta a un WebSocket remoto
    pub async fn connect(&self, url: &str, domain: DivineDomain) -> Result<String, WebSocketError> {
        // Verificar flow control
        if let Err(e) = self.flow_controller.acquire_permit().await {
            return Err(WebSocketError::ConnectionFailed(e.to_string()));
        }

        let ws_manager = self.ws_manager.read().await;
        let connection_id = ws_manager.connect(url, domain.clone()).await?;

        // Registrar en reconnection manager
        self.reconnection_manager
            .register_connection(connection_id.clone(), url.to_string(), domain)
            .await;

        info!("🌊 Poseidon: Conectado a {} (ID: {})", url, connection_id);

        Ok(connection_id)
    }

    /// Desconecta una conexión específica
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), WebSocketError> {
        let ws_manager = self.ws_manager.read().await;
        ws_manager.disconnect(connection_id).await?;

        // Eliminar de reconnection manager
        self.reconnection_manager.unregister_connection(connection_id).await;

        info!("🌊 Poseidon: Desconectado {}", connection_id);

        Ok(())
    }

    /// Envía un mensaje a una conexión
    pub async fn send(&self, connection_id: &str, message: &str) -> Result<(), PoseidonError> {
        // Verificar flow control
        let _permit = self.flow_controller.acquire_permit().await
            .map_err(|e| PoseidonError::FlowControl(e.to_string()))?;

        let ws_manager = self.ws_manager.read().await;
        ws_manager.send_text(connection_id, message.to_string()).await
            .map_err(|e| PoseidonError::WebSocket(e.to_string()))?;

        _permit.record_usage(message.len());

        Ok(())
    }

    /// Envía mensaje binario
    pub async fn send_binary(&self, connection_id: &str, data: Vec<u8>) -> Result<(), PoseidonError> {
        let _permit = self.flow_controller.acquire_permit().await
            .map_err(|e| PoseidonError::FlowControl(e.to_string()))?;

        let ws_manager = self.ws_manager.read().await;
        ws_manager.send_binary(connection_id, data.clone()).await
            .map_err(|e| PoseidonError::WebSocket(e.to_string()))?;

        _permit.record_usage(data.len());

        Ok(())
    }

    /// Broadcast a dominio específico
    pub async fn broadcast(&self, domain: DivineDomain, message: &str) -> Vec<(String, Result<(), PoseidonError>)> {
        let ws_manager = self.ws_manager.read().await;
        let results = ws_manager.broadcast_to_domain(domain, message.to_string()).await;

        results.into_iter()
            .map(|(id, result)| (id, result.map_err(|e| PoseidonError::WebSocket(e.to_string()))))
            .collect()
    }

    /// Broadcast a todas las conexiones
    pub async fn broadcast_all(&self, message: &str) -> Vec<(String, Result<(), PoseidonError>)> {
        let ws_manager = self.ws_manager.read().await;
        let results = ws_manager.broadcast_all(message.to_string()).await;

        results.into_iter()
            .map(|(id, result)| (id, result.map_err(|e| PoseidonError::WebSocket(e.to_string()))))
            .collect()
    }

    /// Obtiene todas las conexiones
    pub async fn get_connections(&self) -> Vec<ConnectionInfo> {
        let ws_manager = self.ws_manager.read().await;
        ws_manager.get_all_connections().await
    }

    /// Obtiene detalles de una conexión
    pub async fn get_connection(&self, connection_id: &str) -> Option<ConnectionInfo> {
        let ws_manager = self.ws_manager.read().await;
        ws_manager.get_connection(connection_id).await
    }

    /// Obtiene estadísticas de WebSocket
    pub async fn get_websocket_stats(&self) -> WebSocketStats {
        let ws_manager = self.ws_manager.read().await;
        ws_manager.get_stats().await
    }

    /// Obtiene estadísticas de flow control
    pub async fn get_flow_stats(&self) -> FlowMetrics {
        self.flow_controller.get_metrics().await
    }

    /// Obtiene estados de reconexión
    pub async fn get_reconnection_states(&self) -> Vec<ReconnectionState> {
        self.reconnection_manager.get_all_states().await
    }

    /// Verifica backpressure
    pub async fn check_backpressure(&self) -> (bool, f64) {
        let metrics = self.flow_controller.get_metrics().await;
        (metrics.backpressure_active, metrics.backpressure_level)
    }

    /// Cierra todas las conexiones
    pub async fn close_all(&self) {
        let ws_manager = self.ws_manager.read().await;
        ws_manager.close_all().await;
        info!("🌊 Poseidon: Todas las conexiones cerradas");
    }

    /// Inicia el loop de procesamiento de mensajes WebSocket
    pub fn start_message_loop(&self) {
        let flow_controller = self.flow_controller.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                // Verificar backpressure periódicamente
                let metrics = flow_controller.get_metrics().await;
                if metrics.backpressure_active {
                    warn!("🌊 Poseidon: Backpressure activo ({:.1}%)", metrics.backpressure_level * 100.0);
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    /// Inicia el loop de comandos
    pub fn start_command_loop(&mut self) {
        if let Some(mut command_rx) = self.command_rx.take() {
            let ws_manager = self.ws_manager.clone();
            let reconnection_manager = self.reconnection_manager.clone();
            let flow_controller = self.flow_controller.clone();
            let running = self.running.clone();

            tokio::spawn(async move {
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    match timeout(Duration::from_secs(1), command_rx.recv()).await {
                        Ok(Some(cmd)) => {
                            match cmd {
                                PoseidonCommand::Connect { url, domain } => {
                                    let ws = ws_manager.read().await;
                                    if let Err(e) = ws.connect(&url, domain).await {
                                        error!("Error conectando a {}: {}", url, e);
                                    }
                                }
                                PoseidonCommand::Disconnect { connection_id } => {
                                    let ws = ws_manager.read().await;
                                    if let Err(e) = ws.disconnect(&connection_id).await {
                                        error!("Error desconectando {}: {}", connection_id, e);
                                    }
                                }
                                PoseidonCommand::Send { connection_id, message } => {
                                    let ws = ws_manager.read().await;
                                    if let Err(e) = ws.send_text(&connection_id, message).await {
                                        error!("Error enviando a {}: {}", connection_id, e);
                                    }
                                }
                                PoseidonCommand::Broadcast { domain, message } => {
                                    let ws = ws_manager.read().await;
                                    let _ = ws.broadcast_to_domain(domain, message).await;
                                }
                                PoseidonCommand::BroadcastAll { message } => {
                                    let ws = ws_manager.read().await;
                                    let _ = ws.broadcast_all(message).await;
                                }
                                PoseidonCommand::FlushBuffer => {
                                    debug!("Flushing buffer...");
                                }
                                PoseidonCommand::ConfigureFlow { config } => {
                                    let _ = config;
                                    debug!("ConfigureFlow command received");
                                }
                                PoseidonCommand::ForceCloseCircuit { connection_id } => {
                                    reconnection_manager.force_close_circuit(&connection_id).await;
                                }
                                PoseidonCommand::ClearBuffer => {
                                    flow_controller.clear_buffer().await;
                                }
                                PoseidonCommand::CloseAll => {
                                    let ws = ws_manager.read().await;
                                    ws.close_all().await;
                                }
                                _ => {}
                            }
                        }
                        Ok(None) => break,
                        Err(_) => continue,
                    }
                }
            });
        }
    }
}

#[async_trait]
impl OlympianActor for Poseidon {
    fn name(&self) -> GodName {
        GodName::Poseidon
    }
    
    fn domain(&self) -> DivineDomain {
        DivineDomain::DataFlow
    }
    
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<ResponsePayload, ActorError> {
        self.state.message_count += 1;
        self.state.last_message_time = chrono::Utc::now();
        
        match msg.payload {
            MessagePayload::Command(cmd) => self.handle_command(cmd).await,
            MessagePayload::Query(query) => self.handle_query(query).await,
            MessagePayload::Event(event) => self.handle_event(event).await,
            MessagePayload::Response(_) => Ok(ResponsePayload::Ack { message_id: msg.id }),
        }
    }
    
    async fn persistent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Poseidon",
            "active_connections": 0,
            "buffered_messages": 0,
        })
    }
    
    fn load_state(&mut self, _state: &serde_json::Value) -> Result<(), ActorError> {
        Ok(())
    }
    
    fn heartbeat(&self) -> GodHeartbeat {
        GodHeartbeat {
            god: GodName::Poseidon,
            status: ActorStatus::Healthy,
            last_seen: chrono::Utc::now(),
            load: 0.0,
            memory_usage_mb: 0.0,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
        }
    }
    
    async fn health_check(&self) -> HealthStatus {
        let ws_stats = self.get_websocket_stats().await;
        let flow_metrics = self.get_flow_stats().await;
        
        let status = if flow_metrics.backpressure_level > 0.95 {
            ActorStatus::Degraded
        } else if flow_metrics.backpressure_level > 0.8 {
            ActorStatus::Degraded
        } else {
            ActorStatus::Healthy
        };

        HealthStatus {
            god: GodName::Poseidon,
            status,
            uptime_seconds: (chrono::Utc::now() - self.state.start_time).num_seconds() as u64,
            message_count: self.state.message_count,
            error_count: self.state.error_count,
            last_error: self.state.last_error.as_ref().map(|e| e.to_string()),
            memory_usage_mb: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn config(&self) -> Option<&ActorConfig> {
        Some(&self.config)
    }
    
    async fn initialize(&mut self) -> Result<(), ActorError> {
        info!("🌊 Poseidon: Inicializando sistema de flujo de datos v15...");
        
        self.start_message_loop();
        self.start_command_loop();
        
        info!("🌊 Poseidon: Sistema de flujo de datos inicializado");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), ActorError> {
        info!("🌊 Poseidon: Iniciando shutdown...");
        
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        
        self.close_all().await;
        
        self.reconnection_manager.stop().await;
        
        self.flow_controller.stop();
        
        info!("🌊 Poseidon: Shutdown completado");
        Ok(())
    }
    
    fn actor_state(&self) -> ActorState {
        self.state.clone()
    }
}

impl Poseidon {
    async fn handle_command(&mut self, cmd: CommandPayload) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Connect { url } => {
                let domain = DivineDomain::DataFlow;
                match self.connect(&url, domain.clone()).await {
                    Ok(id) => {
                        let _ = self.command_tx.send(PoseidonCommand::Connect { url, domain }).await;
                        Ok(ResponsePayload::Success { message: format!("Connected: {}", id) })
                    }
                    Err(e) => Ok(ResponsePayload::Error { error: e.to_string(), code: 500 }),
                }
            }
            CommandPayload::Disconnect { connection_id } => {
                match self.disconnect(&connection_id).await {
                    Ok(_) => Ok(ResponsePayload::Success { message: "Disconnected".to_string() }),
                    Err(e) => Ok(ResponsePayload::Error { error: e.to_string(), code: 500 }),
                }
            }
            CommandPayload::Configure { config } => {
                info!("🌊 Poseidon: Configurando con {:?}", config);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Error { error: "Unknown command".to_string(), code: 400 }),
        }
    }
    
    async fn handle_query(&self, query: QueryPayload) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::HealthStatus => {
                let health = self.health_check().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(health).unwrap() })
            }
            QueryPayload::Metrics => {
                let ws_stats = self.get_websocket_stats().await;
                let flow_stats = self.get_flow_stats().await;
                
                let data = json!({
                    "websocket": ws_stats,
                    "flow_control": flow_stats,
                    "uptime_seconds": (chrono::Utc::now() - self.state.start_time).num_seconds(),
                    "total_messages": self.state.message_count,
                });
                
                Ok(ResponsePayload::Data { data })
            }
            QueryPayload::ActorState => {
                let connections = self.get_connections().await;
                Ok(ResponsePayload::Data { data: json!(connections) })
            }
            _ => {
                let ws_stats = self.get_websocket_stats().await;
                Ok(ResponsePayload::Data { data: serde_json::to_value(ws_stats).unwrap() })
            }
        }
    }
    
    async fn handle_event(&mut self, event: EventPayload) -> Result<ResponsePayload, ActorError> {
        match event {
            EventPayload::DataReceived { source, data_type } => {
                debug!("🌊 Poseidon: Datos recibidos de {:?}: {}", source, data_type);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            EventPayload::ErrorOccurred { error, actor } => {
                error!("🌊 Poseidon: Error en {:?}: {}", actor, error);
                Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() })
            }
            _ => Ok(ResponsePayload::Ack { message_id: uuid::Uuid::new_v4().to_string() }),
        }
    }
}

/// Errores de Poseidon
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoseidonError {
    WebSocket(String),
    FlowControl(String),
    ConnectionNotFound(String),
    ReconnectionFailed(String),
    BufferFull,
    Timeout,
}

impl std::fmt::Display for PoseidonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoseidonError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            PoseidonError::FlowControl(msg) => write!(f, "Flow control error: {}", msg),
            PoseidonError::ConnectionNotFound(id) => write!(f, "Connection not found: {}", id),
            PoseidonError::ReconnectionFailed(msg) => write!(f, "Reconnection failed: {}", msg),
            PoseidonError::BufferFull => write!(f, "Buffer is full"),
            PoseidonError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for PoseidonError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_command_serialization() {
        let cmd = PoseidonCommand::Connect {
            url: "ws://localhost:8080".to_string(),
            domain: DivineDomain::DataFlow,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("Connect"));
    }

    #[test]
    fn test_poseidon_query_serialization() {
        let query = PoseidonQuery::GetConnections;
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("GetConnections"));
    }
}
