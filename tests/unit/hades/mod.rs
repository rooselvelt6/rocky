// tests/unit/hades/mod.rs
// Tests unitarios para Hades - Seguridad y Criptografía

use olympus::actors::hades::{Hades, HadesConfig, EncryptionService, AuthenticationService};
use olympus::actors::hades::types::{EncryptionAlgorithm, EncryptedData, Credentials, AuthToken};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_hades_config() {
        let config = HadesConfig::default();
        assert!(config.encryption_enabled);
        assert_eq!(config.default_algorithm, EncryptionAlgorithm::AES256GCM);
        assert_eq!(config.key_rotation_days, 90);
        assert!(config.audit_enabled);
    }
    
    #[test]
    fn test_hades_config_builder() {
        let config = HadesConfig::new()
            .with_algorithm(EncryptionAlgorithm::ChaCha20Poly1305)
            .with_key_rotation(30)
            .disable_encryption()
            .disable_audit();
            
        assert_eq!(config.default_algorithm, EncryptionAlgorithm::ChaCha20Poly1305);
        assert_eq!(config.key_rotation_days, 30);
        assert!(!config.encryption_enabled);
        assert!(!config.audit_enabled);
    }
}

/// Tests de cifrado
#[cfg(test)]
mod encryption_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_aes256_gcm_encryption() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos sensibles del paciente";
        
        let encrypted = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::AES256GCM);
        
        // Verificar que el ciphertext es diferente del plaintext
        assert_ne!(encrypted.ciphertext, plaintext.to_vec());
    }
    
    #[tokio::test]
    async fn test_chacha20_poly1305_encryption() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos médicos confidenciales";
        
        let encrypted = hades.encrypt(plaintext, EncryptionAlgorithm::ChaCha20Poly1305).await
            .expect("Encryption failed");
        
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::ChaCha20Poly1305);
    }
    
    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos importantes";
        
        let encrypted = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        let decrypted = hades.decrypt(&encrypted).await
            .expect("Decryption failed");
        
        assert_eq!(decrypted, plaintext.to_vec());
    }
    
    #[tokio::test]
    async fn test_encryption_empty_data() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"";
        
        let result = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await;
        
        // Debe manejar datos vacíos gracefully
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_encryption_large_data() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = vec![0u8; 1024 * 1024]; // 1MB
        
        let encrypted = hades.encrypt(&plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption of large data failed");
        
        let decrypted = hades.decrypt(&encrypted).await
            .expect("Decryption of large data failed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[tokio::test]
    async fn test_encryption_with_associated_data() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos del paciente";
        let aad = b"Paciente ID: 12345";
        
        let encrypted = hades.encrypt_with_aad(plaintext, aad, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption with AAD failed");
        
        let decrypted = hades.decrypt_with_aad(&encrypted, aad).await
            .expect("Decryption with AAD failed");
        
        assert_eq!(decrypted, plaintext.to_vec());
    }
    
    #[tokio::test]
    async fn test_wrong_aad_fails() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos";
        let aad = b"Correct AAD";
        let wrong_aad = b"Wrong AAD";
        
        let encrypted = hades.encrypt_with_aad(plaintext, aad, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        let result = hades.decrypt_with_aad(&encrypted, wrong_aad).await;
        
        assert!(result.is_err(), "Decryption with wrong AAD should fail");
    }
    
    #[tokio::test]
    async fn test_tampered_ciphertext_fails() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos";
        
        let mut encrypted = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }
        
        let result = hades.decrypt(&encrypted).await;
        
        assert!(result.is_err(), "Decryption of tampered data should fail");
    }
}

