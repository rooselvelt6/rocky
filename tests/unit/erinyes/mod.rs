// tests/unit/erinyes/mod.rs
// Tests unitarios para Erinyes - Monitoreo y Recuperación

use olympus::actors::erinyes::{Erinyes, ErinyesConfig, Watchdog, HeartbeatMonitor};
use olympus::actors::erinyes::recovery::{RecoveryStrategy, RecoveryAction};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_erinyes_config() {
        let config = ErinyesConfig::default();
        assert_eq!(config.heartbeat_interval_ms, 500);
        assert_eq!(config.watchdog_timeout_ms, 30000);
        assert!(config.auto_recovery_enabled);
        assert!(config.alerts_enabled);
    }
    
    #[test]
    fn test_erinyes_config_builder() {
        let config = ErinyesConfig::new()
            .with_heartbeat_interval(1000)
            .with_watchdog_timeout(60000)
            .disable_auto_recovery()
            .disable_alerts();
            
        assert_eq!(config.heartbeat_interval_ms, 1000);
        assert_eq!(config.watchdog_timeout_ms, 60000);
        assert!(!config.auto_recovery_enabled);
        assert!(!config.alerts_enabled);
    }
}

/// Tests de heartbeat
#[cfg(test)]
mod heartbeat_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_heartbeat_registration() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        erinyes.register_actor(GodName::Zeus).await.expect("Registration failed");
        
        let actors = erinyes.get_monitored_actors().await;
        assert!(actors.contains(&GodName::Zeus));
    }
    
    #[tokio::test]
    async fn test_heartbeat_reception() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        erinyes.register_actor(GodName::Hermes).await.unwrap();
        
        // Enviar heartbeat
        erinyes.receive_heartbeat(GodName::Hermes).await.expect("Heartbeat failed");
        
        let status = erinyes.get_actor_status(GodName::Hermes).await;
        assert!(status.is_healthy());
        assert!(status.last_heartbeat.elapsed().as_millis() < 100);
    }
    
    #[tokio::test]
    async fn test_missed_heartbeat_detection() {
        let config = ErinyesConfig::new()
            .with_heartbeat_interval(100)
            .with_watchdog_timeout(200);
        
        let erinyes = Erinyes::with_config(config).await.expect("Failed to create Erinyes");
        
        erinyes.register_actor(GodName::Apollo).await.unwrap();
        
        // Enviar un heartbeat inicial
        erinyes.receive_heartbeat(GodName::Apollo).await.unwrap();
        
        // Esperar a que expire
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        let status = erinyes.get_actor_status(GodName::Apollo).await;
        assert!(!status.is_healthy());
        assert!(status.missed_heartbeats > 0);
    }
    
    #[tokio::test]
    async fn test_heartbeat_frequency_check() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        erinyes.register_actor(GodName::Athena).await.unwrap();
        
        // Enviar heartbeats consistentes
        for _ in 0..5 {
            erinyes.receive_heartbeat(GodName::Athena).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        let stats = erinyes.get_heartbeat_stats(GodName::Athena).await;
        assert_eq!(stats.total_heartbeats, 5);
        assert!(stats.average_interval_ms > 0);
    }
}

/// Tests de watchdog
#[cfg(test)]
mod watchdog_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_watchdog_kick() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let actor_id = "test-actor";
        
        erinyes.start_watchdog(actor_id, 1000).await.expect("Watchdog start failed");
        
        // Kick watchdog
        erinyes.kick_watchdog(actor_id).await.expect("Watchdog kick failed");
        
        let watchdog = erinyes.get_watchdog(actor_id).await.expect("Watchdog not found");
        assert!(watchdog.is_active());
    }
    
    #[tokio::test]
    async fn test_watchdog_timeout() {
        let config = ErinyesConfig::new()
            .with_watchdog_timeout(200);
        
        let erinyes = Erinyes::with_config(config).await.expect("Failed to create Erinyes");
        let actor_id = "timeout-actor";
        
        erinyes.start_watchdog(actor_id, 200).await.unwrap();
        
        // No hacer kick - esperar timeout
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        let triggered = erinyes.is_watchdog_triggered(actor_id).await;
        assert!(triggered);
    }
    
    #[tokio::test]
    async fn test_watchdog_stop() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let actor_id = "stop-actor";
        
        erinyes.start_watchdog(actor_id, 1000).await.unwrap();
        assert!(erinyes.get_watchdog(actor_id).await.is_some());
        
        erinyes.stop_watchdog(actor_id).await.expect("Watchdog stop failed");
        assert!(erinyes.get_watchdog(actor_id).await.is_none());
    }
}

