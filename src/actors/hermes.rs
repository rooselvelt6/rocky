/// Hermes v12 - Mensajero Divino
/// Comunicación y mensajería ultrarrápida

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesMessage {
    pub id: String,
    pub message_type: HermesMessageType,
    pub priority: MessagePriority,
    pub sender: String,
    pub recipient: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub delivery_status: DeliveryStatus,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HermesMessageType {
    ClinicalAlert,
    SystemNotification,
    PatientUpdate,
    AssessmentResult,
    SecurityAlert,
    Administrative,
    DataSync,
    EmergencyBroadcast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
    Immediate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Expired,
}

#[derive(Debug, Clone)]
pub struct HermesV12 {
    message_queue: mpsc::UnboundedSender<HermesMessage>,
    delivery_service: DeliveryService,
    message_history: HashMap<String, Vec<HermesMessage>>,
    delivery_stats: DeliveryStatistics,
}

#[derive(Debug, Clone)]
pub struct DeliveryService {
    routes: HashMap<String, DeliveryRoute>,
    retry_policies: HashMap<MessagePriority, RetryPolicy>,
    timeout_config: TimeoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRoute {
    pub route_pattern: String,
    pub handlers: Vec<String>,
    pub load_balancing: LoadBalancingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    PriorityBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub default_timeout_ms: u32,
    pub high_priority_timeout_ms: u32,
    pub critical_timeout_ms: u32,
    pub immediate_timeout_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStatistics {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub average_delivery_time_ms: f64,
    pub success_rate: f64,
    pub retry_rate: f64,
}

impl HermesV12 {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<HermesMessage>) {
        let (message_queue, message_receiver) = mpsc::unbounded_channel();
        
        let delivery_service = DeliveryService {
            routes: HashMap::new(),
            retry_policies: Self::create_default_retry_policies(),
            timeout_config: TimeoutConfig {
                default_timeout_ms: 5000,
                high_priority_timeout_ms: 3000,
                critical_timeout_ms: 1000,
                immediate_timeout_ms: 500,
            },
        };

        let hermes = Self {
            message_queue,
            delivery_service,
            message_history: HashMap::new(),
            delivery_stats: DeliveryStatistics {
                messages_sent: 0,
                messages_delivered: 0,
                messages_failed: 0,
                average_delivery_time_ms: 0.0,
                success_rate: 100.0,
                retry_rate: 0.0,
            },
        };

        (hermes, message_receiver)
    }

    pub async fn send_message(&mut self, message_type: HermesMessageType, recipient: Option<String>, payload: serde_json::Value, priority: MessagePriority) -> Result<String, String> {
        let message = HermesMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            priority: priority.clone(),
            sender: "hermes".to_string(),
            recipient: recipient.clone(),
            payload,
            timestamp: Utc::now(),
            delivery_status: DeliveryStatus::Pending,
            retry_count: 0,
            metadata: HashMap::new(),
        };

        // Validar mensaje
        if let Err(e) = self.validate_message(&message) {
            return Err(format!("Validación fallida: {}", e));
        }

        // Guardar en historial
        let sender_key = message.sender.clone();
        self.message_history.entry(sender_key).or_insert_with(Vec::new).push(message.clone());

        // Enviar a la cola de procesamiento
        match self.message_queue.send(message).await {
            Ok(_) => {
                self.delivery_stats.messages_sent += 1;
                tracing::info!("👟 Hermes: Mensaje {} enviado a {:?} con prioridad {:?}", 
                             message.id, recipient, priority);
                Ok(message.id)
            }
            Err(e) => {
                self.delivery_stats.messages_failed += 1;
                Err(format!("Error enviando mensaje: {}", e))
            }
        }
    }

    pub async fn send_broadcast(&mut self, message_type: HermesMessageType, payload: serde_json::Value, priority: MessagePriority) -> Result<String, String> {
        let message = HermesMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            priority: priority.clone(),
            sender: "hermes".to_string(),
            recipient: None, // Broadcast
            payload,
            timestamp: Utc::now(),
            delivery_status: DeliveryStatus::Pending,
            retry_count: 0,
            metadata: HashMap::new(),
        };

        match self.message_queue.send(message).await {
            Ok(_) => {
                self.delivery_stats.messages_sent += 1;
                tracing::info!("📢 Hermes: Broadcast {} enviado con prioridad {:?}", 
                             message.id, priority);
                Ok(message.id)
            }
            Err(e) => {
                self.delivery_stats.messages_failed += 1;
                Err(format!("Error en broadcast: {}", e))
            }
        }
    }

    pub fn register_route(&mut self, route_pattern: &str, handlers: Vec<String>, load_balancing: LoadBalancingStrategy) {
        let route = DeliveryRoute {
            route_pattern: route_pattern.to_string(),
            handlers,
            load_balancing,
        };

        self.delivery_service.routes.insert(route_pattern.to_string(), route);
        tracing::info!("👟 Hermes: Ruta registrada: {} con {} handlers", route_pattern, handlers.len());
    }

    fn validate_message(&self, message: &HermesMessage) -> Result<(), String> {
        // Validar tamaño del payload
        let payload_size = serde_json::to_string(&message.payload).unwrap_or_default().len();
        if payload_size > 1024 * 1024 { // 1MB
            return Err("Payload demasiado grande (máx 1MB)".to_string());
        }

        // Validar prioridad vs tipo de mensaje
        self.validate_priority_vs_message_type(&message.message_type, &message.priority)?;

        Ok(())
    }

    fn validate_priority_vs_message_type(&self, message_type: &HermesMessageType, priority: &MessagePriority) -> Result<(), String> {
        match message_type {
            HermesMessageType::EmergencyBroadcast => {
                if priority != MessagePriority::Immediate && priority != MessagePriority::Critical {
                    return Err("Broadcasts de emergencia deben tener prioridad Immediate o Critical".to_string());
                }
            }
            HermesMessageType::SecurityAlert => {
                if priority != MessagePriority::High && priority != MessagePriority::Critical && priority != MessagePriority::Immediate {
                    return Err("Alertas de seguridad deben tener prioridad alta o superior".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn create_default_retry_policies() -> HashMap<MessagePriority, RetryPolicy> {
        let mut policies = HashMap::new();
        
        policies.insert(MessagePriority::Low, RetryPolicy {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::Linear,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
        });
        
        policies.insert(MessagePriority::Normal, RetryPolicy {
            max_attempts: 5,
            backoff_strategy: BackoffStrategy::Exponential,
            base_delay_ms: 500,
            max_delay_ms: 30000,
        });
        
        policies.insert(MessagePriority::High, RetryPolicy {
            max_attempts: 5,
            backoff_strategy: BackoffStrategy::Exponential,
            base_delay_ms: 200,
            max_delay_ms: 10000,
        });
        
        policies.insert(MessagePriority::Critical, RetryPolicy {
            max_attempts: 7,
            backoff_strategy: BackoffStrategy::Exponential,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        });
        
        policies.insert(MessagePriority::Immediate, RetryPolicy {
            max_attempts: 1, // Sin reintentos para mensajes inmediatos
            backoff_strategy: BackoffStrategy::Fixed,
            base_delay_ms: 0,
            max_delay_ms: 0,
        });

        policies
    }

    pub fn get_delivery_statistics(&self) -> &DeliveryStatistics {
        &self.delivery_stats
    }

    pub fn get_message_history(&self, sender: &str, limit: Option<usize>) -> Vec<&HermesMessage> {
        if let Some(messages) = self.message_history.get(sender) {
            let mut sorted_messages = messages.clone();
            sorted_messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            if let Some(limit) = limit {
                sorted_messages.truncate(limit);
            }
            
            sorted_messages.iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn update_delivery_stats(&mut self, delivered: bool, delivery_time_ms: u64) {
        if delivered {
            self.delivery_stats.messages_delivered += 1;
        } else {
            self.delivery_stats.messages_failed += 1;
        }

        // Actualizar tiempo promedio de entrega
        let total_deliveries = self.delivery_stats.messages_delivered + self.delivery_stats.messages_failed;
        if total_deliveries > 0 {
            let current_avg = self.delivery_stats.average_delivery_time_ms;
            let new_avg = ((current_avg * (total_deliveries - 1) as f64) + delivery_time_ms as f64) / total_deliveries as f64;
            self.delivery_stats.average_delivery_time_ms = new_avg;
        }

        // Calcular tasas
        if total_deliveries > 0 {
            self.delivery_stats.success_rate = (self.delivery_stats.messages_delivered as f64 / total_deliveries as f64) * 100.0;
            self.delivery_stats.retry_rate = ((total_deliveries - self.delivery_stats.messages_delivered) as f64 / total_deliveries as f64) * 100.0;
        }
    }

    pub fn find_route(&self, recipient: &str) -> Option<&DeliveryRoute> {
        for (pattern, route) in &self.delivery_service.routes {
            if recipient.contains(pattern) {
                return Some(route);
            }
        }
        None
    }

    pub fn optimize_for_high_throughput(&mut self) {
        tracing::info!("👟 Hermes: Optimizando para alto rendimiento");
        
        // Ajustar políticas de retry para alta carga
        for (priority, policy) in self.delivery_service.retry_policies.iter_mut() {
            match priority {
                MessagePriority::Normal => {
                    policy.max_attempts = 3; // Reducir intentos
                    policy.base_delay_ms = 200; // Reducir delay
                }
                MessagePriority::Low => {
                    policy.max_attempts = 1; // Un solo intento
                    policy.base_delay_ms = 500;
                }
                _ => {}
            }
        }
    }

    pub fn enable_guaranteed_delivery(&mut self) {
        tracing::info!("👟 Hermes: Activando modo de entrega garantizada");
        
        // Políticas agresivas de retry para entrega garantizada
        for (priority, policy) in self.delivery_service.retry_policies.iter_mut() {
            policy.max_attempts = 10;
            policy.backoff_strategy = BackoffStrategy::Exponential;
            policy.max_delay_ms = 60000; // 1 minuto máximo
        }
    }

    pub fn get_system_status(&self) -> HermesStatus {
        HermesStatus {
            total_routes: self.delivery_service.routes.len(),
            pending_messages: self.delivery_stats.messages_sent - self.delivery_stats.messages_delivered - self.delivery_stats.messages_failed,
            delivery_rate: format!("{:.2}%", self.delivery_stats.success_rate),
            average_delivery_time: format!("{:.2}ms", self.delivery_stats.average_delivery_time_ms),
            retry_rate: format!("{:.2}%", self.delivery_stats.retry_rate),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HermesStatus {
    pub total_routes: usize,
    pub pending_messages: u64,
    pub delivery_rate: String,
    pub average_delivery_time: String,
    pub retry_rate: String,
}

impl Default for HermesV12 {
    fn default() -> Self {
        Self::new().0
    }
}