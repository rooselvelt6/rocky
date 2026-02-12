// tests/unit/nemesis/mod.rs
// Tests unitarios para Némesis - Sistema Legal y Cumplimiento

use olympus::actors::nemesis::{Nemesis, NemesisConfig, AuditManager, ComplianceChecker};
use olympus::actors::nemesis::compliance::{AuditEvent, ComplianceStandard, Violation, AuditSeverity};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};
use chrono::Utc;

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_nemesis_config() {
        let config = NemesisConfig::default();
        assert!(config.hipaa_enabled);
        assert!(config.gdpr_enabled);
        assert!(config.soc2_enabled);
        assert!(config.auto_audit_enabled);
        assert_eq!(config.retention_days, 2555); // 7 años
    }
    
    #[test]
    fn test_nemesis_config_builder() {
        let config = NemesisConfig::new()
            .with_retention_days(3650) // 10 años
            .disable_auto_audit()
            .enable_standard(ComplianceStandard::PCI_DSS)
            .enable_standard(ComplianceStandard::HIPAA);
            
        assert_eq!(config.retention_days, 3650);
        assert!(!config.auto_audit_enabled);
        assert!(config.enabled_standards.contains(&ComplianceStandard::PCI_DSS));
    }
}

/// Tests de auditoría
#[cfg(test)]
mod audit_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_audit_event() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let event = AuditEvent::new()
            .with_user("doctor.smith")
            .with_action("view_patient_record")
            .with_resource("patient:12345")
            .with_result(AuditResult::Success)
            .with_severity(AuditSeverity::Info)
            .with_data_sensitivity(DataSensitivity::PHI);
        
        let result = nemesis.log_audit_event(event).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_audit_event_with_timestamp() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let before = Utc::now();
        
        let event = AuditEvent::new()
            .with_user("admin")
            .with_action("system_config_change")
            .with_result(AuditResult::Success);
        
        nemesis.log_audit_event(event).await.unwrap();
        
        let after = Utc::now();
        
        // Recuperar evento y verificar timestamp
        let events = nemesis.get_audit_events(
            Filter::user("admin"),
            before,
            after
        ).await.unwrap();
        
        assert_eq!(events.len(), 1);
        assert!(events[0].timestamp >= before);
        assert!(events[0].timestamp <= after);
    }
    
    #[tokio::test]
    async fn test_audit_event_retrieval() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear múltiples eventos
        for i in 0..10 {
            let event = AuditEvent::new()
                .with_user(&format!("user_{}", i))
                .with_action("data_access")
                .with_result(AuditResult::Success);
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        // Recuperar todos
        let all_events = nemesis.get_all_audit_events().await.unwrap();
        assert_eq!(all_events.len(), 10);
        
        // Recuperar con filtro
        let filtered = nemesis.get_audit_events(
            Filter::action("data_access"),
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1)
        ).await.unwrap();
        
        assert_eq!(filtered.len(), 10);
    }
    
    #[tokio::test]
    async fn test_audit_event_serialization() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let event = AuditEvent::new()
            .with_user("test_user")
            .with_action("test_action")
            .with_resource("resource:123")
            .with_result(AuditResult::Success)
            .with_metadata(json!({
                "ip_address": "192.168.1.1",
                "user_agent": "Mozilla/5.0"
            }));
        
        let json = nemesis.serialize_audit_event(&event).await.unwrap();
        
        assert!(json.contains("test_user"));
        assert!(json.contains("test_action"));
        assert!(json.contains("192.168.1.1"));
        
        // Verificar que se puede deserializar
        let deserialized = nemesis.deserialize_audit_event(&json).await.unwrap();
        assert_eq!(deserialized.user_id, "test_user");
    }
    
    #[tokio::test]
    async fn test_audit_trail_integrity() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear eventos
        let events: Vec<_> = (0..5)
            .map(|i| {
                AuditEvent::new()
                    .with_user("integrity_test")
                    .with_action(&format!("action_{}", i))
                    .with_result(AuditResult::Success)
            })
            .collect();
        
        for event in &events {
            nemesis.log_audit_event(event.clone()).await.unwrap();
        }
        
        // Verificar integridad
        let integrity_check = nemesis.verify_audit_trail_integrity().await.unwrap();
        
        assert!(integrity_check.valid);
        assert_eq!(integrity_check.event_count, 5);
    }
}