/// Tests de recuperación
#[cfg(test)]
mod recovery_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_recovery_strategy_restart() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let strategy = RecoveryStrategy::Restart {
            max_attempts: 3,
            backoff_delay_ms: 1000,
        };
        
        let action = erinyes.determine_recovery_action(GodName::Hermes, strategy).await;
        
        assert!(matches!(action, RecoveryAction::RestartActor { .. }));
    }
    
    #[tokio::test]
    async fn test_recovery_strategy_replace() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let strategy = RecoveryStrategy::Replace {
            preserve_state: true,
        };
        
        let action = erinyes.determine_recovery_action(GodName::Athena, strategy).await;
        
        assert!(matches!(action, RecoveryAction::ReplaceActor { .. }));
    }
    
    #[tokio::test]
    async fn test_recovery_strategy_escalate() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let strategy = RecoveryStrategy::Escalate;
        
        let action = erinyes.determine_recovery_action(GodName::Zeus, strategy).await;
        
        assert!(matches!(action, RecoveryAction::EscalateToSupervisor { .. }));
    }
    
    #[tokio::test]
    async fn test_recovery_exhaustion() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let actor = GodName::Apollo;
        
        let strategy = RecoveryStrategy::Restart {
            max_attempts: 2,
            backoff_delay_ms: 100,
        };
        
        // Intentar recuperación 3 veces
        for i in 0..3 {
            let action = erinyes.determine_recovery_action(actor, strategy.clone()).await;
            
            if i < 2 {
                assert!(matches!(action, RecoveryAction::RestartActor { .. }));
            } else {
                // Después de 2 intentos, debe escalar
                assert!(matches!(action, RecoveryAction::EscalateToSupervisor { .. }));
            }
            
            erinyes.record_recovery_attempt(actor).await;
        }
    }
    
    #[tokio::test]
    async fn test_auto_recovery_execution() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let actor = GodName::Artemis;
        
        erinyes.register_actor(actor).await.unwrap();
        
        // Simular fallo
        erinyes.simulate_failure(actor).await;
        
        // Ejecutar recuperación automática
        let result = erinyes.execute_recovery(actor).await;
        
        // La recuperación puede fallar o tener éxito, pero no debe panic
        let _ = result;
        
        // Verificar que se registró el intento
        let history = erinyes.get_recovery_history(actor).await;
        assert!(!history.is_empty());
    }
}

/// Tests de alertas
#[cfg(test)]
mod alert_tests {
    use super::*;
    use olympus::actors::erinyes::alerts::{Alert, AlertSeverity, AlertChannel};
    
    #[tokio::test]
    async fn test_alert_creation() {
        let alert = Alert::new()
            .with_title("Actor Failure")
            .with_message("Hermes has failed 3 times")
            .with_severity(AlertSeverity::Critical)
            .with_actor(GodName::Hermes);
        
        assert_eq!(alert.title, "Actor Failure");
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert!(alert.actor.is_some());
    }
    
    #[tokio::test]
    async fn test_alert_routing() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let alert = Alert::new()
            .with_severity(AlertSeverity::Warning);
        
        let channels = erinyes.determine_alert_channels(&alert).await;
        
        assert!(channels.contains(&AlertChannel::Log));
        assert!(!channels.contains(&AlertChannel::PagerDuty)); // Solo para critical
    }
    
    #[tokio::test]
    async fn test_critical_alert_routing() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let alert = Alert::new()
            .with_severity(AlertSeverity::Critical);
        
        let channels = erinyes.determine_alert_channels(&alert).await;
        
        assert!(channels.contains(&AlertChannel::Log));
        assert!(channels.contains(&AlertChannel::Email));
        assert!(channels.contains(&AlertChannel::Slack));
    }
    
    #[tokio::test]
    async fn test_alert_deduplication() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let alert = Alert::new()
            .with_actor(GodName::Hermes)
            .with_message("Test alert");
        
        // Enviar misma alerta 3 veces
        for _ in 0..3 {
            erinyes.send_alert(alert.clone()).await.unwrap();
        }
        
        let alert_count = erinyes.count_alerts_last_minute(GodName::Hermes).await;
        assert!(alert_count <= 2); // Debe deduplicar
    }
}

