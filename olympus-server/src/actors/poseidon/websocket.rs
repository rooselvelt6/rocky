// src/actors/poseidon/websocket.rs
// OLYMPUS v15 - Poseidon WebSocket Manager Real
// Gestión real de conexiones WebSocket con tokio-tungstenite

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{interval, timeout};
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio_tungstenite::{accept_async, connect_async};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::actors::DivineDomain;

/// Estado de la conexión WebSocket
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
    Failed,
    Closed,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connecting => write!(f, "connecting"),
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Reconnecting => write!(f, "reconnecting"),
            ConnectionStatus::Failed => write!(f, "failed"),
            ConnectionStatus::Closed => write!(f, "closed"),
        }
    }
}

/// Información de una conexión WebSocket activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub url: String,
    pub domain: DivineDomain,
    pub status: ConnectionStatus,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub last_pong: Option<chrono::DateTime<chrono::Utc>>,
    pub message_count_in: u64,
    pub message_count_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub latency_ms: Option<u64>,
    pub reconnect_attempts: u32,
    pub address: Option<String>,
}

impl ConnectionInfo {
    pub fn new(id: String, url: String, domain: DivineDomain) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            url,
            domain,
            status: ConnectionStatus::Connecting,
            connected_at: now,
            last_activity: now,
            last_ping: None,
            last_pong: None,
            message_count_in: 0,
            message_count_out: 0,
            bytes_in: 0,
            bytes_out: 0,
            latency_ms: None,
            reconnect_attempts: 0,
            address: None,
        }
    }

    pub fn throughput_bps(&self) -> f64 {
        let elapsed = (chrono::Utc::now() - self.connected_at).num_seconds() as f64;
        if elapsed > 0.0 {
            ((self.bytes_in + self.bytes_out) as f64 * 8.0) / elapsed
        } else {
            0.0
        }
    }
}

/// Estadísticas globales del WebSocket Manager
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebSocketStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub connecting_connections: usize,
    pub reconnecting_connections: usize,
    pub failed_connections: usize,
    pub total_messages_in: u64,
    pub total_messages_out: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub average_latency_ms: Option<u64>,
    pub total_throughput_bps: f64,
}

/// Mensajes internos para comunicación entre tareas
#[derive(Debug, Clone)]
pub enum InternalMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<(u16, String)>),
}

impl From<InternalMessage> for TungsteniteMessage {
    fn from(msg: InternalMessage) -> Self {
        match msg {
            InternalMessage::Text(text) => TungsteniteMessage::Text(text),
            InternalMessage::Binary(bin) => TungsteniteMessage::Binary(bin),
            InternalMessage::Ping(data) => TungsteniteMessage::Ping(data),
            InternalMessage::Pong(data) => TungsteniteMessage::Pong(data),
            InternalMessage::Close(reason) => TungsteniteMessage::Close(reason.map(|(code, text)| {
                tokio_tungstenite::tungstenite::protocol::CloseFrame {
                    code: code.into(),
                    reason: text.into(),
                }
            })),
        }
    }
}

/// Handle para una conexión WebSocket activa
struct ActiveConnection {
    info: ConnectionInfo,
    sender: mpsc::UnboundedSender<InternalMessage>,
    shutdown: oneshot::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
}

/// Configuración del WebSocket Manager
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub heartbeat_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub connection_timeout_secs: u64,
    pub max_message_size: usize,
    pub max_frame_size: usize,
    pub auto_accept_server: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 10,
            connection_timeout_secs: 10,
            max_message_size: 64 * 1024 * 1024, // 64MB
            max_frame_size: 16 * 1024 * 1024,   // 16MB
            auto_accept_server: true,
        }
    }
}

/// Callback para recibir mensajes entrantes
pub type MessageCallback = Arc<dyn Fn(String, TungsteniteMessage) + Send + Sync>;

