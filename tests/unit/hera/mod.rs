// tests/unit/hera/mod.rs
// Tests unitarios para Hera - Validación de Datos

use olympus::actors::hera::{Hera, HeraConfig, ValidationEngine, SchemaRegistry};
use olympus::actors::hera::validation::{ValidationRule, ValidationResult, DataType, Constraint};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_hera_config() {
        let config = HeraConfig::default();
        assert!(config.strict_mode);
        assert!(config.sanitization_enabled);
        assert_eq!(config.max_validation_time_ms, 1000);
        assert!(config.cache_schemas);
    }
    
    #[test]
    fn test_hera_config_builder() {
        let config = HeraConfig::new()
            .with_max_validation_time(500)
            .disable_strict_mode()
            .disable_sanitization()
            .disable_schema_cache();
            
        assert_eq!(config.max_validation_time_ms, 500);
        assert!(!config.strict_mode);
        assert!(!config.sanitization_enabled);
        assert!(!config.cache_schemas);
    }
}

/// Tests de validación de tipos
#[cfg(test)]
mod type_validation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validate_string_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let result = hera.validate_type("hello", DataType::String).await;
        assert!(result.is_valid());
        
        let result = hera.validate_type(123, DataType::String).await;
        assert!(!result.is_valid());
    }
    
    #[tokio::test]
    async fn test_validate_integer_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let result = hera.validate_type(42, DataType::Integer).await;
        assert!(result.is_valid());
        
        let result = hera.validate_type("not a number", DataType::Integer).await;
        assert!(!result.is_valid());
        
        let result = hera.validate_type(3.14, DataType::Integer).await;
        assert!(!result.is_valid()); // Float no es Integer
    }
    
    #[tokio::test]
    async fn test_validate_boolean_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        assert!(hera.validate_type(true, DataType::Boolean).await.is_valid());
        assert!(hera.validate_type(false, DataType::Boolean).await.is_valid());
        assert!(!hera.validate_type("true", DataType::Boolean).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_validate_date_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let valid_dates = vec![
            "2024-01-15",
            "2024-01-15T10:30:00Z",
            "15/01/2024",
        ];
        
        for date in valid_dates {
            assert!(
                hera.validate_type(date, DataType::Date).await.is_valid(),
                "Date '{}' should be valid",
                date
            );
        }
        
        assert!(!hera.validate_type("not a date", DataType::Date).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_validate_email_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let valid_emails = vec![
            "user@example.com",
            "test.user@domain.co.uk",
            "user+tag@example.org",
        ];
        
        for email in valid_emails {
            assert!(
                hera.validate_type(email, DataType::Email).await.is_valid(),
                "Email '{}' should be valid",
                email
            );
        }
        
        let invalid_emails = vec![
            "not-an-email",
            "@example.com",
            "user@",
            "user@@example.com",
        ];
        
        for email in invalid_emails {
            assert!(
                !hera.validate_type(email, DataType::Email).await.is_valid(),
                "Email '{}' should be invalid",
                email
            );
        }
    }
    
    #[tokio::test]
    async fn test_validate_uuid_type() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(hera.validate_type(valid_uuid, DataType::UUID).await.is_valid());
        
        let invalid_uuid = "not-a-uuid";
        assert!(!hera.validate_type(invalid_uuid, DataType::UUID).await.is_valid());
    }
}