/// Tests de métricas de salud
#[cfg(test)]
mod health_metrics_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_system_health_overview() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        // Registrar varios actores
        for god in [GodName::Zeus, GodName::Hades, GodName::Hestia] {
            erinyes.register_actor(god).await.unwrap();
            erinyes.receive_heartbeat(god).await.unwrap();
        }
        
        let overview = erinyes.get_system_health_overview().await;
        
        assert_eq!(overview.total_actors, 3);
        assert_eq!(overview.healthy_actors, 3);
        assert_eq!(overview.unhealthy_actors, 0);
    }
    
    #[tokio::test]
    async fn test_health_trend() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        erinyes.register_actor(GodName::Athena).await.unwrap();
        
        // Simular tendencia de salud
        for _ in 0..5 {
            erinyes.receive_heartbeat(GodName::Athena).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        let trend = erinyes.get_health_trend(GodName::Athena, 10).await;
        
        assert!(trend.uptime_percentage > 95.0);
        assert!(trend.average_response_time_ms >= 0.0);
    }
    
    #[tokio::test]
    async fn test_failure_rate_calculation() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let actor = GodName::Apollo;
        
        erinyes.register_actor(actor).await.unwrap();
        
        // Simular 10 heartbeats y 2 fallos
        for _ in 0..10 {
            erinyes.receive_heartbeat(actor).await.unwrap();
        }
        erinyes.simulate_failure(actor).await;
        erinyes.simulate_failure(actor).await;
        
        let failure_rate = erinyes.calculate_failure_rate(actor, 100).await;
        
        assert!((failure_rate - 0.166).abs() < 0.01); // 2/12 ≈ 16.6%
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_erinyes_message_register_actor() {
        let mut erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let message = ActorMessage::register_actor(GodName::Chronos);
        let response = erinyes.handle_message(message).await;
        
        assert!(response.is_ok());
        
        let actors = erinyes.get_monitored_actors().await;
        assert!(actors.contains(&GodName::Chronos));
    }
    
    #[tokio::test]
    async fn test_erinyes_message_heartbeat() {
        let mut erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        erinyes.register_actor(GodName::Ares).await.unwrap();
        
        let message = ActorMessage::heartbeat(GodName::Ares);
        let response = erinyes.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_erinyes_message_health_check() {
        let mut erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let message = ActorMessage::system_health_request();
        let response = erinyes.handle_message(message).await;
        
        assert!(response.is_ok());
        let payload = response.unwrap();
        assert!(payload.contains_health_info());
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_heartbeat_processing_performance() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let count = 10000;
        
        // Registrar actor
        erinyes.register_actor(GodName::Zeus).await.unwrap();
        
        let start = Instant::now();
        
        for _ in 0..count {
            erinyes.receive_heartbeat(GodName::Zeus).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let heartbeats_per_sec = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            heartbeats_per_sec > 50000.0,
            "Heartbeat processing too slow: {:.0} hb/sec",
            heartbeats_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_concurrent_health_checks() {
        use tokio::task;
        
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        let erinyes = std::sync::Arc::new(tokio::sync::Mutex::new(erinyes));
        
        // Registrar múltiples actores
        for god in [GodName::Zeus, GodName::Hades, GodName::Hestia] {
            erinyes.lock().await.register_actor(god).await.unwrap();
        }
        
        let mut handles = vec![];
        
        for _ in 0..100 {
            let erinyes_clone = erinyes.clone();
            let handle = task::spawn(async move {
                erinyes_clone.lock().await.get_system_health_overview().await
            });
            handles.push(handle);
        }
        
        let start = Instant::now();
        
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        let elapsed = start.elapsed();
        
        assert!(
            elapsed.as_millis() < 1000,
            "Concurrent health checks too slow: {:?}",
            elapsed
        );
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_erinyes_creation() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        assert_eq!(erinyes.name(), GodName::Erinyes);
        assert_eq!(erinyes.domain(), DivineDomain::Monitoring);
    }
    
    #[tokio::test]
    async fn test_erinyes_health_check() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        let health = erinyes.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_erinyes_shutdown_graceful() {
        let erinyes = Erinyes::new().await.expect("Failed to create Erinyes");
        
        // Registrar algunos actores
        erinyes.register_actor(GodName::Zeus).await.unwrap();
        erinyes.start_watchdog("test", 1000).await.unwrap();
        
        // Shutdown
        erinyes.shutdown().await.expect("Shutdown failed");
        
        // Verificar que se limpiaron los recursos
        let actors = erinyes.get_monitored_actors().await;
        assert!(actors.is_empty());
    }
}