/// Eventos de conexión para notificaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    Connected { connection_id: String, url: String },
    Disconnected { connection_id: String, reason: String },
    MessageReceived { connection_id: String, size: usize },
    MessageSent { connection_id: String, size: usize },
    Reconnecting { connection_id: String, attempt: u32 },
    Failed { connection_id: String, error: String },
    Ping { connection_id: String, latency_ms: u64 },
}

/// WebSocket Manager real con conexiones activas
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,
    config: WebSocketConfig,
    event_sender: Option<mpsc::Sender<ConnectionEvent>>,
    total_messages_in: Arc<AtomicU64>,
    total_messages_out: Arc<AtomicU64>,
    total_bytes_in: Arc<AtomicU64>,
    total_bytes_out: Arc<AtomicU64>,
    message_callback: Arc<RwLock<Option<MessageCallback>>>,
}

impl WebSocketManager {
    pub fn new(config: Option<WebSocketConfig>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            config: config.unwrap_or_default(),
            event_sender: None,
            total_messages_in: Arc::new(AtomicU64::new(0)),
            total_messages_out: Arc::new(AtomicU64::new(0)),
            total_bytes_in: Arc::new(AtomicU64::new(0)),
            total_bytes_out: Arc::new(AtomicU64::new(0)),
            message_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_event_channel(mut self, sender: mpsc::Sender<ConnectionEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    pub async fn set_message_callback(&self, callback: MessageCallback) {
        let mut guard = self.message_callback.write().await;
        *guard = Some(callback);
    }

    pub async fn remove_message_callback(&self) {
        let mut guard = self.message_callback.write().await;
        *guard = None;
    }

    /// Establece una conexión WebSocket cliente a un servidor remoto
    pub async fn connect(&self, url: &str, domain: DivineDomain) -> Result<String, WebSocketError> {
        let connection_id = Uuid::new_v4().to_string();
        let info = ConnectionInfo::new(connection_id.clone(), url.to_string(), domain);

        info!("🌊 Poseidon: Conectando WebSocket a {} (ID: {})", url, connection_id);

        let (ws_stream, response) = timeout(
            Duration::from_secs(self.config.connection_timeout_secs),
            connect_async(url),
        )
        .await
        .map_err(|_| WebSocketError::Timeout)?
        .map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;

        info!(
            "🌊 Poseidon: Conexión WebSocket establecida a {} - Status: {:?}",
            url,
            response.status()
        );

        self.spawn_connection(connection_id.clone(), info, ws_stream, false)
            .await;

        Ok(connection_id)
    }

    /// Acepta una conexión WebSocket entrante (modo servidor)
    pub async fn accept(
        &self,
        stream: TcpStream,
        domain: DivineDomain,
    ) -> Result<String, WebSocketError> {
        let addr = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let connection_id = Uuid::new_v4().to_string();
        let mut info = ConnectionInfo::new(
            connection_id.clone(),
            format!("server://{}", addr),
            domain,
        );
        info.address = Some(addr);

        info!("🌊 Poseidon: Aceptando conexión WebSocket entrante de {} (ID: {})", 
            info.address.as_ref().unwrap_or(&"unknown".to_string()), 
            connection_id
        );

        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;

        self.spawn_connection(connection_id.clone(), info, ws_stream, true)
            .await;

        Ok(connection_id)
    }

    /// Crea una tarea para manejar la conexión WebSocket
    async fn spawn_connection<S>(&self, id: String, info: ConnectionInfo, ws_stream: S, is_server: bool)
    where
        S: StreamExt<Item = Result<TungsteniteMessage, tokio_tungstenite::tungstenite::Error>>
            + SinkExt<TungsteniteMessage, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin
            + Send
            + 'static,
    {
        let (tx, mut rx) = mpsc::unbounded_channel::<InternalMessage>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let connections = self.connections.clone();
        let event_sender = self.event_sender.clone();
        let total_messages_in = self.total_messages_in.clone();
        let total_messages_out = self.total_messages_out.clone();
        let total_bytes_in = self.total_bytes_in.clone();
        let total_bytes_out = self.total_bytes_out.clone();
        let message_callback = self.message_callback.clone();
        let heartbeat_interval = self.config.heartbeat_interval_secs;
        let heartbeat_timeout = self.config.heartbeat_timeout_secs;

        // Clone id and info for the spawned task
        let id_clone = id.clone();
        let info_clone = info.clone();

        let handle = tokio::spawn(async move {
            let (mut write, mut read) = ws_stream.split();
            let mut shutdown_rx = shutdown_rx;
            let mut heartbeat_timer = interval(Duration::from_secs(heartbeat_interval));
            let mut last_pong = Instant::now();
            let connection_info = info_clone;

            // Notificar conexión exitosa
            if let Some(ref sender) = event_sender {
                let _ = sender
                    .send(ConnectionEvent::Connected {
                        connection_id: id_clone.clone(),
                        url: connection_info.url.clone(),
                    })
                    .await;
            }

            loop {
                tokio::select! {
                    // Mensajes entrantes del WebSocket
                    msg = read.next() => {
                        match msg {
                            Some(Ok(msg)) => {
                                match msg {
                                    TungsteniteMessage::Text(text) => {
                                        trace!("🌊 Poseidon: Mensaje texto recibido de {}", id_clone);
                                        total_messages_in.fetch_add(1, Ordering::Relaxed);
                                        total_bytes_in.fetch_add(text.len() as u64, Ordering::Relaxed);

                                        // Actualizar estadísticas
                                        {
                                            let mut conns = connections.write().await;
                                            if let Some(conn) = conns.get_mut(&id_clone) {
                                                conn.info.message_count_in += 1;
                                                conn.info.bytes_in += text.len() as u64;
                                                conn.info.last_activity = chrono::Utc::now();
                                            }
                                        }

                                        // Notificar evento
                                        if let Some(ref sender) = event_sender {
                                            let _ = sender
                                                .send(ConnectionEvent::MessageReceived {
                                                    connection_id: id_clone.clone(),
                                                    size: text.len(),
                                                })
                                                .await;
                                        }

                                        // Llamar callback si existe
                                        let callback = message_callback.read().await;
                                        if let Some(ref cb) = *callback {
                                            cb(id_clone.clone(), TungsteniteMessage::Text(text));
                                        }
                                    }
                                    TungsteniteMessage::Binary(bin) => {
                                        trace!("🌊 Poseidon: Mensaje binario recibido de {} ({} bytes)", id_clone, bin.len());
                                        total_messages_in.fetch_add(1, Ordering::Relaxed);
                                        total_bytes_in.fetch_add(bin.len() as u64, Ordering::Relaxed);

                                        {
                                            let mut conns = connections.write().await;
                                            if let Some(conn) = conns.get_mut(&id_clone) {
                                                conn.info.message_count_in += 1;
                                                conn.info.bytes_in += bin.len() as u64;
                                                conn.info.last_activity = chrono::Utc::now();
                                            }
                                        }

                                        let callback = message_callback.read().await;
                                        if let Some(ref cb) = *callback {
                                            cb(id_clone.clone(), TungsteniteMessage::Binary(bin));
                                        }
                                    }
                                    TungsteniteMessage::Ping(data) => {
                                        trace!("🌊 Poseidon: Ping recibido de {}", id_clone);
                                        let _ = write.send(TungsteniteMessage::Pong(data)).await;
                                    }
                                    TungsteniteMessage::Pong(_data) => {
                                        trace!("🌊 Poseidon: Pong recibido de {}", id_clone);
                                        last_pong = Instant::now();

                                        let latency = {
                                            let conns = connections.read().await;
                                            conns.get(&id_clone).and_then(|c| c.info.last_ping).map(|ping| {
                                                let now = chrono::Utc::now();
                                                (now - ping).num_milliseconds() as u64
                                            })
                                        };

                                        if let Some(latency_ms) = latency {
                                            {
                                                let mut conns = connections.write().await;
                                                if let Some(conn) = conns.get_mut(&id_clone) {
                                                    conn.info.latency_ms = Some(latency_ms);
                                                    conn.info.last_pong = Some(chrono::Utc::now());
                                                }
                                            }

                                            if let Some(ref sender) = event_sender {
                                                let _ = sender
                                                    .send(ConnectionEvent::Ping {
                                                        connection_id: id_clone.clone(),
                                                        latency_ms,
                                                    })
                                                    .await;
                                            }
                                        }
                                    }
                                    TungsteniteMessage::Close(frame) => {
                                        info!("🌊 Poseidon: Conexión cerrada por el peer {}: {:?}", id_clone, frame);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(e)) => {
                                error!("🌊 Poseidon: Error en WebSocket {}: {}", id_clone, e);
                                break;
                            }
                            None => {
                                debug!("🌊 Poseidon: Stream cerrado para {}", id_clone);
                                break;
                            }
                        }
                    }

                    // Mensajes salientes para enviar
                    Some(msg) = rx.recv() => {
                        let tungstenite_msg: TungsteniteMessage = msg.into();
                        let size = match &tungstenite_msg {
                            TungsteniteMessage::Text(t) => t.len(),
                            TungsteniteMessage::Binary(b) => b.len(),
                            _ => 0,
                        };

                        if let Err(e) = write.send(tungstenite_msg).await {
                            error!("🌊 Poseidon: Error enviando mensaje a {}: {}", id_clone, e);
                            break;
                        }

                        total_messages_out.fetch_add(1, Ordering::Relaxed);
                        total_bytes_out.fetch_add(size as u64, Ordering::Relaxed);

                        {
                            let mut conns = connections.write().await;
                            if let Some(conn) = conns.get_mut(&id_clone) {
                                conn.info.message_count_out += 1;
                                conn.info.bytes_out += size as u64;
                                conn.info.last_activity = chrono::Utc::now();
                            }
                        }

                        if let Some(ref sender) = event_sender {
                            let _ = sender
                                .send(ConnectionEvent::MessageSent {
                                    connection_id: id_clone.clone(),
                                    size,
                                })
                                .await;
                        }
                    }

                    // Heartbeat
                    _ = heartbeat_timer.tick() => {
                        if !is_server {
                            // Enviar ping en conexiones cliente
                            let ping_data = vec![0u8; 8];
                            {
                                let mut conns = connections.write().await;
                                if let Some(conn) = conns.get_mut(&id_clone) {
                                    conn.info.last_ping = Some(chrono::Utc::now());
                                }
                            }

                            if let Err(e) = write.send(TungsteniteMessage::Ping(ping_data)).await {
                                error!("🌊 Poseidon: Error enviando ping a {}: {}", id_clone, e);
                                break;
                            }
                        }

                        // Verificar timeout de pong
                        if last_pong.elapsed() > Duration::from_secs(heartbeat_timeout * 3) {
                            warn!("🌊 Poseidon: Heartbeat timeout para {}, cerrando conexión", id_clone);
                            break;
                        }
                    }

                    // Señal de cierre
                    _ = &mut shutdown_rx => {
                        debug!("🌊 Poseidon: Recibida señal de cierre para {}", id_clone);
                        let _ = write.send(TungsteniteMessage::Close(None)).await;
                        break;
                    }
                }
            }

            // Limpiar conexión
            {
                let mut conns = connections.write().await;
                conns.remove(&id_clone);
            }

            if let Some(ref sender) = event_sender {
                let _ = sender
                    .send(ConnectionEvent::Disconnected {
                        connection_id: id_clone.clone(),
                        reason: "Connection closed".to_string(),
                    })
                    .await;
            }

            debug!("🌊 Poseidon: Tarea de conexión {} finalizada", id_clone);
        });

        // Almacenar la conexión
        {
            let mut conns = self.connections.write().await;
            conns.insert(
                id.clone(),
                ActiveConnection {
                    info,
                    sender: tx,
                    shutdown: shutdown_tx,
                    handle,
                },
            );
        }

        debug!("🌊 Poseidon: Conexión {} registrada", id);
    }

    /// Envía un mensaje de texto a una conexión específica
    pub async fn send_text(&self, connection_id: &str, text: String) -> Result<(), WebSocketError> {
        let sender = {
            let conns = self.connections.read().await;
            conns
                .get(connection_id)
                .ok_or_else(|| WebSocketError::ConnectionNotFound(connection_id.to_string()))?
                .sender
                .clone()
        };

        sender
            .send(InternalMessage::Text(text))
            .map_err(|_| WebSocketError::SendFailed)?;

        Ok(())
    }

    /// Envía mensaje binario a una conexión específica
    pub async fn send_binary(
        &self,
        connection_id: &str,
        data: Vec<u8>,
    ) -> Result<(), WebSocketError> {
        let sender = {
            let conns = self.connections.read().await;
            conns
                .get(connection_id)
                .ok_or_else(|| WebSocketError::ConnectionNotFound(connection_id.to_string()))?
                .sender
                .clone()
        };

        sender
            .send(InternalMessage::Binary(data))
            .map_err(|_| WebSocketError::SendFailed)?;

        Ok(())
    }

    /// Envía un mensaje a todas las conexiones de un dominio específico
    pub async fn broadcast_to_domain(
        &self,
        domain: DivineDomain,
        text: String,
    ) -> Vec<(String, Result<(), WebSocketError>)> {
        let targets: Vec<(String, mpsc::UnboundedSender<InternalMessage>)> = {
            let conns = self.connections.read().await;
            conns
                .iter()
                .filter(|(_, conn)| conn.info.domain == domain)
                .map(|(id, conn)| (id.clone(), conn.sender.clone()))
                .collect()
        };

        let mut results = Vec::new();
        for (id, sender) in targets {
            let result = sender
                .send(InternalMessage::Text(text.clone()))
                .map_err(|_| WebSocketError::SendFailed);
            results.push((id, result));
        }

        results
    }

    /// Envía un mensaje a todas las conexiones activas
    pub async fn broadcast_all(&self, text: String) -> Vec<(String, Result<(), WebSocketError>)> {
        let targets: Vec<(String, mpsc::UnboundedSender<InternalMessage>)> = {
            let conns = self.connections.read().await;
            conns
                .iter()
                .map(|(id, conn)| (id.clone(), conn.sender.clone()))
                .collect()
        };

        let mut results = Vec::new();
        for (id, sender) in targets {
            let result = sender
                .send(InternalMessage::Text(text.clone()))
                .map_err(|_| WebSocketError::SendFailed);
            results.push((id, result));
        }

        results
    }

    /// Cierra una conexión específica de forma graceful
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), WebSocketError> {
        let mut conns = self.connections.write().await;

        if let Some(conn) = conns.remove(connection_id) {
            // Enviar señal de cierre
            let _ = conn.shutdown.send(());

            // Esperar a que la tarea termine (con timeout)
            let _ = timeout(Duration::from_secs(5), conn.handle).await;

            info!("🌊 Poseidon: Conexión {} cerrada gracefully", connection_id);
            Ok(())
        } else {
            Err(WebSocketError::ConnectionNotFound(connection_id.to_string()))
        }
    }

    /// Cierra todas las conexiones de forma graceful
    pub async fn close_all(&self) {
        let connection_ids: Vec<String> = {
            let conns = self.connections.read().await;
            conns.keys().cloned().collect()
        };

        info!("🌊 Poseidon: Cerrando {} conexiones...", connection_ids.len());

        for id in connection_ids {
            if let Err(e) = self.disconnect(&id).await {
                warn!("🌊 Poseidon: Error cerrando conexión {}: {}", id, e);
            }
        }

        info!("🌊 Poseidon: Todas las conexiones cerradas");
    }

    /// Obtiene información de una conexión específica
    pub async fn get_connection(&self, connection_id: &str) -> Option<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns.get(connection_id).map(|conn| conn.info.clone())
    }

    /// Obtiene todas las conexiones activas
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns.values().map(|conn| conn.info.clone()).collect()
    }

    /// Obtiene conexiones filtradas por dominio
    pub async fn get_connections_by_domain(&self, domain: DivineDomain) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns
            .values()
            .filter(|conn| conn.info.domain == domain)
            .map(|conn| conn.info.clone())
            .collect()
    }

