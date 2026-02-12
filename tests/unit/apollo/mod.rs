// tests/unit/apollo/mod.rs
// Tests unitarios para Apollo - Motor de Eventos

use olympus::actors::apollo::{Apollo, ApolloConfig, EventBus, EventStore};
use olympus::actors::apollo::events::{Event, EventType, EventPayload, EventFilter};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_apollo_config() {
        let config = ApolloConfig::default();
        assert!(config.persistence_enabled);
        assert_eq!(config.retention_days, 30);
        assert!(config.pub_sub_enabled);
        assert_eq!(config.max_subscribers_per_event, 1000);
    }
    
    #[test]
    fn test_apollo_config_builder() {
        let config = ApolloConfig::new()
            .with_retention_days(7)
            .with_max_subscribers(500)
            .disable_persistence()
            .disable_pub_sub();
            
        assert_eq!(config.retention_days, 7);
        assert_eq!(config.max_subscribers_per_event, 500);
        assert!(!config.persistence_enabled);
        assert!(!config.pub_sub_enabled);
    }
}

/// Tests de emisión de eventos
#[cfg(test)]
mod event_emission_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_emit_simple_event() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let event = Event::new(EventType::UserLogin)
            .with_payload(EventPayload::json({"user_id": "123"}));
        
        let result = apollo.emit(event).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_emit_event_with_correlation_id() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let correlation_id = "req-abc-123";
        let event = Event::new(EventType::PatientCreated)
            .with_correlation_id(correlation_id)
            .with_payload(EventPayload::json({"patient_id": "P001"}));
        
        let result = apollo.emit(event).await;
        
        assert!(result.is_ok());
        
        // Verificar que el evento tiene el correlation_id
        let events = apollo.get_events_by_correlation(correlation_id).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].correlation_id, Some(correlation_id.to_string()));
    }
    
    #[tokio::test]
    async fn test_emit_multiple_events() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        for i in 0..100 {
            let event = Event::new(EventType::SystemMetric)
                .with_payload(EventPayload::json({"metric_id": i}));
            
            apollo.emit(event).await.unwrap();
        }
        
        let event_count = apollo.get_event_count().await;
        assert_eq!(event_count, 100);
    }
    
    #[tokio::test]
    async fn test_emit_event_with_timestamp() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let before = chrono::Utc::now();
        
        let event = Event::new(EventType::DataModified)
            .with_payload(EventPayload::json({"entity": "patient"}));
        
        apollo.emit(event).await.unwrap();
        
        let after = chrono::Utc::now();
        let stored_events = apollo.get_all_events().await;
        
        assert_eq!(stored_events.len(), 1);
        assert!(stored_events[0].timestamp >= before);
        assert!(stored_events[0].timestamp <= after);
    }
}

/// Tests de pub/sub
#[cfg(test)]
mod pub_sub_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_subscribe_to_event_type() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let subscriber_id = "sub-1";
        
        apollo.subscribe(EventType::UserLogin, subscriber_id).await
            .expect("Subscription failed");
        
        let subscribers = apollo.get_subscribers(EventType::UserLogin).await;
        assert!(subscribers.contains(&subscriber_id.to_string()));
    }
    
    #[tokio::test]
    async fn test_event_delivery_to_subscribers() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let subscriber_id = "sub-2";
        
        apollo.subscribe(EventType::PatientUpdated, subscriber_id).await.unwrap();
        
        let event = Event::new(EventType::PatientUpdated)
            .with_payload(EventPayload::json({"patient_id": "P001", "change": "name"}));
        
        apollo.emit(event).await.unwrap();
        
        // Verificar que el suscriptor recibió el evento
        let received = apollo.get_delivered_events(subscriber_id).await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].event_type, EventType::PatientUpdated);
    }
    
    #[tokio::test]
    async fn test_multiple_subscribers_receive_event() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        apollo.subscribe(EventType::SystemAlert, "sub-a").await.unwrap();
        apollo.subscribe(EventType::SystemAlert, "sub-b").await.unwrap();
        apollo.subscribe(EventType::SystemAlert, "sub-c").await.unwrap();
        
        let event = Event::new(EventType::SystemAlert)
            .with_payload(EventPayload::json({"severity": "critical"}));
        
        apollo.emit(event).await.unwrap();
        
        // Todos los suscriptores deben recibirlo
        assert_eq!(apollo.get_delivered_events("sub-a").await.len(), 1);
        assert_eq!(apollo.get_delivered_events("sub-b").await.len(), 1);
        assert_eq!(apollo.get_delivered_events("sub-c").await.len(), 1);
    }
    
    #[tokio::test]
    async fn test_unsubscribe() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let subscriber_id = "sub-3";
        
        apollo.subscribe(EventType::AuditLog, subscriber_id).await.unwrap();
        assert!(apollo.is_subscribed(EventType::AuditLog, subscriber_id).await);
        
        apollo.unsubscribe(EventType::AuditLog, subscriber_id).await.unwrap();
        assert!(!apollo.is_subscribed(EventType::AuditLog, subscriber_id).await);
    }
    
    #[tokio::test]
    async fn test_wildcard_subscription() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let subscriber_id = "wild-sub";
        
        // Suscribirse a todos los eventos de pacientes (Patient*)
        apollo.subscribe_pattern("Patient*", subscriber_id).await.unwrap();
        
        // Emitir diferentes eventos de pacientes
        apollo.emit(Event::new(EventType::PatientCreated)).await.unwrap();
        apollo.emit(Event::new(EventType::PatientUpdated)).await.unwrap();
        apollo.emit(Event::new(EventType::PatientDeleted)).await.unwrap();
        
        // El suscriptor debe recibir todos
        let received = apollo.get_delivered_events(subscriber_id).await;
        assert_eq!(received.len(), 3);
    }
}

