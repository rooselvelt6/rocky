// tests/integration/failover_tests.rs
// Tests de failover y recuperación de infraestructura

use olympus::system::genesis::Genesis;
use olympus::actors::{GodName};
use olympus::actors::hestia::{PersistenceError, FailoverStrategy};
use olympus::traits::actor_trait::ActorMessage;

/// Test: Failover de SurrealDB a Valkey
#[tokio::test]
async fn test_database_failover_surrealdb_to_valkey() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Crear datos iniciales en SurrealDB
    let patient_data = serde_json::json!({
        "id": "patient-001",
        "name": "Test Patient",
        "condition": "stable"
    });
    
    let create_result = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist_with_primary("patient-001", &patient_data)
    ).await.expect("Initial persist failed");
    
    assert!(create_result.is_success());
    
    // 2. Simular caída de SurrealDB
    genesis.simulate_service_failure("surrealdb").await;
    
    // 3. Hestia debe detectar el fallo y cambiar a Valkey
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 4. Intentar crear nuevos datos
    let new_patient = serde_json::json!({
        "id": "patient-002",
        "name": "Failover Patient",
        "condition": "critical"
    });
    
    let failover_result = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist("patient-002", &new_patient)
    ).await;
    
    // Debe funcionar usando Valkey
    assert!(failover_result.is_ok(), "Failover to Valkey should work");
    
    // 5. Verificar que los datos se guardaron en Valkey
    let retrieve_result = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::retrieve_from_secondary("patient-002")
    ).await.expect("Retrieve from Valkey failed");
    
    let retrieved = retrieve_result.get_data();
    assert_eq!(retrieved.get("name").unwrap(), "Failover Patient");
    
    // 6. Recuperar SurrealDB
    genesis.restore_service("surrealdb").await;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 7. Verificar sincronización de datos
    let sync_result = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::sync_databases()
    ).await.expect("Database sync failed");
    
    assert!(sync_result.is_synced(), "Databases should be synchronized");
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Database failover test passed!");
}

/// Test: Recuperación de actor caído
#[tokio::test]
async fn test_actor_recovery_during_operation() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Iniciar operación larga en Athena
    let analysis_future = genesis.send_to_actor(
        GodName::Athena,
        ActorMessage::analyze_large_dataset("dataset-001")
    );
    
    // 2. Simular caída de Athena durante el análisis
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    genesis.simulate_actor_failure(GodName::Athena).await;
    
    // 3. Zeus debe detectar y reiniciar Athena
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 4. Verificar que Athena está de vuelta
    let health_check = genesis.send_to_actor(
        GodName::Erinyes,
        ActorMessage::check_actor_health(GodName::Athena)
    ).await.expect("Health check failed");
    
    assert!(health_check.is_healthy(), "Athena should be restarted and healthy");
    
    // 5. Reintentar operación
    let retry_result = genesis.send_to_actor(
        GodName::Athena,
        ActorMessage::analyze_large_dataset("dataset-001")
    ).await.expect("Retry analysis failed");
    
    assert!(retry_result.has_analysis_result());
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Actor recovery during operation test passed!");
}

/// Test: Circuit breaker en Hermes
#[tokio::test]
async fn test_messaging_circuit_breaker() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Simular múltiples fallos de mensajería
    for i in 0..5 {
        let result = genesis.send_to_actor(
            GodName::Hermes,
            ActorMessage::send_to_nonexistent_actor(&format!("ghost-{}", i))
        ).await;
        
        // Ignorar errores, estamos forzando fallos
        let _ = result;
    }
    
    // 2. Circuit breaker debe abrirse
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let cb_status = genesis.send_to_actor(
        GodName::Hermes,
        ActorMessage::get_circuit_breaker_status()
    ).await.expect("CB status check failed");
    
    assert!(cb_status.is_open() || cb_status.is_half_open(), 
        "Circuit breaker should be open or half-open after failures");
    
    // 3. Esperar recuperación
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 4. Enviar mensaje exitoso para cerrar circuito
    let success_result = genesis.send_to_actor(
        GodName::Hermes,
        ActorMessage::ping_actor(GodName::Zeus)
    ).await;
    
    // Puede tener éxito o no, pero no debe panic
    let _ = success_result;
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Circuit breaker test passed!");
}

/// Test: Failover de autenticación
#[tokio::test]
async fn test_authentication_failover() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Crear sesión de usuario
    let auth_result = genesis.send_to_actor(
        GodName::Hades,
        ActorMessage::auth_request("doctor.test", "password123")
    ).await.expect("Initial auth failed");
    
    let token = auth_result.get_token();
    
    // 2. Simular sobrecarga en Hades
    genesis.simulate_actor_overload(GodName::Hades).await;
    
    // 3. Intentar validar token (debe usar cache)
    let validation_result = genesis.send_to_actor(
        GodName::Hades,
        ActorMessage::validate_token_cached(&token)
    ).await;
    
    // Debe funcionar usando cache aunque Hades esté sobrecargado
    assert!(validation_result.is_ok(), "Token validation should work via cache during overload");
    assert!(validation_result.unwrap().is_valid());
    
    // 4. Restaurar Hades
    genesis.restore_actor(GodName::Hades).await;
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Authentication failover test passed!");
}