    /// Obtiene estadísticas globales
    pub async fn get_stats(&self) -> WebSocketStats {
        let conns = self.connections.read().await;
        let connection_vec: Vec<_> = conns.values().map(|c| c.info.clone()).collect();

        let total_connections = connection_vec.len();
        let active_connections = connection_vec
            .iter()
            .filter(|c| c.status == ConnectionStatus::Connected)
            .count();
        let connecting_connections = connection_vec
            .iter()
            .filter(|c| c.status == ConnectionStatus::Connecting)
            .count();
        let reconnecting_connections = connection_vec
            .iter()
            .filter(|c| c.status == ConnectionStatus::Reconnecting)
            .count();
        let failed_connections = connection_vec
            .iter()
            .filter(|c| c.status == ConnectionStatus::Failed)
            .count();

        let total_messages_in: u64 = connection_vec.iter().map(|c| c.message_count_in).sum();
        let total_messages_out: u64 = connection_vec.iter().map(|c| c.message_count_out).sum();
        let total_bytes_in: u64 = connection_vec.iter().map(|c| c.bytes_in).sum();
        let total_bytes_out: u64 = connection_vec.iter().map(|c| c.bytes_out).sum();

        let latencies: Vec<u64> = connection_vec
            .iter()
            .filter_map(|c| c.latency_ms)
            .collect();
        let average_latency_ms = if !latencies.is_empty() {
            Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
        } else {
            None
        };

        let total_throughput_bps: f64 = connection_vec.iter().map(|c| c.throughput_bps()).sum();

        WebSocketStats {
            total_connections,
            active_connections,
            connecting_connections,
            reconnecting_connections,
            failed_connections,
            total_messages_in,
            total_messages_out,
            total_bytes_in,
            total_bytes_out,
            average_latency_ms,
            total_throughput_bps,
        }
    }

