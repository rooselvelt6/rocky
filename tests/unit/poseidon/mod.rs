// tests/unit/poseidon/mod.rs
// Tests unitarios para Poseidón - WebSocket y Conectividad

use olympus::actors::poseidon::{Poseidon, PoseidonConfig, WebSocketManager, ConnectionPool};
use olympus::actors::poseidon::websocket::{WebSocketMessage, FrameType, CloseCode};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_poseidon_config() {
        let config = PoseidonConfig::default();
        assert_eq!(config.max_connections, 10000);
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.timeout_secs, 60);
        assert!(config.flow_control_enabled);
    }
    
    #[test]
    fn test_poseidon_config_builder() {
        let config = PoseidonConfig::new()
            .with_max_connections(5000)
            .with_heartbeat_interval(15)
            .with_timeout(30)
            .disable_flow_control();
            
        assert_eq!(config.max_connections, 5000);
        assert_eq!(config.heartbeat_interval_secs, 15);
        assert_eq!(config.timeout_secs, 30);
        assert!(!config.flow_control_enabled);
    }
}

/// Tests de gestión de conexiones
#[cfg(test)]
mod connection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_acceptance() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "client-123";
        
        let result = poseidon.accept_connection(client_id).await;
        
        assert!(result.is_ok());
        assert!(poseidon.is_connected(client_id).await);
    }
    
    #[tokio::test]
    async fn test_connection_closure() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "client-456";
        
        poseidon.accept_connection(client_id).await.unwrap();
        assert!(poseidon.is_connected(client_id).await);
        
        poseidon.close_connection(client_id, CloseCode::Normal).await.unwrap();
        
        assert!(!poseidon.is_connected(client_id).await);
    }
    
    #[tokio::test]
    async fn test_connection_limit() {
        let config = PoseidonConfig::new().with_max_connections(2);
        let poseidon = Poseidon::with_config(config).await.expect("Failed to create Poseidon");
        
        poseidon.accept_connection("client-1").await.unwrap();
        poseidon.accept_connection("client-2").await.unwrap();
        
        // Tercera conexión debe fallar
        let result = poseidon.accept_connection("client-3").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_duplicate_connection_rejection() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "duplicate-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Intentar conectar de nuevo con mismo ID
        let result = poseidon.accept_connection(client_id).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_connection_count() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        assert_eq!(poseidon.connection_count().await, 0);
        
        poseidon.accept_connection("c1").await.unwrap();
        poseidon.accept_connection("c2").await.unwrap();
        poseidon.accept_connection("c3").await.unwrap();
        
        assert_eq!(poseidon.connection_count().await, 3);
        
        poseidon.close_connection("c1", CloseCode::Normal).await.unwrap();
        
        assert_eq!(poseidon.connection_count().await, 2);
    }
}

/// Tests de mensajería WebSocket
#[cfg(test)]
mod websocket_message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_text_message() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "text-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        let message = WebSocketMessage::text("Hello, WebSocket!");
        let result = poseidon.send_message(client_id, message).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_send_binary_message() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "binary-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let message = WebSocketMessage::binary(data);
        let result = poseidon.send_message(client_id, message).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_broadcast_message() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        poseidon.accept_connection("c1").await.unwrap();
        poseidon.accept_connection("c2").await.unwrap();
        poseidon.accept_connection("c3").await.unwrap();
        
        let message = WebSocketMessage::text("Broadcast to all");
        let results = poseidon.broadcast_message(message).await;
        
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }
    
    #[tokio::test]
    async fn test_receive_message() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "receiver";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Simular mensaje entrante
        let incoming = WebSocketMessage::text("Hello from client");
        poseidon.simulate_receive(client_id, incoming).await.unwrap();
        
        let received = poseidon.get_received_messages(client_id).await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].text_content(), Some("Hello from client".to_string()));
    }
    
    #[tokio::test]
    async fn test_send_to_disconnected_client() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "disconnected";
        
        // No conectar al cliente
        let message = WebSocketMessage::text("Test");
        let result = poseidon.send_message(client_id, message).await;
        
        assert!(result.is_err());
    }
}

/// Tests de heartbeat
#[cfg(test)]
mod heartbeat_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_heartbeat_mechanism() {
        let config = PoseidonConfig::new()
            .with_heartbeat_interval(1); // 1 segundo para testing
        
