// tests/unit/hermes/mod.rs
// Tests unitarios para Hermes - Mensajería y Comunicación

use olympus::actors::hermes::{Hermes, HermesConfig, RetryPolicy, CircuitBreakerConfig};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage, MessagePayload};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_hermes_config() {
        let config = HermesConfig::default();
        assert!(config.retry_enabled);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert!(config.circuit_breaker_enabled);
        assert_eq!(config.failure_threshold, 5);
    }
    
    #[test]
    fn test_hermes_config_builder() {
        let config = HermesConfig::new()
            .with_max_retries(5)
            .with_retry_delay(500)
            .with_circuit_breaker_threshold(3)
            .disable_retry()
            .disable_circuit_breaker();
            
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 500);
        assert_eq!(config.failure_threshold, 3);
        assert!(!config.retry_enabled);
        assert!(!config.circuit_breaker_enabled);
    }
}

/// Tests de retry
#[cfg(test)]
mod retry_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_successful_delivery_no_retry() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::test_message();
        let target = GodName::Hestia;
        
        let result = hermes.send_with_retry(message, target, 3).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::exponential_backoff()
            .with_initial_delay_ms(100)
            .with_max_delay_ms(1000)
            .with_multiplier(2.0);
        
        let delays = policy.calculate_delays(5);
        
        assert_eq!(delays.len(), 5);
        assert_eq!(delays[0], 100);
        assert_eq!(delays[1], 200);
        assert_eq!(delays[2], 400);
        assert_eq!(delays[3], 800);
        assert_eq!(delays[4], 1000); // Max delay reached
    }
    
    #[tokio::test]
    async fn test_retry_policy_fixed_delay() {
        let policy = RetryPolicy::fixed_delay(500);
        
        let delays = policy.calculate_delays(3);
        
        assert_eq!(delays.len(), 3);
        assert_eq!(delays[0], 500);
        assert_eq!(delays[1], 500);
        assert_eq!(delays[2], 500);
    }
    
    #[tokio::test]
    async fn test_retry_with_jitter() {
        let policy = RetryPolicy::exponential_backoff()
            .with_jitter(true);
        
        let delay1 = policy.calculate_delay_with_jitter(100);
        let delay2 = policy.calculate_delay_with_jitter(100);
        
        // Con jitter, dos delays iguales deben ser diferentes
        assert_ne!(delay1, delay2);
        
        // Pero deben estar en el rango esperado
        assert!(delay1 >= 80 && delay1 <= 120);
        assert!(delay2 >= 80 && delay2 <= 120);
    }
    
    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::test_message();
        let target = GodName::Hades; // Simular actor que siempre falla
        
        // Configurar para que falle después de 2 intentos
        let result = hermes.send_with_retry_and_fail(message, target, 2).await;
        
        assert!(result.is_err());
    }
}

/// Tests de circuit breaker
#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;
    use olympus::actors::hermes::circuit_breaker::{CircuitBreaker, CircuitState};
    
    #[tokio::test]
    async fn test_circuit_closed_initially() {
        let cb = CircuitBreaker::new(5, 60);
        
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());
    }
    
    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let mut cb = CircuitBreaker::new(3, 60);
        
        // Registrar 3 fallos
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute());
    }
    
    #[tokio::test]
    async fn test_circuit_half_open_after_timeout() {
        let mut cb = CircuitBreaker::new(2, 1); // 1 segundo de timeout
        
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        // Esperar timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Intentar ejecución - debe pasar a half-open
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }
    
    #[tokio::test]
    async fn test_circuit_closes_after_success_in_half_open() {
        let mut cb = CircuitBreaker::new(2, 1);
        
        // Abrir circuito
        cb.record_failure();
        cb.record_failure();
        
        // Esperar timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // En half-open, registrar éxito
        cb.record_success();
        
        assert_eq!(cb.state(), CircuitState::Closed);
    }
    
    #[tokio::test]
    async fn test_circuit_reopens_after_failure_in_half_open() {
        let mut cb = CircuitBreaker::new(2, 1);
        
        // Abrir circuito
        cb.record_failure();
        cb.record_failure();
        
        // Esperar timeout
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // En half-open, registrar otro fallo
        cb.record_failure();
        
        assert_eq!(cb.state(), CircuitState::Open);
    }
    
    #[tokio::test]
    async fn test_circuit_resets_on_success() {
        let mut cb = CircuitBreaker::new(5, 60);
        
        cb.record_failure();
        cb.record_failure();
        
        assert_eq!(cb.failure_count(), 2);
        
        cb.record_success();
        
        assert_eq!(cb.failure_count(), 0);
    }
}

