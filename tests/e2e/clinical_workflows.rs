// tests/e2e/clinical_workflows.rs
// Tests End-to-End de flujos clínicos completos

use olympus::system::genesis::Genesis;
use olympus::actors::{GodName};
use olympus::traits::actor_trait::{ActorMessage, MessagePayload};
use olympus::models::patient::Patient;

/// Test E2E: Admisión completa de paciente
#[tokio::test]
async fn test_end_to_end_patient_admission() {
    // 1. Inicializar sistema
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    
    // Esperar que todos los actores estén listos
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 2. Autenticar doctor (Hades)
    let auth_msg = ActorMessage::auth_request("doctor.smith", "secure_pass123");
    let auth_result = genesis.send_to_actor(GodName::Hades, auth_msg).await
        .expect("Auth failed");
    let token = auth_result.get_token();
    assert!(!token.is_empty(), "Token should be generated");
    
    // 3. Validar datos del paciente (Hera)
    let patient_data = Patient::new()
        .with_name("John Doe")
        .with_age(45)
        .with_medical_history(vec!["diabetes".to_string()])
        .with_current_condition("Chest pain");
    
    let validation_msg = ActorMessage::validate_patient_data(&patient_data);
    let validation_result = genesis.send_to_actor(GodName::Hera, validation_msg).await
        .expect("Validation request failed");
    
    assert!(validation_result.is_valid(), "Patient data should be valid");
    
    // 4. Crear paciente (Hestia)
    let create_msg = ActorMessage::create_patient(patient_data.clone());
    let create_result = genesis.send_to_actor(GodName::Hestia, create_msg).await
        .expect("Patient creation failed");
    
    let patient_id = create_result.get_patient_id();
    assert!(!patient_id.is_empty(), "Patient ID should be generated");
    
    // 5. Athena analiza riesgo
    let analysis_msg = ActorMessage::analyze_patient_risk(patient_id.clone());
    let analysis_result = genesis.send_to_actor(GodName::Athena, analysis_msg).await
        .expect("Risk analysis failed");
    
    let risk_score = analysis_result.get_risk_score();
    assert!(risk_score >= 0.0 && risk_score <= 1.0, "Risk score should be between 0 and 1");
    
    // 6. Emitir evento de admisión (Apollo)
    let admission_event = ActorMessage::emit_event(
        Event::patient_admission(patient_id.clone(), "doctor.smith")
    );
    genesis.send_to_actor(GodName::Apollo, admission_event).await
        .expect("Event emission failed");
    
    // 7. Indexar para búsqueda (Artemis)
    let index_msg = ActorMessage::index_patient(&patient_data, &patient_id);
    genesis.send_to_actor(GodName::Artemis, index_msg).await
        .expect("Indexing failed");
    
    // 8. Auditoría HIPAA (Némesis)
    let audit_msg = ActorMessage::audit_event(
        "PATIENT_ADMISSION",
        &patient_id,
        "doctor.smith",
        ActionType::Create
    );
    genesis.send_to_actor(GodName::Nemesis, audit_msg).await
        .expect("Audit failed");
    
    // 9. Verificar que Erinyes está monitoreando
    let health_msg = ActorMessage::check_actor_health(GodName::Hestia);
    let health_result = genesis.send_to_actor(GodName::Erinyes, health_msg).await
        .expect("Health check failed");
    
    assert!(health_result.is_healthy(), "Hestia should be healthy");
    
    // 10. Verificar que se puede recuperar el paciente
    let retrieve_msg = ActorMessage::get_patient(&patient_id);
    let retrieve_result = genesis.send_to_actor(GodName::Hestia, retrieve_msg).await
        .expect("Patient retrieval failed");
    
    let retrieved_patient = retrieve_result.get_patient();
    assert_eq!(retrieved_patient.name, "John Doe");
    assert_eq!(retrieved_patient.age, 45);
    
    // 11. Shutdown graceful
    genesis.shutdown().await.expect("Shutdown failed");
    
    println!("✅ End-to-end patient admission test passed!");
}