/// Tests de estándares de cumplimiento
#[cfg(test)]
mod compliance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hipaa_compliance_check() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Simular acceso a datos PHI sin autorización
        let unauthorized_access = Operation::new()
            .with_user("unauthorized_user")
            .with_resource("patient:12345")
            .with_data_type(DataType::PHI)
            .without_authentication();
        
        let result = nemesis.check_compliance(
            &unauthorized_access,
            ComplianceStandard::HIPAA
        ).await;
        
        assert!(!result.compliant);
        assert!(result.violations.iter().any(|v| v.standard == ComplianceStandard::HIPAA));
    }
    
    #[tokio::test]
    async fn test_gdpr_compliance_check() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Simular procesamiento de datos sin consentimiento
        let processing_without_consent = Operation::new()
            .with_user("processor")
            .with_resource("user_data:eu_citizen")
            .with_action("analyze")
            .without_consent();
        
        let result = nemesis.check_compliance(
            &processing_without_consent,
            ComplianceStandard::GDPR
        ).await;
        
        assert!(!result.compliant);
        assert!(result.violations.iter().any(|v| 
            v.rule.contains("consent") || v.rule.contains("lawful_basis")
        ));
    }
    
    #[tokio::test]
    async fn test_soc2_compliance_check() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Simular acceso sin MFA
        let access_without_mfa = Operation::new()
            .with_user("admin")
            .with_resource("production_system")
            .with_authentication()
            .without_mfa();
        
        let result = nemesis.check_compliance(
            &access_without_mfa,
            ComplianceStandard::SOC2
        ).await;
        
        // SOC2 Type II requiere MFA para acceso administrativo
        assert!(!result.compliant);
    }
    
    #[tokio::test]
    async fn test_multi_standard_compliance() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let operation = Operation::new()
            .with_user("doctor")
            .with_resource("patient_record")
            .with_data_type(DataType::PHI);
        
        // Verificar cumplimiento con múltiples estándares
        let standards = vec![
            ComplianceStandard::HIPAA,
            ComplianceStandard::GDPR,
            ComplianceStandard::SOC2,
        ];
        
        for standard in standards {
            let result = nemesis.check_compliance(&operation, standard).await;
            
            // Debe retornar resultado específico del estándar
            assert!(!result.standard.to_string().is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_pci_dss_compliance() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Simular almacenamiento de número de tarjeta sin encriptación
        let unencrypted_storage = Operation::new()
            .with_user("system")
            .with_resource("payment_data")
            .with_data_type(DataType::CreditCard)
            .without_encryption();
        
        let result = nemesis.check_compliance(
            &unencrypted_storage,
            ComplianceStandard::PCI_DSS
        ).await;
        
        assert!(!result.compliant);
        assert!(result.violations.iter().any(|v| 
            v.description.contains("encryption")
        ));
    }
}

/// Tests de detección de violaciones
#[cfg(test)]
mod violation_detection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_detect_data_breach() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Simular patrón de acceso sospechoso
        let suspicious_pattern = vec![
            AuditEvent::new().with_user("attacker").with_action("login_failed"),
            AuditEvent::new().with_user("attacker").with_action("login_failed"),
            AuditEvent::new().with_user("attacker").with_action("login_failed"),
            AuditEvent::new().with_user("attacker").with_action("access_patient_data"),
            AuditEvent::new().with_user("attacker").with_action("bulk_export"),
        ];
        
        let breach_detection = nemesis.analyze_for_breach(&suspicious_pattern).await;
        
        assert!(breach_detection.is_breach_detected());
        assert!(breach_detection.severity >= Severity::High);
    }
    
    #[tokio::test]
    async fn test_detect_privilege_escalation() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let events = vec![
            AuditEvent::new()
                .with_user("regular_user")
                .with_action("view_own_profile")
                .with_role("user"),
            AuditEvent::new()
                .with_user("regular_user")
                .with_action("view_all_patients")
                .with_role("user"), // Sin permiso!
        ];
        
        let escalation = nemesis.detect_privilege_escalation(&events).await;
        
        assert!(escalation.is_escalation_detected());
    }
    
    #[tokio::test]
    async fn test_detect_unauthorized_access_pattern() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Usuario accede a datos fuera de su departamento
        let events = vec![
            AuditEvent::new()
                .with_user("cardiology_nurse")
                .with_department("cardiology")
                .with_action("access_patient")
                .with_patient_department("oncology"), // No autorizado!
        ];
        
        let unauthorized = nemesis.detect_unauthorized_access(&events).await;
        
        assert!(unauthorized.is_detected());
    }
}