/// Tests de restricciones (constraints)
#[cfg(test)]
mod constraint_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_required_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let constraint = Constraint::required();
        
        assert!(hera.validate_constraint("value", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("", &constraint).await.is_valid());
        assert!(!hera.validate_constraint(None::<String>, &constraint).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_min_max_length_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let constraint = Constraint::length_range(3, 10);
        
        assert!(hera.validate_constraint("hello", &constraint).await.is_valid());
        assert!(hera.validate_constraint("hi", &constraint).await.is_valid()); // 2 chars, pero min es 3
        assert!(!hera.validate_constraint("hi", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("this is way too long", &constraint).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_min_max_value_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let constraint = Constraint::value_range(0, 100);
        
        assert!(hera.validate_constraint(50, &constraint).await.is_valid());
        assert!(hera.validate_constraint(0, &constraint).await.is_valid());
        assert!(hera.validate_constraint(100, &constraint).await.is_valid());
        assert!(!hera.validate_constraint(-1, &constraint).await.is_valid());
        assert!(!hera.validate_constraint(101, &constraint).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_pattern_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let constraint = Constraint::pattern(r"^[A-Z]{2}\d{4}$"); // Dos letras mayúsculas + 4 dígitos
        
        assert!(hera.validate_constraint("AB1234", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("ab1234", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("ABC123", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("AB123", &constraint).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_enum_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let allowed = vec!["active", "inactive", "pending"];
        let constraint = Constraint::enum_values(allowed);
        
        assert!(hera.validate_constraint("active", &constraint).await.is_valid());
        assert!(hera.validate_constraint("inactive", &constraint).await.is_valid());
        assert!(!hera.validate_constraint("deleted", &constraint).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_unique_constraint() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let constraint = Constraint::unique();
        
        // Primera vez es válido
        assert!(hera.validate_unique("user1", "users", "username").await.is_valid());
        
        // Segunda vez con mismo valor no es válido
        assert!(!hera.validate_unique("user1", "users", "username").await.is_valid());
        
        // Valor diferente es válido
        assert!(hera.validate_unique("user2", "users", "username").await.is_valid());
    }
}

/// Tests de validación de schemas
#[cfg(test)]
mod schema_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_and_validate_schema() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("patient")
            .with_field("id", DataType::UUID, vec![Constraint::required()])
            .with_field("name", DataType::String, vec![
                Constraint::required(),
                Constraint::length_range(1, 100)
            ])
            .with_field("age", DataType::Integer, vec![
                Constraint::value_range(0, 150)
            ])
            .with_field("email", DataType::Email, vec![]);
        
        hera.register_schema(schema).await.unwrap();
        
        // Validar datos válidos
        let valid_data = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "John Doe",
            "age": 45,
            "email": "john@example.com"
        });
        
        let result = hera.validate_against_schema(&valid_data, "patient").await;
        assert!(result.is_valid());
        
        // Validar datos inválidos
        let invalid_data = serde_json::json!({
            "id": "not-a-uuid",
            "name": "",
            "age": 200,
            "email": "invalid-email"
        });
        
        let result = hera.validate_against_schema(&invalid_data, "patient").await;
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 4); // 4 errores
    }
    
    #[tokio::test]
    async fn test_nested_schema_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let address_schema = Schema::new("address")
            .with_field("street", DataType::String, vec![Constraint::required()])
            .with_field("city", DataType::String, vec![Constraint::required()])
            .with_field("zipcode", DataType::String, vec![Constraint::pattern(r"^\d{5}$")]);
        
        let person_schema = Schema::new("person")
            .with_field("name", DataType::String, vec![Constraint::required()])
            .with_nested_field("address", address_schema);
        
        hera.register_schema(person_schema).await.unwrap();
        
        let valid_data = serde_json::json!({
            "name": "Jane Doe",
            "address": {
                "street": "123 Main St",
                "city": "New York",
                "zipcode": "10001"
            }
        });
        
        let result = hera.validate_against_schema(&valid_data, "person").await;
        assert!(result.is_valid());
        
        // Datos anidados inválidos
        let invalid_data = serde_json::json!({
            "name": "Jane Doe",
            "address": {
                "street": "123 Main St",
                "city": "",
                "zipcode": "invalid"
            }
        });
        
        let result = hera.validate_against_schema(&invalid_data, "person").await;
        assert!(!result.is_valid());
    }
    
    #[tokio::test]
    async fn test_schema_versioning() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        // Schema v1
        let schema_v1 = Schema::new("api_request")
            .with_version(1)
            .with_field("user_id", DataType::String, vec![Constraint::required()]);
        
        hera.register_schema(schema_v1).await.unwrap();
        
        // Schema v2 (agrega campo opcional)
        let schema_v2 = Schema::new("api_request")
            .with_version(2)
            .with_field("user_id", DataType::String, vec![Constraint::required()])
            .with_field("timestamp", DataType::Date, vec![]);
        
        hera.register_schema(schema_v2).await.unwrap();
        
        // Validar con v1
        let data_v1 = serde_json::json!({"user_id": "user123"});
        let result = hera.validate_with_version(&data_v1, "api_request", 1).await;
        assert!(result.is_valid());
        
        // Validar con v2
        let data_v2 = serde_json::json!({
            "user_id": "user123",
            "timestamp": "2024-01-15T10:30:00Z"
        });
        let result = hera.validate_with_version(&data_v2, "api_request", 2).await;
        assert!(result.is_valid());
    }
}

/// Tests de sanitización
#[cfg(test)]
mod sanitization_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_xss_sanitization() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let malicious = "<script>alert('xss')</script><b>Safe</b>";
        let sanitized = hera.sanitize_xss(malicious).await;
        
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("<b>")); // Tags seguros permitidos
    }
    
    #[tokio::test]
    async fn test_sql_injection_sanitization() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let malicious = "'; DROP TABLE users; --";
        let sanitized = hera.sanitize_sql(malicious).await;
        
        assert!(!sanitized.contains("DROP"));
        assert!(!sanitized.contains(";"));
    }
    
    #[tokio::test]
    async fn test_trim_whitespace() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let dirty = "  hello world  \n\t";
        let clean = hera.sanitize_whitespace(dirty).await;
        
        assert_eq!(clean, "hello world");
    }
    
    #[tokio::test]
    async fn test_normalize_unicode() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        // UTF-8 con diferentes formas de codificación
        let unicode = "café"; // é puede ser e + ́ o é
        let normalized = hera.sanitize_unicode(unicode).await;
        
        // Debe normalizar a forma NFC
        assert!(normalized.chars().all(|c| c.len_utf8() <= 3));
    }
}