/// Test E2E: Flujo de emergencia completo
#[tokio::test]
async fn test_end_to_end_emergency_flow() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 1. Paciente crítico llega a emergencias
    let critical_patient = Patient::critical()
        .with_name("Emergency Patient")
        .with_vitals(Vitals::critical()
            .with_heart_rate(150.0)
            .with_blood_pressure(80.0, 40.0)
            .with_oxygen_saturation(85.0));
    
    // 2. Validación rápida (Hera)
    let validation = genesis.send_to_actor(
        GodName::Hera,
        ActorMessage::validate_emergency_data(&critical_patient)
    ).await.unwrap();
    
    assert!(validation.is_valid());
    
    // 3. Athena calcula SOFA score inmediatamente
    let sofa_result = genesis.send_to_actor(
        GodName::Athena,
        ActorMessage::calculate_sofa(&critical_patient)
    ).await.unwrap();
    
    let sofa_score = sofa_result.get_sofa_score();
    assert!(sofa_score.total > 8, "Critical patient should have high SOFA score");
    
    // 4. Si SOFA > 8, alerta inmediata (Erinyes)
    if sofa_score.total > 8 {
        let alert_msg = ActorMessage::critical_alert(
            "CRITICAL_PATIENT",
            &format!("SOFA Score: {}", sofa_score.total),
            AlertPriority::Critical
        );
        
        genesis.send_to_actor(GodName::Erinyes, alert_msg).await
            .expect("Critical alert failed");
    }
    
    // 5. Guardar en BD (Hestia)
    let patient_id = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::create_patient(critical_patient)
    ).await.unwrap().get_patient_id();
    
    // 6. Programar monitoreo continuo (Chronos)
    let schedule_msg = ActorMessage::schedule_vital_monitoring(
        &patient_id,
        Duration::from_secs(300) // Cada 5 minutos
    );
    
    genesis.send_to_actor(GodName::Chronos, schedule_msg).await
        .expect("Scheduling failed");
    
    // 7. Notificar a equipos médicos (Hermes broadcast)
    let notification = ActorMessage::broadcast_notification(
        "emergency",
        &format!("Critical patient admitted: {}", patient_id)
    );
    
    genesis.send_to_actor(
        GodName::Hermes,
        notification
    ).await.expect("Broadcast failed");
    
    // 8. Auditoría completa (Némesis)
    let audit = ActorMessage::audit_emergency_event(
        &patient_id,
        sofa_score.total,
        "Emergency admission with critical vitals"
    );
    
    genesis.send_to_actor(GodName::Nemesis, audit).await
        .expect("Emergency audit failed");
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ End-to-end emergency flow test passed!");
}

/// Test E2E: Actualización de historial médico
#[tokio::test]
async fn test_end_to_end_medical_record_update() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Setup: Crear paciente primero
    let patient = Patient::test_patient().with_name("Jane Smith");
    let patient_id = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::create_patient(patient)
    ).await.unwrap().get_patient_id();
    
    // 1. Doctor autenticado intenta actualizar
    let auth = genesis.send_to_actor(
        GodName::Hades,
        ActorMessage::auth_request("doctor.jones", "pass123")
    ).await.unwrap();
    
    let doctor_token = auth.get_token();
    
    // 2. Verificar permisos (RBAC)
    let perm_check = genesis.send_to_actor(
        GodName::Hades,
        ActorMessage::check_permission(doctor_token, "patient.write")
    ).await.unwrap();
    
    assert!(perm_check.has_permission(), "Doctor should have write permission");
    
    // 3. Crear entrada de historial
    let new_record = MedicalRecord::new()
        .with_diagnosis("Hypertension")
        .with_treatment("Lisinopril 10mg daily")
        .with_notes("Patient responding well");
    
    // 4. Validar entrada (Hera)
    let validation = genesis.send_to_actor(
        GodName::Hera,
        ActorMessage::validate_medical_record(&new_record)
    ).await.unwrap();
    
    assert!(validation.is_valid());
    
    // 5. Actualizar en BD (Hestia)
    let update_msg = ActorMessage::update_medical_record(
        &patient_id,
        new_record
    );
    
    let update_result = genesis.send_to_actor(GodName::Hestia, update_msg).await
        .expect("Record update failed");
    
    assert!(update_result.is_success());
    
    // 6. Actualizar índice de búsqueda (Artemis)
    let index_update = ActorMessage::reindex_patient(&patient_id);
    genesis.send_to_actor(GodName::Artemis, index_update).await
        .expect("Reindexing failed");
    
    // 7. Emitir evento de actualización (Apollo)
    let update_event = ActorMessage::emit_event(
        Event::record_updated(&patient_id, "doctor.jones")
    );
    
    genesis.send_to_actor(GodName::Apollo, update_event).await
        .expect("Event emission failed");
    
    // 8. Verificar integridad (Hestia + Hades hash)
    let integrity_check = genesis.send_to_actor(
        GodName::Hestia,
        ActorMessage::verify_record_integrity(&patient_id)
    ).await.unwrap();
    
    assert!(integrity_check.is_valid(), "Record integrity should be maintained");
    
    // 9. Auditoría (Némesis)
    let audit = ActorMessage::audit_event(
        "RECORD_UPDATE",
        &patient_id,
        "doctor.jones",
        ActionType::Update
    );
    
    genesis.send_to_actor(GodName::Nemesis, audit).await
        .expect("Audit failed");
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ End-to-end medical record update test passed!");
}