/// Tests de persistencia de eventos
#[cfg(test)]
mod event_store_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_persistence() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let event = Event::new(EventType::DataCreated)
            .with_payload(EventPayload::json({"entity": "record", "id": "R001"}));
        
        apollo.emit(event.clone()).await.unwrap();
        
        // Recuperar evento por ID
        let retrieved = apollo.get_event_by_id(&event.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, event.id);
    }
    
    #[tokio::test]
    async fn test_event_query_by_type() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Emitir diferentes tipos de eventos
        apollo.emit(Event::new(EventType::UserLogin)).await.unwrap();
        apollo.emit(Event::new(EventType::UserLogout)).await.unwrap();
        apollo.emit(Event::new(EventType::UserLogin)).await.unwrap();
        apollo.emit(Event::new(EventType::SystemMetric)).await.unwrap();
        
        // Consultar solo UserLogin
        let logins = apollo.get_events_by_type(EventType::UserLogin).await;
        assert_eq!(logins.len(), 2);
    }
    
    #[tokio::test]
    async fn test_event_query_by_time_range() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let start = chrono::Utc::now();
        
        // Emitir eventos
        for _ in 0..5 {
            apollo.emit(Event::new(EventType::SystemMetric)).await.unwrap();
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let end = chrono::Utc::now();
        
        // Consultar por rango de tiempo
        let events = apollo.get_events_by_time_range(start, end).await;
        assert_eq!(events.len(), 5);
    }
    
    #[tokio::test]
    async fn test_event_filtering() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Eventos con diferentes payloads
        apollo.emit(Event::new(EventType::DataModified)
            .with_payload(EventPayload::json({"severity": "low"}))).await.unwrap();
        
        apollo.emit(Event::new(EventType::DataModified)
            .with_payload(EventPayload::json({"severity": "high"}))).await.unwrap();
        
        apollo.emit(Event::new(EventType::DataModified)
            .with_payload(EventPayload::json({"severity": "critical"}))).await.unwrap();
        
        // Filtrar por severidad en payload
        let filter = EventFilter::new()
            .with_type(EventType::DataModified)
            .with_payload_contains("critical");
        
        let critical_events = apollo.filter_events(filter).await;
        assert_eq!(critical_events.len(), 1);
    }
    
    #[tokio::test]
    async fn test_event_replay() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Emitir secuencia de eventos
        for i in 0..10 {
            apollo.emit(Event::new(EventType::StateChange)
                .with_payload(EventPayload::json({"step": i}))).await.unwrap();
        }
        
        // Reproducir desde el principio
        let replayed = apollo.replay_events_from(0).await;
        assert_eq!(replayed.len(), 10);
        
        // Verificar orden
        for (i, event) in replayed.iter().enumerate() {
            let step = event.payload.get("step").unwrap().as_i64().unwrap();
            assert_eq!(step, i as i64);
        }
    }
    
    #[tokio::test]
    async fn test_event_retention_cleanup() {
        let config = ApolloConfig::new()
            .with_retention_days(0); // Inmediatamente
        
        let apollo = Apollo::with_config(config).await.expect("Failed to create Apollo");
        
        // Emitir eventos viejos (simulados)
        apollo.emit_old_event(Event::new(EventType::OldEvent), 31).await.unwrap();
        
        // Ejecutar cleanup
        apollo.cleanup_old_events().await.unwrap();
        
        // Los eventos viejos deben estar eliminados
        let old_events = apollo.get_events_by_type(EventType::OldEvent).await;
        assert_eq!(old_events.len(), 0);
    }
}

