// tests/integration/actor_interaction.rs
// Tests de interacción entre múltiples actores

use super::*;
use olympus::traits::actor_trait::{ActorMessage, MessagePayload};

/// Test: Mensaje atraviesa 5 actores correctamente
#[tokio::test]
async fn test_message_flow_through_multiple_actors() {
    let genesis = setup_test_olympus().await;
    
    // Asegurar que todos los actores están listos
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    // Crear mensaje que debe pasar por:
    // Apollo (event) -> Hermes (route) -> Athena (analyze) -> Hestia (store) -> Zeus (log)
    let patient_data = MessagePayload::patient_data_test();
    let message = ActorMessage::clinical_event(patient_data);
    
    // Enviar a Apollo
    let result = genesis.send_to_actor(GodName::Apollo, message).await;
    assert!(result.is_ok());
    
    // Esperar procesamiento
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Verificar que Athena procesó el análisis
    let athena_status = genesis.get_actor_status(GodName::Athena).await;
    assert!(athena_status.messages_processed > 0);
    
    // Verificar que Hestia almacenó
    let hestia_status = genesis.get_actor_status(GodName::Hestia).await;
    assert!(hestia_status.messages_processed > 0);
    
    teardown_test_olympus(genesis).await;
}

/// Test: Múltiples actores responden a broadcast
#[tokio::test]
async fn test_broadcast_to_multiple_actors() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    let message = ActorMessage::system_status_request();
    let targets = vec![
        GodName::Zeus,
        GodName::Hades,
        GodName::Hestia,
        GodName::Hermes,
    ];
    
    let responses = genesis.broadcast_and_collect(message, &targets, 5000).await;
    
    assert_eq!(responses.len(), 4);
    
    for (god, response) in responses {
        assert!(
            response.is_ok(),
            "Actor {:?} failed to respond",
            god
        );
    }
    
    teardown_test_olympus(genesis).await;
}

/// Test: Secuencia de operaciones cruzadas
#[tokio::test]
async fn test_cross_actor_operation_sequence() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    // Secuencia: Autenticación -> Validación -> Procesamiento -> Almacenamiento -> Auditoría
    
    // 1. Autenticación (Hades)
    let auth_msg = ActorMessage::auth_request("doctor1", "password123");
    let auth_result = genesis.send_to_actor(GodName::Hades, auth_msg).await;
    assert!(auth_result.is_ok());
    let token = auth_result.unwrap().get_token();
    
    // 2. Validación de datos (Hera)
    let validation_msg = ActorMessage::validate_patient_data(test_patient_data());
    let validation_result = genesis.send_to_actor(GodName::Hera, validation_msg).await;
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap().is_valid());
    
    // 3. Procesamiento (Athena)
    let analysis_msg = ActorMessage::analyze_patient(test_patient_data());
    let analysis_result = genesis.send_to_actor(GodName::Athena, analysis_msg).await;
    assert!(analysis_result.is_ok());
    
    // 4. Almacenamiento (Hestia)
    let store_msg = ActorMessage::persist_data("patient:123", test_patient_data());
    let store_result = genesis.send_to_actor(GodName::Hestia, store_msg).await;
    assert!(store_result.is_ok());
    
    // 5. Auditoría (Némesis)
    let audit_msg = ActorMessage::audit_event("patient_created", "doctor1");
    let audit_result = genesis.send_to_actor(GodName::Nemesis, audit_msg).await;
    assert!(audit_result.is_ok());
    
    teardown_test_olympus(genesis).await;
}

/// Test: Actores del mismo dominio colaboran
#[tokio::test]
async fn test_same_domain_actor_collaboration() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    // Dominio: Inteligencia (Athena, Apollo, Artemis)
    let patient_data = test_patient_data();
    
    // Athena analiza
    let analysis = genesis.send_to_actor(
        GodName::Athena,
        ActorMessage::analyze_patient(patient_data.clone())
    ).await.unwrap();
    
    // Apollo emite evento con resultado
    let event_msg = ActorMessage::event_with_payload("analysis_complete", analysis.payload());
    genesis.send_to_actor(GodName::Apollo, event_msg).await.unwrap();
    
    // Artemis indexa para búsqueda
    let index_msg = ActorMessage::index_document("analysis:123", analysis.payload());
    genesis.send_to_actor(GodName::Artemis, index_msg).await.unwrap();
    
    // Esperar procesamiento
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // Verificar que Artemis tiene el documento indexado
    let search_msg = ActorMessage::search_query("patient analysis");
    let search_result = genesis.send_to_actor(GodName::Artemis, search_msg).await;
    assert!(search_result.is_ok());
    assert!(!search_result.unwrap().results().is_empty());
    
    teardown_test_olympus(genesis).await;
}

/// Test: Manejo de dependencias circulares (o evitación de ellas)
#[tokio::test]
async fn test_no_circular_dependency_deadlock() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    // Zeus supervisa a todos, pero ningún actor debe enviar mensaje directo a Zeus
    // en respuesta a un mensaje de Zeus (evitar deadlock)
    
    let status_request = ActorMessage::status_request();
    let result = genesis.send_to_actor(GodName::Zeus, status_request).await;
    
    assert!(result.is_ok());
    // Si hay deadlock, el timeout lo detectará
    
    teardown_test_olympus(genesis).await;
}

/// Test: Preservación de contexto entre actores
#[tokio::test]
async fn test_context_preservation_across_actors() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    let context_id = "request-uuid-12345";
    let message = ActorMessage::with_context(
        MessagePayload::patient_data_test(),
        context_id,
    );
    
    // Enviar a través de múltiples actores
    let result1 = genesis.send_to_actor(GodName::Hermes, message.clone()).await;
    assert!(result1.is_ok());
    
    let result2 = genesis.send_to_actor(GodName::Athena, message.clone()).await;
    assert!(result2.is_ok());
    
    let result3 = genesis.send_to_actor(GodName::Hestia, message).await;
    assert!(result3.is_ok());
    
    // Verificar que el contexto se preservó
    assert!(result3.unwrap().contains_context(context_id));
    
    teardown_test_olympus(genesis).await;
}

/// Test: Recuperación de actores durante interacción
#[tokio::test]
async fn test_actor_recovery_during_interaction() {
    let genesis = setup_test_olympus().await;
    assert!(helpers::wait_for_all_actors(&genesis).await);
    
    // Simular fallo de Hermes
    genesis.simulate_actor_failure(GodName::Hermes).await;
    
    // Zeus debe detectar y reiniciar
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Verificar que Hermes está de vuelta
    assert!(helpers::wait_for_actor_ready(&genesis, GodName::Hermes).await);
    
    // La interacción debe continuar funcionando
    let message = ActorMessage::ping();
    let result = genesis.send_to_actor(GodName::Hermes, message).await;
    assert!(result.is_ok());
    
    teardown_test_olympus(genesis).await;
}

fn test_patient_data() -> MessagePayload {
    MessagePayload::patient_data_test()
}