/// Test E2E: Consulta y análisis de datos
#[tokio::test]
async fn test_end_to_end_data_query_and_analysis() {
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Setup: Crear múltiples pacientes
    let patients = vec![
        Patient::test_patient().with_age(65).with_condition("diabetes"),
        Patient::test_patient().with_age(70).with_condition("hypertension"),
        Patient::test_patient().with_age(45).with_condition("diabetes"),
        Patient::test_patient().with_age(55).with_condition("asthma"),
    ];
    
    let mut patient_ids = vec![];
    for patient in patients {
        let id = genesis.send_to_actor(
            GodName::Hestia,
            ActorMessage::create_patient(patient)
        ).await.unwrap().get_patient_id();
        patient_ids.push(id);
    }
    
    // 1. Búsqueda de pacientes con diabetes (Artemis)
    let search_msg = ActorMessage::search_patients("diabetes");
    let search_result = genesis.send_to_actor(GodName::Artemis, search_msg).await
        .expect("Search failed");
    
    let diabetic_patients = search_result.get_patients();
    assert_eq!(diabetic_patients.len(), 2, "Should find 2 diabetic patients");
    
    // 2. Análisis estadístico (Athena)
    let stats_msg = ActorMessage::analyze_patient_cohort(&patient_ids);
    let stats_result = genesis.send_to_actor(GodName::Athena, stats_msg).await
        .expect("Statistical analysis failed");
    
    let avg_age = stats_result.get_average_age();
    assert!(avg_age > 0.0, "Average age should be calculated");
    
    // 3. Predicción de riesgo para cohorte
    let risk_analysis = genesis.send_to_actor(
        GodName::Athena,
        ActorMessage::predict_cohort_risk(&patient_ids)
    ).await.unwrap();
    
    let risk_distribution = risk_analysis.get_risk_distribution();
    assert!(!risk_distribution.is_empty());
    
    // 4. Generar reporte (Apollo + Hestia)
    let report_msg = ActorMessage::generate_report(
        "diabetes_cohort_analysis",
        &patient_ids
    );
    
    let report = genesis.send_to_actor(GodName::Athena, report_msg).await
        .expect("Report generation failed");
    
    assert!(!report.get_content().is_empty());
    
    // 5. Guardar reporte
    let save_report = ActorMessage::persist_document(
        &format!("report_{}", chrono::Utc::now().timestamp()),
        report.get_content()
    );
    
    genesis.send_to_actor(GodName::Hestia, save_report).await
        .expect("Report saving failed");
    
    // 6. Emitir evento de reporte generado
    let report_event = ActorMessage::emit_event(
        Event::report_generated("diabetes_cohort_analysis", patient_ids.len())
    );
    
    genesis.send_to_actor(GodName::Apollo, report_event).await
        .expect("Event emission failed");
    
    genesis.shutdown().await.unwrap();
    
    println!("✅ End-to-end data query and analysis test passed!");
}

/// Test E2E: Sistema bajo carga
#[tokio::test]
async fn test_end_to_end_system_under_load() {
    use tokio::task;
    
    let genesis = Genesis::new().await.expect("Failed to initialize Olympus");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let start_time = std::time::Instant::now();
    let concurrent_users = 50;
    let operations_per_user = 20;
    
    let genesis = std::sync::Arc::new(tokio::sync::Mutex::new(genesis));
    let mut handles = vec![];
    
    for user_id in 0..concurrent_users {
        let genesis_clone = genesis.clone();
        let handle = task::spawn(async move {
            let user_name = format!("user_{}", user_id);
            
            // Cada usuario realiza operaciones
            for op_id in 0..operations_per_user {
                let op_type = op_id % 4;
                
                match op_type {
                    0 => {
                        // Crear paciente
                        let patient = Patient::test_patient()
                            .with_name(&format!("Patient_{}_{}", user_id, op_id));
                        
                        let _ = genesis_clone.lock().await.send_to_actor(
                            GodName::Hestia,
                            ActorMessage::create_patient(patient)
                        ).await;
                    }
                    1 => {
                        // Validar datos
                        let data = serde_json::json!({
                            "name": format!("Test_{}", op_id),
                            "age": 30 + op_id
                        });
                        
                        let _ = genesis_clone.lock().await.send_to_actor(
                            GodName::Hera,
                            ActorMessage::validate_json(data)
                        ).await;
                    }
                    2 => {
                        // Health check
                        let _ = genesis_clone.lock().await.send_to_actor(
                            GodName::Erinyes,
                            ActorMessage::system_health_request()
                        ).await;
                    }
                    3 => {
                        // Query
                        let _ = genesis_clone.lock().await.send_to_actor(
                            GodName::Hestia,
                            ActorMessage::count_patients()
                        ).await;
                    }
                    _ => {}
                }
            }
            
            user_name
        });
        
        handles.push(handle);
    }
    
    // Esperar a que todos completen
    for handle in handles {
        let _ = handle.await.expect("Task failed");
    }
    
    let elapsed = start_time.elapsed();
    let total_operations = concurrent_users * operations_per_user;
    let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();
    
    println!(
        "✅ Load test completed: {} operations in {:?} ({:.0} ops/sec)",
        total_operations, elapsed, ops_per_second
    );
    
    // Verificar que el sistema sigue saludable
    let final_health = genesis.lock().await.send_to_actor(
        GodName::Zeus,
        ActorMessage::system_health_request()
    ).await.expect("Final health check failed");
    
    assert!(final_health.is_healthy(), "System should remain healthy after load");
    
    genesis.lock().await.shutdown().await.unwrap();
}