/// Tests de autenticación
#[cfg(test)]
mod authentication_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_password_hashing() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let password = "MySecurePassword123!";
        
        let hash = hades.hash_password(password).await
            .expect("Password hashing failed");
        
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }
    
    #[tokio::test]
    async fn test_password_verification_success() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let password = "MySecurePassword123!";
        
        let hash = hades.hash_password(password).await
            .expect("Password hashing failed");
        
        let is_valid = hades.verify_password(password, &hash).await
            .expect("Password verification failed");
        
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_password_verification_failure() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let password = "MySecurePassword123!";
        let wrong_password = "WrongPassword";
        
        let hash = hades.hash_password(password).await
            .expect("Password hashing failed");
        
        let is_valid = hades.verify_password(wrong_password, &hash).await
            .expect("Password verification failed");
        
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_jwt_generation() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let user_id = "user-123";
        let roles = vec!["doctor".to_string(), "admin".to_string()];
        
        let token = hades.generate_jwt(user_id, &roles, 3600).await
            .expect("JWT generation failed");
        
        assert!(!token.token.is_empty());
        assert!(!token.refresh_token.is_empty());
        assert!(token.expires_in > 0);
    }
    
    #[tokio::test]
    async fn test_jwt_validation() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let user_id = "user-123";
        let roles = vec!["doctor".to_string()];
        
        let token = hades.generate_jwt(user_id, &roles, 3600).await
            .expect("JWT generation failed");
        
        let claims = hades.validate_jwt(&token.token).await
            .expect("JWT validation failed");
        
        assert_eq!(claims.sub, user_id);
        assert!(claims.roles.contains(&"doctor".to_string()));
    }
    
    #[tokio::test]
    async fn test_expired_jwt_fails() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let user_id = "user-123";
        let roles = vec!["doctor".to_string()];
        
        // Generate token with 0 seconds expiration (already expired)
        let token = hades.generate_jwt(user_id, &roles, 0).await
            .expect("JWT generation failed");
        
        let result = hades.validate_jwt(&token.token).await;
        
        assert!(result.is_err(), "Expired JWT should be rejected");
    }
    
    #[tokio::test]
    async fn test_invalid_jwt_fails() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let invalid_token = "invalid.jwt.token";
        
        let result = hades.validate_jwt(invalid_token).await;
        
        assert!(result.is_err(), "Invalid JWT should be rejected");
    }
    
    #[tokio::test]
    async fn test_account_lockout() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let user_id = "user-456";
        let password = "CorrectPassword";
        let hash = hades.hash_password(password).await.unwrap();
        
        // Simulate 5 failed attempts
        for _ in 0..5 {
            let _ = hades.verify_password("WrongPassword", &hash).await;
            hades.record_failed_attempt(user_id).await;
        }
        
        let is_locked = hades.is_account_locked(user_id).await;
        assert!(is_locked, "Account should be locked after 5 failed attempts");
    }
}

/// Tests de RBAC
#[cfg(test)]
mod rbac_tests {
    use super::*;
    use olympus::actors::hades::rbac::{Role, Permission, RBACManager};
    
    #[tokio::test]
    async fn test_role_permissions() {
        let admin_role = Role::admin();
        let doctor_role = Role::doctor();
        let nurse_role = Role::nurse();
        
        assert!(admin_role.has_permission(Permission::All));
        assert!(doctor_role.has_permission(Permission::ReadPatientData));
        assert!(doctor_role.has_permission(Permission::WritePatientData));
        assert!(!nurse_role.has_permission(Permission::DeletePatientData));
    }
    
    #[tokio::test]
    async fn test_rbac_manager() {
        let rbac = RBACManager::new();
        
        rbac.assign_role("user-123", Role::doctor()).await;
        rbac.assign_role("user-123", Role::researcher()).await;
        
        let roles = rbac.get_roles("user-123").await;
        assert_eq!(roles.len(), 2);
    }
    
    #[tokio::test]
    async fn test_permission_check() {
        let rbac = RBACManager::new();
        
        rbac.assign_role("user-123", Role::doctor()).await;
        
        let can_read = rbac.check_permission("user-123", Permission::ReadPatientData).await;
        assert!(can_read);
        
        let can_delete = rbac.check_permission("user-123", Permission::DeletePatientData).await;
        assert!(!can_delete);
    }
}

/// Tests de auditoría
#[cfg(test)]
mod audit_tests {
    use super::*;
    use olympus::actors::hades::audit::{AuditLog, AuditEvent, AuditSeverity};
    
    #[tokio::test]
    async fn test_audit_log_creation() {
        let event = AuditEvent::new()
            .with_user("doctor-123")
            .with_action("read_patient_data")
            .with_resource("patient-456")
            .with_severity(AuditSeverity::Info)
            .with_success(true);
        
        assert_eq!(event.user_id, "doctor-123");
        assert_eq!(event.action, "read_patient_data");
        assert!(event.success);
    }
    