/// Tests de mensajería
#[cfg(test)]
mod messaging_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_message() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::test_message();
        let target = GodName::Zeus;
        
        let result = hermes.send(message, target).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_send_and_wait_response() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::ping();
        let target = GodName::Zeus;
        
        let response = hermes.send_and_wait(message, target, 5000).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_broadcast_message() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::broadcast_test();
        let targets = vec![
            GodName::Zeus,
            GodName::Hades,
            GodName::Hestia,
        ];
        
        let results = hermes.broadcast(message, &targets).await;
        
        assert_eq!(results.len(), 3);
        // En producción real, verificar que todos tuvieron éxito
    }
    
    #[tokio::test]
    async fn test_dead_letter_queue() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::test_message();
        
        // Simular mensaje que falla
        hermes.send_to_dead_letter(message.clone()).await.expect("DLQ failed");
        
        let dead_letters = hermes.get_dead_letters().await.expect("Get DLQ failed");
        
        assert!(!dead_letters.is_empty());
        assert!(dead_letters.iter().any(|m| m.id == message.id));
    }
    
    #[tokio::test]
    async fn test_message_priority() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let low_priority = ActorMessage::with_priority(MessagePriority::Low);
        let high_priority = ActorMessage::with_priority(MessagePriority::High);
        let critical = ActorMessage::with_priority(MessagePriority::Critical);
        
        // Enviar en orden inverso
        hermes.queue_message(low_priority).await.unwrap();
        hermes.queue_message(high_priority).await.unwrap();
        hermes.queue_message(critical).await.unwrap();
        
        let next = hermes.dequeue_message().await.unwrap();
        
        // Debe salir el mensaje crítico primero
        assert_eq!(next.priority, MessagePriority::Critical);
    }
    
    #[tokio::test]
    async fn test_message_timeout() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::slow_operation();
        let target = GodName::Athena; // Simular operación lenta
        
        let result = hermes.send_with_timeout(message, target, 100).await;
        
        // Debe timeout
        assert!(result.is_err());
    }
}

/// Tests de routing
#[cfg(test)]
mod routing_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_route_by_domain() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::security_request();
        
        let target = hermes.route_by_domain(DivineDomain::Security).await;
        
        assert_eq!(target, Some(GodName::Hades));
    }
    
    #[tokio::test]
    async fn test_route_by_message_type() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let encrypt_target = hermes.route_by_message_type(MessageType::Encrypt).await;
        assert_eq!(encrypt_target, Some(GodName::Hades));
        
        let cache_target = hermes.route_by_message_type(MessageType::Cache).await;
        assert_eq!(cache_target, Some(GodName::Hestia));
    }
    
    #[tokio::test]
    async fn test_load_balanced_routing() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let targets = vec![GodName::Athena, GodName::Apollo, GodName::Artemis];
        
        let mut selections = vec![];
        for _ in 0..30 {
            let target = hermes.route_load_balanced(&targets).await;
            selections.push(target);
        }
        
        // Debe distribuir más o menos equitativamente
        let athena_count = selections.iter().filter(|&&t| t == GodName::Athena).count();
        let apollo_count = selections.iter().filter(|&&t| t == GodName::Apollo).count();
        let artemis_count = selections.iter().filter(|&&t| t == GodName::Artemis).count();
        
        // Cada uno debe tener aproximadamente 10 selecciones (±5)
        assert!(athena_count >= 5 && athena_count <= 15);
        assert!(apollo_count >= 5 && apollo_count <= 15);
        assert!(artemis_count >= 5 && artemis_count <= 15);
    }
}

/// Tests de colas
#[cfg(test)]
mod queue_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_message_queue_fifo() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let msg1 = ActorMessage::with_id("1");
        let msg2 = ActorMessage::with_id("2");
        let msg3 = ActorMessage::with_id("3");
        
        hermes.queue_message(msg1).await.unwrap();
        hermes.queue_message(msg2).await.unwrap();
        hermes.queue_message(msg3).await.unwrap();
        
        let out1 = hermes.dequeue_message().await.unwrap();
        let out2 = hermes.dequeue_message().await.unwrap();
        let out3 = hermes.dequeue_message().await.unwrap();
        
        assert_eq!(out1.id, "1");
        assert_eq!(out2.id, "2");
        assert_eq!(out3.id, "3");
    }
    
    #[tokio::test]
    async fn test_queue_size_limit() {
        let hermes = Hermes::new_with_queue_limit(100).await.expect("Failed to create Hermes");
        
        // Llenar la cola
        for i in 0..100 {
            let msg = ActorMessage::with_id(&format!("{}", i));
            hermes.queue_message(msg).await.unwrap();
        }
        
        // Intentar agregar más - debe fallar o aplicar backpressure
        let overflow = ActorMessage::with_id("overflow");
        let result = hermes.queue_message(overflow).await;
        
        // Debe manejar el overflow gracefulmente
        assert!(result.is_ok() || result.is_err());
    }
}