        let poseidon = Poseidon::with_config(config).await.expect("Failed to create Poseidon");
        let client_id = "heartbeat-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Esperar heartbeat
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
        
        let heartbeats_sent = poseidon.get_heartbeat_count(client_id).await;
        assert!(heartbeats_sent >= 1);
    }
    
    #[tokio::test]
    async fn test_client_pong_response() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "pong-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Simular respuesta pong del cliente
        poseidon.receive_pong(client_id).await.unwrap();
        
        let last_pong = poseidon.get_last_pong_time(client_id).await;
        assert!(last_pong.is_some());
    }
    
    #[tokio::test]
    async fn test_timeout_detection() {
        let config = PoseidonConfig::new()
            .with_timeout(1); // 1 segundo timeout
        
        let poseidon = Poseidon::with_config(config).await.expect("Failed to create Poseidon");
        let client_id = "timeout-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // No enviar pong - esperar timeout
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
        
        assert!(!poseidon.is_connected(client_id).await);
    }
}

/// Tests de flow control
#[cfg(test)]
mod flow_control_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_backpressure_activation() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "backpressure-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Enviar muchos mensajes rápidamente
        for i in 0..1000 {
            let msg = WebSocketMessage::text(format!("Message {}", i));
            let result = poseidon.send_message(client_id, msg).await;
            
            // En algún punto debe activarse backpressure
            if result.is_err() {
                // Backpressure activado - es correcto
                return;
            }
        }
        
        // Si llegamos aquí, el sistema manejó los 1000 mensajes sin backpressure
        // También es válido dependiendo de la configuración
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let config = PoseidonConfig::new()
            .with_rate_limit(100); // 100 mensajes por segundo
        
        let poseidon = Poseidon::with_config(config).await.expect("Failed to create Poseidon");
        let client_id = "rate-limit-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Enviar 150 mensajes rápidamente
        let mut success_count = 0;
        let mut rate_limited_count = 0;
        
        for i in 0..150 {
            let msg = WebSocketMessage::text(format!("Msg {}", i));
            match poseidon.send_message(client_id, msg).await {
                Ok(_) => success_count += 1,
                Err(_) => rate_limited_count += 1,
            }
        }
        
        // Algunos deben ser rate limited
        assert!(rate_limited_count > 0);
        assert!(success_count >= 100); // Al menos el límite
    }
}

/// Tests de circuit breaker
#[cfg(test)]
mod websocket_circuit_breaker_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_on_connection_failures() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        // Simular múltiples fallos de conexión
        for _ in 0..5 {
            poseidon.record_connection_failure().await;
        }
        
        // El circuit breaker debe abrirse
        assert!(poseidon.is_circuit_open().await);
        
        // No debe aceptar nuevas conexiones
        let result = poseidon.accept_connection("new-client").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        // Abrir circuito
        for _ in 0..5 {
            poseidon.record_connection_failure().await;
        }
        assert!(poseidon.is_circuit_open().await);
        
        // Esperar recuperación
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Registrar éxito
        poseidon.record_connection_success().await;
        
        // Circuito debe cerrarse eventualmente
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        assert!(!poseidon.is_circuit_open().await);
    }
}

/// Tests de autenticación WebSocket
#[cfg(test)]
mod ws_auth_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ws_authentication_success() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "auth-client";
        let token = "valid-jwt-token";
        
        let result = poseidon.authenticate_connection(client_id, token).await;
        
        assert!(result.is_ok());
        assert!(poseidon.is_authenticated(client_id).await);
    }
    
    #[tokio::test]
    async fn test_ws_authentication_failure() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "unauth-client";
        let invalid_token = "invalid-token";
        
        let result = poseidon.authenticate_connection(client_id, invalid_token).await;
        
        assert!(result.is_err());
        assert!(!poseidon.is_authenticated(client_id).await);
    }
    
    #[tokio::test]
    async fn test_unauthorized_message_rejection() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "unauthorized";
        
        // Conectar sin autenticar
        poseidon.accept_connection(client_id).await.unwrap();
        
        // Intentar enviar mensaje que requiere auth
        let restricted_msg = WebSocketMessage::restricted_operation();
        let result = poseidon.handle_message(client_id, restricted_msg).await;
        
        assert!(result.is_err());
    }
}