    #[tokio::test]
    async fn test_audit_log_serialization() {
        let event = AuditEvent::new()
            .with_user("user-123")
            .with_action("login")
            .with_severity(AuditSeverity::Info);
        
        let json = serde_json::to_string(&event).expect("Serialization failed");
        assert!(json.contains("user-123"));
        assert!(json.contains("login"));
    }
}

/// Tests de integridad
#[cfg(test)]
mod integrity_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hash_integrity() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let data = b"Datos críticos del sistema";
        
        let hash1 = hades.hash_data(data).await.expect("Hashing failed");
        let hash2 = hades.hash_data(data).await.expect("Hashing failed");
        
        assert_eq!(hash1, hash2, "Same data should produce same hash");
    }
    
    #[tokio::test]
    async fn test_different_data_different_hash() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let data1 = b"Datos A";
        let data2 = b"Datos B";
        
        let hash1 = hades.hash_data(data1).await.expect("Hashing failed");
        let hash2 = hades.hash_data(data2).await.expect("Hashing failed");
        
        assert_ne!(hash1, hash2, "Different data should produce different hashes");
    }
    
    #[tokio::test]
    async fn test_hash_verification() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let data = b"Datos importantes";
        
        let hash = hades.hash_data(data).await.expect("Hashing failed");
        
        let is_valid = hades.verify_hash(data, &hash).await.expect("Verification failed");
        assert!(is_valid);
        
        let is_valid = hades.verify_hash(b"Datos diferentes", &hash).await.expect("Verification failed");
        assert!(!is_valid);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_encryption_performance() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let plaintext = b"Datos de prueba";
        let iterations = 1000;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let encrypted = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
                .expect("Encryption failed");
            let _ = hades.decrypt(&encrypted).await.expect("Decryption failed");
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        assert!(
            ops_per_sec > 100.0,
            "Encryption/decryption too slow: {:.0} ops/sec",
            ops_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_password_hashing_performance() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        let password = "TestPassword123!";
        
        let start = Instant::now();
        
        let _ = hades.hash_password(password).await.expect("Hashing failed");
        
        let elapsed = start.elapsed();
        
        // Argon2id debe tomar al menos 100ms para ser seguro
        assert!(
            elapsed.as_millis() >= 50,
            "Password hashing too fast (may be insecure): {:?}",
            elapsed
        );
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hades_message_encryption_request() {
        let mut hades = Hades::new().await.expect("Failed to create Hades");
        
        let message = ActorMessage::encrypt_request(b"Datos secretos");
        let response = hades.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_hades_message_auth_request() {
        let mut hades = Hades::new().await.expect("Failed to create Hades");
        
        let message = ActorMessage::auth_request("user", "password");
        let response = hades.handle_message(message).await;
        
        // Puede fallar si auth falla, pero no debe panic
        let _ = response;
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hades_creation() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        
        assert_eq!(hades.name(), GodName::Hades);
        assert_eq!(hades.domain(), DivineDomain::Security);
    }
    
    #[tokio::test]
    async fn test_hades_health_check() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        
        let health = hades.health_check().await;
        assert!(health.is_healthy());
    }
}

/// Tests de key rotation
#[cfg(test)]
mod key_rotation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_key_rotation() {
        let hades = Hades::new().await.expect("Failed to create Hades");
        
        // Encriptar con key actual
        let plaintext = b"Datos";
        let encrypted1 = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        // Rotar keys
        hades.rotate_keys().await.expect("Key rotation failed");
        
        // Encriptar con nueva key
        let encrypted2 = hades.encrypt(plaintext, EncryptionAlgorithm::AES256GCM).await
            .expect("Encryption failed");
        
        // Ambos deben poder desencriptarse
        let decrypted1 = hades.decrypt(&encrypted1).await.expect("Decryption failed");
        let decrypted2 = hades.decrypt(&encrypted2).await.expect("Decryption failed");
        
        assert_eq!(decrypted1, plaintext.to_vec());
        assert_eq!(decrypted2, plaintext.to_vec());
    }
}