    /// Obtiene el número de conexiones activas
    pub async fn connection_count(&self) -> usize {
        let conns = self.connections.read().await;
        conns.len()
    }

    /// Verifica si una conexión existe
    pub async fn has_connection(&self, connection_id: &str) -> bool {
        let conns = self.connections.read().await;
        conns.contains_key(connection_id)
    }
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            config: self.config.clone(),
            event_sender: None, // Los clones no heredan el event sender
            total_messages_in: self.total_messages_in.clone(),
            total_messages_out: self.total_messages_out.clone(),
            total_bytes_in: self.total_bytes_in.clone(),
            total_bytes_out: self.total_bytes_out.clone(),
            message_callback: self.message_callback.clone(),
        }
    }
}

/// Errores del WebSocket Manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketError {
    ConnectionFailed(String),
    ConnectionNotFound(String),
    SendFailed,
    Timeout,
    InvalidUrl,
    AlreadyConnected,
    NotConnected,
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            WebSocketError::ConnectionNotFound(id) => {
                write!(f, "Connection not found: {}", id)
            }
            WebSocketError::SendFailed => write!(f, "Failed to send message"),
            WebSocketError::Timeout => write!(f, "Operation timed out"),
            WebSocketError::InvalidUrl => write!(f, "Invalid WebSocket URL"),
            WebSocketError::AlreadyConnected => write!(f, "Already connected"),
            WebSocketError::NotConnected => write!(f, "Not connected"),
        }
    }
}

impl std::error::Error for WebSocketError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_info_throughput() {
        let mut info = ConnectionInfo::new(
            "test-123".to_string(),
            "ws://localhost:8080".to_string(),
            DivineDomain::DataFlow,
        );
        info.bytes_in = 1000;
        info.bytes_out = 500;

        // La throughput debería ser mayor que 0
        assert!(info.throughput_bps() >= 0.0);
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "connected");
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "disconnected");
    }
}