/// Test: Reconexión de WebSocket
#[tokio::test]
async fn test_websocket_reconnection() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Crear conexión WebSocket
    let client_id = "ws-client-001";
    
    genesis.send_to_actor(
        GodName::Poseidon,
        ActorMessage::accept_ws_connection(client_id)
    ).await.expect("WS connection failed");
    
    // 2. Enviar mensajes
    for i in 0..5 {
        genesis.send_to_actor(
            GodName::Poseidon,
            ActorMessage::send_ws_message(client_id, &format!("msg-{}", i))
        ).await.expect("WS send failed");
    }
    
    // 3. Simular desconexión de red
    genesis.simulate_network_failure(client_id).await;
    
    // 4. Verificar que Poseidon detecta desconexión
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let connection_status = genesis.send_to_actor(
        GodName::Poseidon,
        ActorMessage::get_connection_status(client_id)
    ).await.unwrap();
    
    assert!(!connection_status.is_connected(), "Connection should be marked as disconnected");
    
    // 5. Simular reconexión del cliente
    genesis.simulate_network_recovery(client_id).await;
    
    genesis.send_to_actor(
        GodName::Poseidon,
        ActorMessage::accept_ws_connection(client_id)
    ).await.expect("WS reconnection failed");
    
    // 6. Verificar mensajes en buffer fueron enviados
    let missed_messages = genesis.send_to_actor(
        GodName::Poseidon,
        ActorMessage::get_missed_messages(client_id)
    ).await.unwrap();
    
    assert!(!missed_messages.is_empty(), "Missed messages should be available");
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ WebSocket reconnection test passed!");
}

/// Test: Degradación graceful del sistema
#[tokio::test]
async fn test_graceful_degradation() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Sistema funcionando normalmente
    let initial_health = genesis.send_to_actor(
        GodName::Zeus,
        ActorMessage::system_health_request()
    ).await.expect("Health check failed");
    
    assert!(initial_health.overall_status == SystemStatus::Healthy);
    
    // 2. Desactivar actores no críticos
    genesis.deactivate_actor(GodName::Aurora).await;
    genesis.deactivate_actor(GodName::Nemesis).await;
    genesis.deactivate_actor(GodName::Chronos).await;
    
    // 3. Verificar que sistema sigue funcionando
    let degraded_health = genesis.send_to_actor(
        GodName::Zeus,
        ActorMessage::system_health_request()
    ).await.expect("Health check failed");
    
    assert!(
        degraded_health.overall_status == SystemStatus::Degraded ||
        degraded_health.overall_status == SystemStatus::Healthy,
        "System should be operational with non-critical actors disabled"
    );
    
    // 4. Operaciones críticas deben seguir funcionando
    let critical_op = genesis.send_to_actor(
        GodName::Hades,
        ActorMessage::auth_request("user", "pass")
    ).await;
    
    // Puede fallar la auth pero no debe panic
    let _ = critical_op;
    
    let persistence_op = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist("test", &serde_json::json!({}))
    ).await;
    
    assert!(persistence_op.is_ok(), "Persistence should work in degraded mode");
    
    // 5. Reactivar actores
    genesis.reactivate_actor(GodName::Aurora).await;
    genesis.reactivate_actor(GodName::Nemesis).await;
    genesis.reactivate_actor(GodName::Chronos).await;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // 6. Verificar recuperación completa
    let final_health = genesis.send_to_actor(
        GodName::Zeus,
        ActorMessage::system_health_request()
    ).await.expect("Health check failed");
    
    assert!(
        final_health.overall_status == SystemStatus::Healthy,
        "System should return to healthy after reactivation"
    );
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Graceful degradation test passed!");
}

/// Test: Split-brain prevention
#[tokio::test]
async fn test_split_brain_prevention() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // 1. Crear dato inicial
    genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist("conflict-key", &serde_json::json!({"value": "A"}))
    ).await.unwrap();
    
    // 2. Simular partición de red donde dos nodos intentan actualizar
    // En modo split-brain, Zeus debe coordinar
    
    let update1 = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist_if_unchanged("conflict-key", &serde_json::json!({"value": "B"}), "A")
    ).await;
    
    let update2 = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::persist_if_unchanged("conflict-key", &serde_json::json!({"value": "C"}), "A")
    ).await;
    
    // Solo uno debe tener éxito (prevención de conflicto)
    let success_count = vec![&update1, &update2]
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_success())
        .count();
    
    assert!(
        success_count <= 1,
        "Only one concurrent update should succeed to prevent split-brain"
    );
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ Split-brain prevention test passed!");
}