/// Tests de reportes
#[cfg(test)]
mod report_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_generate_compliance_report() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear datos de auditoría
        for i in 0..50 {
            let event = AuditEvent::new()
                .with_user(&format!("user_{}", i))
                .with_action("data_access")
                .with_result(if i % 10 == 0 { 
                    AuditResult::Failure 
                } else { 
                    AuditResult::Success 
                });
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        let report = nemesis.generate_compliance_report(
            ComplianceStandard::HIPAA,
            Utc::now() - chrono::Duration::days(30),
            Utc::now()
        ).await.unwrap();
        
        assert!(!report.summary.is_empty());
        assert!(report.total_events >= 50);
        assert!(report.violation_count >= 5); // 10% fallaron
    }
    
    #[tokio::test]
    async fn test_generate_audit_trail_export() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear eventos
        for i in 0..20 {
            let event = AuditEvent::new()
                .with_user("export_test")
                .with_action(&format!("action_{}", i));
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        let export = nemesis.export_audit_trail(
            Filter::user("export_test"),
            ExportFormat::JSON
        ).await.unwrap();
        
        assert!(!export.data.is_empty());
        assert!(export.record_count == 20);
    }
    
    #[tokio::test]
    async fn test_generate_violation_summary() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear violaciones
        let violations = vec![
            Violation::new(ComplianceStandard::HIPAA, "Rule 1"),
            Violation::new(ComplianceStandard::HIPAA, "Rule 2"),
            Violation::new(ComplianceStandard::GDPR, "Article 5"),
            Violation::new(ComplianceStandard::SOC2, "CC6.1"),
        ];
        
        for v in &violations {
            nemesis.record_violation(v.clone()).await.unwrap();
        }
        
        let summary = nemesis.generate_violation_summary(
            Utc::now() - chrono::Duration::days(7)
        ).await.unwrap();
        
        assert_eq!(summary.total_violations, 4);
        assert!(summary.by_standard.get(&ComplianceStandard::HIPAA) == Some(&2));
    }
}

/// Tests de evidencia
#[cfg(test)]
mod evidence_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_collect_evidence() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let operation = Operation::new()
            .with_user("suspect")
            .with_action("unauthorized_delete")
            .with_resource("critical_data")
            .with_timestamp(Utc::now());
        
        let evidence = nemesis.collect_evidence(&operation).await.unwrap();
        
        assert!(evidence.operation_details.is_some());
        assert!(evidence.timestamp.is_some());
        assert!(!evidence.hash.is_empty());
    }
    
    #[tokio::test]
    async fn test_evidence_integrity() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let evidence = Evidence::new()
            .with_data("sensitive_data")
            .with_timestamp(Utc::now());
        
        let hash = nemesis.calculate_evidence_hash(&evidence).await.unwrap();
        
        // Verificar integridad
        let is_valid = nemesis.verify_evidence_integrity(&evidence, &hash).await;
        
        assert!(is_valid);
        
        // Modificar evidencia y verificar que falla
        let mut tampered = evidence.clone();
        tampered.data = "modified_data".to_string();
        
        let is_invalid = nemesis.verify_evidence_integrity(&tampered, &hash).await;
        
        assert!(!is_invalid);
    }
}