/// Tests de snapshotting
#[cfg(test)]
mod snapshot_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_snapshot() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Emitir eventos de estado
        for i in 0..100 {
            apollo.emit(Event::new(EventType::StateChange)
                .with_payload(EventPayload::json({"value": i}))).await.unwrap();
        }
        
        // Crear snapshot
        let snapshot = apollo.create_snapshot("aggregate-1").await
            .expect("Snapshot creation failed");
        
        assert_eq!(snapshot.event_count, 100);
        assert!(snapshot.data.contains_key("value"));
    }
    
    #[tokio::test]
    async fn test_restore_from_snapshot() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Crear snapshot
        apollo.emit(Event::new(EventType::CounterIncrement)).await.unwrap();
        apollo.emit(Event::new(EventType::CounterIncrement)).await.unwrap();
        apollo.emit(Event::new(EventType::CounterIncrement)).await.unwrap();
        
        let snapshot = apollo.create_snapshot("counter").await.unwrap();
        
        // Limpiar eventos
        apollo.clear_events().await.unwrap();
        
        // Restaurar desde snapshot
        apollo.restore_from_snapshot(&snapshot).await.expect("Restore failed");
        
        // Verificar estado restaurado
        let state = apollo.get_aggregate_state("counter").await;
        assert_eq!(state.get("count").unwrap().as_i64().unwrap(), 3);
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_apollo_message_emit_event() {
        let mut apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let event = Event::new(EventType::TestEvent);
        let message = ActorMessage::emit_event(event);
        
        let response = apollo.handle_message(message).await;
        
        assert!(response.is_ok());
        assert_eq!(apollo.get_event_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_apollo_message_subscribe() {
        let mut apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let message = ActorMessage::subscribe(EventType::AuditLog, "test-sub");
        let response = apollo.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(apollo.is_subscribed(EventType::AuditLog, "test-sub").await);
    }
    
    #[tokio::test]
    async fn test_apollo_message_query_events() {
        let mut apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Agregar eventos
        for _ in 0..5 {
            apollo.emit(Event::new(EventType::UserAction)).await.unwrap();
        }
        
        let message = ActorMessage::query_events(EventType::UserAction);
        let response = apollo.handle_message(message).await;
        
        assert!(response.is_ok());
        let events = response.unwrap().get_events();
        assert_eq!(events.len(), 5);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_event_throughput() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let count = 10000;
        
        let start = Instant::now();
        
        for i in 0..count {
            let event = Event::new(EventType::HighFrequency)
                .with_payload(EventPayload::json({"seq": i}));
            apollo.emit(event).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let events_per_sec = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            events_per_sec > 50000.0,
            "Event throughput too low: {:.0} events/sec",
            events_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_pub_sub_latency() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        apollo.subscribe(EventType::LatencyTest, "latency-sub").await.unwrap();
        
        let start = Instant::now();
        
        apollo.emit(Event::new(EventType::LatencyTest)).await.unwrap();
        
        // Esperar delivery
        let mut received = false;
        for _ in 0..100 {
            if !apollo.get_delivered_events("latency-sub").await.is_empty() {
                received = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        
        let elapsed = start.elapsed();
        
        assert!(received, "Event was not delivered");
        assert!(
            elapsed.as_micros() < 10000,
            "Pub/sub latency too high: {:?}",
            elapsed
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_event_emission() {
        use tokio::task;
        
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let apollo = std::sync::Arc::new(tokio::sync::Mutex::new(apollo));
        
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let apollo_clone = apollo.clone();
            let handle = task::spawn(async move {
                for i in 0..100 {
                    let event = Event::new(EventType::ConcurrentEvent)
                        .with_payload(EventPayload::json({
                            "thread": thread_id,
                            "seq": i
                        }));
                    apollo_clone.lock().await.emit(event).await.unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        let total_events = apollo.lock().await.get_event_count().await;
        assert_eq!(total_events, 1000);
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_event_payload() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let event = Event::new(EventType::EmptyEvent);
        // Sin payload
        
        let result = apollo.emit(event).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_very_large_payload() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let large_data: serde_json::Value = (0..10000)
            .map(|i| (format!("key-{}", i), i))
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();
        
        let event = Event::new(EventType::LargeData)
            .with_payload(EventPayload::json(large_data));
        
        let result = apollo.emit(event).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rapid_subscribe_unsubscribe() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        let subscriber_id = "rapid-sub";
        
        for _ in 0..100 {
            apollo.subscribe(EventType::VolatileEvent, subscriber_id).await.unwrap();
            apollo.unsubscribe(EventType::VolatileEvent, subscriber_id).await.unwrap();
        }
        
        assert!(!apollo.is_subscribed(EventType::VolatileEvent, subscriber_id).await);
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_apollo_creation() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        assert_eq!(apollo.name(), GodName::Apollo);
        assert_eq!(apollo.domain(), DivineDomain::Events);
    }
    
    #[tokio::test]
    async fn test_apollo_health_check() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        let health = apollo.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let apollo = Apollo::new().await.expect("Failed to create Apollo");
        
        // Emitir algunos eventos
        for _ in 0..50 {
            apollo.emit(Event::new(EventType::MetricTest)).await.unwrap();
        }
        
        let metrics = apollo.collect_metrics().await;
        
        assert_eq!(metrics.events_emitted, 50);
        assert_eq!(metrics.events_stored, 50);
        assert!(metrics.average_latency_ms >= 0.0);
    }
}