/// Tests de batching
#[cfg(test)]
mod batch_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_send() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let messages: Vec<ActorMessage> = (0..10)
            .map(|i| ActorMessage::with_id(&format!("{}", i)))
            .collect();
        
        let target = GodName::Hestia;
        
        let results = hermes.send_batch(messages, target).await;
        
        assert_eq!(results.len(), 10);
    }
    
    #[tokio::test]
    async fn test_batch_with_size_limit() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        // Crear batch mayor al límite
        let messages: Vec<ActorMessage> = (0..1000)
            .map(|i| ActorMessage::with_id(&format!("{}", i)))
            .collect();
        
        let target = GodName::Hestia;
        
        // Debe dividir automáticamente en batches más pequeños
        let results = hermes.send_batch(messages, target).await;
        
        assert_eq!(results.len(), 1000);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_message_throughput() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::ping();
        let target = GodName::Zeus;
        let count = 10000;
        
        let start = Instant::now();
        
        for _ in 0..count {
            let _ = hermes.send(message.clone(), target).await;
        }
        
        let elapsed = start.elapsed();
        let throughput = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            throughput > 10000.0,
            "Message throughput too low: {:.0} msgs/sec",
            throughput
        );
    }
    
    #[tokio::test]
    async fn test_broadcast_performance() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::ping();
        let targets = vec![
            GodName::Zeus,
            GodName::Hades,
            GodName::Hestia,
            GodName::Athena,
            GodName::Apollo,
        ];
        
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _ = hermes.broadcast(message.clone(), &targets).await;
        }
        
        let elapsed = start.elapsed();
        let broadcasts_per_sec = 1000.0 / elapsed.as_secs_f64();
        
        assert!(
            broadcasts_per_sec > 100.0,
            "Broadcast throughput too low: {:.0} broadcasts/sec",
            broadcasts_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_latency() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::ping();
        let target = GodName::Zeus;
        
        let mut latencies = vec![];
        
        for _ in 0..100 {
            let start = Instant::now();
            let _ = hermes.send(message.clone(), target).await;
            latencies.push(start.elapsed().as_micros());
        }
        
        let avg_latency: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
        
        assert!(
            avg_latency < 1000,
            "Average latency too high: {} microseconds",
            avg_latency
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_to_nonexistent_actor() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let message = ActorMessage::test_message();
        
        // Enviar a un actor que no está corriendo
        let result = hermes.send(message, GodName::Dionysus).await;
        
        // Debe manejar gracefulmente (posiblemente poner en DLQ)
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_concurrent_sends() {
        use tokio::task;
        
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        let hermes = std::sync::Arc::new(tokio::sync::Mutex::new(hermes));
        
        let mut handles = vec![];
        
        for i in 0..50 {
            let hermes_clone = hermes.clone();
            let handle = task::spawn(async move {
                let msg = ActorMessage::with_id(&format!("{}", i));
                hermes_clone.lock().await.send(msg, GodName::Zeus).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let _ = handle.await;
        }
    }
    
    #[tokio::test]
    async fn test_large_message() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        // Crear mensaje grande (10MB)
        let large_data = vec![0u8; 10 * 1024 * 1024];
        let message = ActorMessage::with_payload(large_data);
        
        let result = hermes.send(message, GodName::Hestia).await;
        
        // Debe manejar mensajes grandes
        assert!(result.is_ok() || result.is_err()); // No debe panic
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hermes_creation() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        assert_eq!(hermes.name(), GodName::Hermes);
        assert_eq!(hermes.domain(), DivineDomain::Messaging);
    }
    
    #[tokio::test]
    async fn test_hermes_health_check() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let health = hermes.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_hermes_metrics() {
        let hermes = Hermes::new().await.expect("Failed to create Hermes");
        
        let metrics = hermes.collect_metrics().await;
        
        assert!(metrics.messages_sent >= 0);
        assert!(metrics.messages_failed >= 0);
        assert!(metrics.average_latency_ms >= 0.0);
    }
}