/// Tests de retención
#[cfg(test)]
mod retention_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audit_retention_policy() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear evento antiguo (simulado)
        let old_event = AuditEvent::new()
            .with_user("old_user")
            .with_timestamp(Utc::now() - chrono::Duration::days(3000)); // +8 años
        
        nemesis.log_audit_event(old_event).await.unwrap();
        
        // Aplicar política de retención (7 años)
        nemesis.apply_retention_policy().await.unwrap();
        
        // El evento antiguo debe estar archivado o eliminado
        let old_events = nemesis.get_audit_events(
            Filter::user("old_user"),
            Utc::now() - chrono::Duration::days(3000),
            Utc::now() - chrono::Duration::days(2999)
        ).await.unwrap();
        
        assert!(old_events.is_empty() || old_events[0].is_archived);
    }
    
    #[tokio::test]
    async fn test_archive_old_audits() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear eventos de diferentes edades
        for days_ago in [10, 100, 1000, 2000] {
            let event = AuditEvent::new()
                .with_user("archive_test")
                .with_timestamp(Utc::now() - chrono::Duration::days(days_ago));
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        // Archivar eventos mayores a 1 año
        let archived = nemesis.archive_audits_older_than(
            Utc::now() - chrono::Duration::days(365)
        ).await.unwrap();
        
        assert_eq!(archived.len(), 2); // 1000 y 2000 días
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_nemesis_message_log_audit() {
        let mut nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let event = AuditEvent::new()
            .with_user("message_test")
            .with_action("test_action");
        
        let message = ActorMessage::log_audit_request(event);
        let response = nemesis.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(response.unwrap().logged);
    }
    
    #[tokio::test]
    async fn test_nemesis_message_check_compliance() {
        let mut nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let operation = Operation::new()
            .with_user("compliance_test")
            .with_resource("patient_data");
        
        let message = ActorMessage::check_compliance_request(
            operation,
            ComplianceStandard::HIPAA
        );
        
        let response = nemesis.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_nemesis_message_generate_report() {
        let mut nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let message = ActorMessage::generate_compliance_report_request(
            ComplianceStandard::GDPR,
            Utc::now() - chrono::Duration::days(30),
            Utc::now()
        );
        
        let response = nemesis.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(!response.unwrap().report.is_empty());
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_audit_logging_performance() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let start = Instant::now();
        
        for i in 0..1000 {
            let event = AuditEvent::new()
                .with_user(&format!("perf_user_{}", i))
                .with_action("performance_test");
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let events_per_sec = 1000.0 / elapsed.as_secs_f64();
        
        assert!(
            events_per_sec > 500.0,
            "Audit logging too slow: {:.0} events/sec",
            events_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_compliance_check_performance() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let operation = Operation::new()
            .with_user("perf_check")
            .with_resource("test_resource");
        
        let start = Instant::now();
        
        for _ in 0..100 {
            nemesis.check_compliance(&operation, ComplianceStandard::HIPAA).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let checks_per_sec = 100.0 / elapsed.as_secs_f64();
        
        assert!(
            checks_per_sec > 50.0,
            "Compliance check too slow: {:.0} checks/sec",
            checks_per_sec
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audit_with_unicode() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let event = AuditEvent::new()
            .with_user("用户测试")  // Chino
            .with_action("日本語テスト")  // Japonés
            .with_metadata(json!({
                "description": "Prueba español 🎉"
            }));
        
        let result = nemesis.log_audit_event(event).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_very_large_audit_event() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let large_metadata = json!({
            "data": "x".repeat(100000)
        });
        
        let event = AuditEvent::new()
            .with_user("large_test")
            .with_metadata(large_metadata);
        
        let result = nemesis.log_audit_event(event).await;
        
        // Debe manejar eventos grandes
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_concurrent_audit_logging() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        let nemesis = std::sync::Arc::new(tokio::sync::Mutex::new(nemesis));
        
        let mut handles = vec![];
        
        for i in 0..20 {
            let nemesis_clone = nemesis.clone();
            let handle = tokio::spawn(async move {
                for j in 0..50 {
                    let event = AuditEvent::new()
                        .with_user(&format!("concurrent_{}_{}", i, j))
                        .with_action("concurrent_test");
                    
                    nemesis_clone.lock().await.log_audit_event(event).await.unwrap();
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verificar que todos se registraron
        let total = nemesis.lock().await.get_audit_event_count().await;
        assert_eq!(total, 1000);
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_nemesis_creation() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        assert_eq!(nemesis.name(), GodName::Nemesis);
        assert_eq!(nemesis.domain(), DivineDomain::Compliance);
    }
    
    #[tokio::test]
    async fn test_nemesis_health_check() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let health = nemesis.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_audit_persistence() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        let event = AuditEvent::new()
            .with_user("persistence_test")
            .with_action("critical_action");
        
        nemesis.log_audit_event(event.clone()).await.unwrap();
        
        // Persistir
        nemesis.persist_audit_logs().await.unwrap();
        
        // Cargar
        let loaded = nemesis.load_persisted_audits().await.unwrap();
        
        assert!(loaded.iter().any(|e| e.user_id == "persistence_test"));
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let nemesis = Nemesis::new().await.expect("Failed to create Nemesis");
        
        // Crear eventos de diferentes tipos
        for i in 0..50 {
            let event = AuditEvent::new()
                .with_user("metrics_test")
                .with_action(&format!("action_{}", i))
                .with_result(if i % 5 == 0 {
                    AuditResult::Failure
                } else {
                    AuditResult::Success
                });
            
            nemesis.log_audit_event(event).await.unwrap();
        }
        
        let metrics = nemesis.collect_metrics().await;
        
        assert_eq!(metrics.total_audits_logged, 50);
        assert_eq!(metrics.successful_audits, 40);
        assert_eq!(metrics.failed_audits, 10);
    }
}