/// Tests de validación de negocio
#[cfg(test)]
mod business_rule_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_date_range_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let start_date = "2024-01-01";
        let end_date = "2024-01-31";
        
        // Rango válido
        assert!(hera.validate_date_range(start_date, end_date).await.is_valid());
        
        // Rango inválido (start > end)
        assert!(!hera.validate_date_range(end_date, start_date).await.is_valid());
        
        // Fecha en el futuro no válida para eventos pasados
        let future = "2025-12-31";
        assert!(!hera.validate_past_date(future).await.is_valid());
    }
    
    #[tokio::test]
    async fn test_cross_field_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let data = serde_json::json!({
            "password": "MyPass123!",
            "password_confirm": "MyPass123!"
        });
        
        // Campos coinciden
        assert!(hera.validate_fields_match(&data, "password", "password_confirm").await.is_valid());
        
        let data_mismatch = serde_json::json!({
            "password": "MyPass123!",
            "password_confirm": "DifferentPass!"
        });
        
        // Campos no coinciden
        assert!(!hera.validate_fields_match(&data_mismatch, "password", "password_confirm").await.is_valid());
    }
    
    #[tokio::test]
    async fn test_conditional_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        // Si country == "USA", state es requerido
        let usa_data = serde_json::json!({
            "country": "USA",
            "state": "CA"
        });
        
        assert!(hera.validate_conditional(&usa_data, "country", "USA", "state").await.is_valid());
        
        let usa_no_state = serde_json::json!({
            "country": "USA"
        });
        
        assert!(!hera.validate_conditional(&usa_no_state, "country", "USA", "state").await.is_valid());
        
        // Si country != "USA", state es opcional
        let uk_data = serde_json::json!({
            "country": "UK"
        });
        
        assert!(hera.validate_conditional(&uk_data, "country", "USA", "state").await.is_valid());
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hera_message_validate_data() {
        let mut hera = Hera::new().await.expect("Failed to create Hera");
        
        // Registrar schema primero
        let schema = Schema::new("test_data")
            .with_field("name", DataType::String, vec![Constraint::required()]);
        hera.register_schema(schema).await.unwrap();
        
        let data = serde_json::json!({"name": "Test"});
        let message = ActorMessage::validate_request(data, "test_data");
        
        let response = hera.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(response.unwrap().is_valid());
    }
    
    #[tokio::test]
    async fn test_hera_message_register_schema() {
        let mut hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("new_schema")
            .with_field("id", DataType::UUID, vec![]);
        
        let message = ActorMessage::register_schema(schema);
        let response = hera.handle_message(message).await;
        
        assert!(response.is_ok());
        assert!(hera.has_schema("new_schema").await);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_validation_throughput() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("perf_test")
            .with_field("name", DataType::String, vec![Constraint::required()])
            .with_field("age", DataType::Integer, vec![Constraint::value_range(0, 150)]);
        
        hera.register_schema(schema).await.unwrap();
        
        let data = serde_json::json!({
            "name": "Test User",
            "age": 30
        });
        
        let count = 10000;
        let start = Instant::now();
        
        for _ in 0..count {
            let _ = hera.validate_against_schema(&data, "perf_test").await;
        }
        
        let elapsed = start.elapsed();
        let validations_per_sec = count as f64 / elapsed.as_secs_f64();
        
        assert!(
            validations_per_sec > 10000.0,
            "Validation throughput too low: {:.0} validations/sec",
            validations_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_schema_cache_performance() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("cached_schema");
        hera.register_schema(schema).await.unwrap();
        
        // Primera consulta (sin cache)
        let start = Instant::now();
        hera.get_schema("cached_schema").await.unwrap();
        let first_time = start.elapsed();
        
        // Segunda consulta (con cache)
        let start = Instant::now();
        hera.get_schema("cached_schema").await.unwrap();
        let second_time = start.elapsed();
        
        // Cache debe ser al menos 10x más rápido
        assert!(
            second_time.as_nanos() * 10 < first_time.as_nanos(),
            "Schema cache not effective"
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_data_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("empty_test")
            .with_field("optional_field", DataType::String, vec![]);
        
        hera.register_schema(schema).await.unwrap();
        
        let empty = serde_json::json!({});
        let result = hera.validate_against_schema(&empty, "empty_test").await;
        
        // Debe ser válido (todos los campos son opcionales)
        assert!(result.is_valid());
    }
    
    #[tokio::test]
    async fn test_deeply_nested_validation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        // Crear estructura muy anidada (10 niveles)
        let mut data = serde_json::json!({"value": 1});
        for _ in 0..10 {
            data = serde_json::json!({"nested": data});
        }
        
        // Debe manejar sin stack overflow
        let result = hera.validate_json_structure(&data).await;
        assert!(result.is_valid());
    }
    
    #[tokio::test]
    async fn test_very_large_payload() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let large_array: Vec<i32> = (0..100000).collect();
        let data = serde_json::json!({"numbers": large_array});
        
        let start = Instant::now();
        let result = hera.validate_json_structure(&data).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_valid());
        assert!(
            elapsed.as_millis() < 1000,
            "Large payload validation too slow: {:?}",
            elapsed
        );
    }
    
    #[tokio::test]
    async fn test_unicode_edge_cases() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let unicode_strings = vec![
            "🎉",                                    // Emoji
            "中文测试",                              // Chino
            "العربية",                              // Árabe
            "München",                              // Alemán
            "🚀🌟💻",                                // Múltiples emojis
            "\u{200B}",                             // Zero-width space
            "<script>alert('test')</script>",      // Intentos de XSS
        ];
        
        for s in unicode_strings {
            let sanitized = hera.sanitize_xss(s).await;
            assert!(!sanitized.contains("<script>"));
        }
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hera_creation() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        assert_eq!(hera.name(), GodName::Hera);
        assert_eq!(hera.domain(), DivineDomain::Validation);
    }
    
    #[tokio::test]
    async fn test_hera_health_check() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let health = hera.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_schema_persistence() {
        let hera = Hera::new().await.expect("Failed to create Hera");
        
        let schema = Schema::new("persistent_schema")
            .with_field("id", DataType::UUID, vec![Constraint::required()]);
        
        hera.register_schema(schema).await.unwrap();
        
        // Verificar que el schema está registrado
        assert!(hera.has_schema("persistent_schema").await);
        
        // Obtener schema
        let retrieved = hera.get_schema("persistent_schema").await;
        assert!(retrieved.is_some());
    }
}