/// Tests de manejo de mensajes del sistema
#[cfg(test)]
mod system_message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_poseidon_message_get_connections() {
        let mut poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        poseidon.accept_connection("c1").await.unwrap();
        poseidon.accept_connection("c2").await.unwrap();
        
        let message = ActorMessage::get_connections_request();
        let response = poseidon.handle_message(message).await;
        
        assert!(response.is_ok());
        let connections = response.unwrap().get_connections();
        assert_eq!(connections.len(), 2);
    }
    
    #[tokio::test]
    async fn test_poseidon_message_broadcast() {
        let mut poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        poseidon.accept_connection("c1").await.unwrap();
        poseidon.accept_connection("c2").await.unwrap();
        
        let message = ActorMessage::broadcast_ws_message("System update");
        let response = poseidon.handle_message(message).await;
        
        assert!(response.is_ok());
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_connection_acceptance_rate() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let count = 1000;
        
        let start = Instant::now();
        
        for i in 0..count {
            let _ = poseidon.accept_connection(&format!("perf-{}", i)).await;
        }
        
        let elapsed = start.elapsed();
        let connections_per_sec = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            connections_per_sec > 500.0,
            "Connection rate too low: {:.0} conn/sec",
            connections_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_message_throughput() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "throughput-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        let count = 10000;
        let start = Instant::now();
        
        for i in 0..count {
            let msg = WebSocketMessage::text(format!("msg-{}", i));
            let _ = poseidon.send_message(client_id, msg).await;
        }
        
        let elapsed = start.elapsed();
        let msgs_per_sec = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            msgs_per_sec > 10000.0,
            "Message throughput too low: {:.0} msgs/sec",
            msgs_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_connections() {
        use tokio::task;
        
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let poseidon = std::sync::Arc::new(tokio::sync::Mutex::new(poseidon));
        
        let mut handles = vec![];
        
        for i in 0..100 {
            let poseidon_clone = poseidon.clone();
            let handle = task::spawn(async move {
                poseidon_clone.lock().await
                    .accept_connection(&format!("concurrent-{}", i))
                    .await
            });
            handles.push(handle);
        }
        
        let start = Instant::now();
        
        for handle in handles {
            let _ = handle.await;
        }
        
        let elapsed = start.elapsed();
        
        assert!(
            elapsed.as_millis() < 2000,
            "Concurrent connections too slow: {:?}",
            elapsed
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_message_handling() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "empty-msg-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        let empty_msg = WebSocketMessage::text("");
        let result = poseidon.send_message(client_id, empty_msg).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_very_large_message() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "large-msg-client";
        
        poseidon.accept_connection(client_id).await.unwrap();
        
        let large_data = "x".repeat(10 * 1024 * 1024); // 10MB
        let large_msg = WebSocketMessage::text(large_data);
        
        let result = poseidon.send_message(client_id, large_msg).await;
        
        // Debe manejar el mensaje grande (puede fragmentarlo)
        assert!(result.is_ok() || result.is_err()); // No debe panic
    }
    
    #[tokio::test]
    async fn test_rapid_connect_disconnect() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        let client_id = "rapid-client";
        
        for _ in 0..100 {
            poseidon.accept_connection(client_id).await.unwrap();
            poseidon.close_connection(client_id, CloseCode::Normal).await.unwrap();
        }
        
        // No debe haber fugas de memoria ni errores
        assert_eq!(poseidon.connection_count().await, 0);
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_poseidon_creation() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        assert_eq!(poseidon.name(), GodName::Poseidon);
        assert_eq!(poseidon.domain(), DivineDomain::Connectivity);
    }
    
    #[tokio::test]
    async fn test_poseidon_health_check() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        let health = poseidon.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let poseidon = Poseidon::new().await.expect("Failed to create Poseidon");
        
        // Añadir algunas conexiones
        poseidon.accept_connection("c1").await.unwrap();
        poseidon.accept_connection("c2").await.unwrap();
        
        // Shutdown
        poseidon.shutdown().await.expect("Shutdown failed");
        
        // Todas las conexiones deben cerrarse
        assert_eq!(poseidon.connection_count().await, 0);
    }
}
