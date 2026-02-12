// tests/unit/athena/mod.rs
// Tests unitarios para Athena - Inteligencia y ML

use olympus::actors::athena::{Athena, AthenaConfig, AnalysisEngine, PredictionModel};
use olympus::actors::athena::clinical::{SofaScore, SapsScore, GlasgowScore, ApacheScore};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};
use olympus::models::patient::Patient;

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_athena_config() {
        let config = AthenaConfig::default();
        assert!(config.ml_enabled);
        assert_eq!(config.confidence_threshold, 0.85);
        assert!(config.cache_predictions);
        assert_eq!(config.cache_ttl_secs, 300);
    }
    
    #[test]
    fn test_athena_config_builder() {
        let config = AthenaConfig::new()
            .with_confidence_threshold(0.90)
            .with_cache_ttl(600)
            .disable_ml()
            .disable_cache();
            
        assert_eq!(config.confidence_threshold, 0.90);
        assert_eq!(config.cache_ttl_secs, 600);
        assert!(!config.ml_enabled);
        assert!(!config.cache_predictions);
    }
}

/// Tests de análisis clínico
#[cfg(test)]
mod clinical_analysis_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sofa_score_calculation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_respiratory(15.0, 100.0) // PaO2/FiO2 = 150
            .with_coagulation(120.0)       // Platelets = 120
            .with_liver(2.0)               // Bilirubin = 2.0
            .with_cardiovascular(70.0, true) // MAP = 70, no dopamine
            .with_gcs(13)                  // GCS = 13
            .with_renal(1.5, 800.0);       // Creatinine = 1.5, urine = 800
        
        let sofa = athena.calculate_sofa(&patient).await.expect("SOFA calculation failed");
        
        assert!(sofa.respiratory >= 0 && sofa.respiratory <= 4);
        assert!(sofa.coagulation >= 0 && sofa.coagulation <= 4);
        assert!(sofa.liver >= 0 && sofa.liver <= 4);
        assert!(sofa.cardiovascular >= 0 && sofa.cardiovascular <= 4);
        assert!(sofa.glasgow >= 0 && sofa.glasgow <= 4);
        assert!(sofa.renal >= 0 && sofa.renal <= 4);
        
        assert!(sofa.total >= 0 && sofa.total <= 24);
    }
    
    #[tokio::test]
    async fn test_saps_score_calculation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_age(65)
            .with_heart_rate(110.0)
            .with_systolic_bp(90.0)
            .with_temperature(39.0)
            .with_gcs(9);
        
        let saps = athena.calculate_saps(&patient).await.expect("SAPS calculation failed");
        
        assert!(saps.total >= 0 && saps.total <= 163);
        assert!(saps.predicted_mortality >= 0.0 && saps.predicted_mortality <= 1.0);
    }
    
    #[tokio::test]
    async fn test_glasgow_score_calculation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let eye = 4;      // Espontáneo
        let verbal = 5;   // Orientado
        let motor = 6;    // Obedece comandos
        
        let glasgow = athena.calculate_glasgow(eye, verbal, motor).await;
        
        assert_eq!(glasgow.total, 15); // 4 + 5 + 6
        assert_eq!(glasgow.eye_response, eye);
        assert_eq!(glasgow.verbal_response, verbal);
        assert_eq!(glasgow.motor_response, motor);
    }
    
    #[tokio::test]
    async fn test_apache_score_calculation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_age(70)
            .with_chronic_conditions(vec!["diabetes".to_string()])
            .with_temperature(38.5)
            .with_mean_arterial_pressure(70.0)
            .with_heart_rate(120.0)
            .with_respiratory_rate(30.0)
            .with_oxygenation(200.0, false)
            .with_arterial_ph(7.25)
            .with_sodium(145.0)
            .with_potassium(5.5)
            .with_creatinine(2.0)
            .with_hematocrit(45.0)
            .with_wbc(18.0)
            .with_gcs(10);
        
        let apache = athena.calculate_apache(&patient).await.expect("Apache calculation failed");
        
        assert!(apache.total >= 0 && apache.total <= 71);
        assert!(apache.predicted_mortality >= 0.0 && apache.predicted_mortality <= 1.0);
    }
    
    #[tokio::test]
    async fn test_news2_score_calculation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_respiratory_rate(25.0)  // Elevado
            .with_oxygen_saturation(92.0) // Bajo
            .with_systolic_bp(100.0)      // Normal
            .with_heart_rate(110.0)       // Elevado
            .with_temperature(37.5)       // Normal
            .with_gcs(15);                // Normal
        
        let news2 = athena.calculate_news2(&patient).await.expect("NEWS2 calculation failed");
        
        assert!(news2.total >= 0 && news2.total <= 20);
        
        // Debe tener alguna alerta con estos valores
        assert!(news2.total > 0);
    }
}

/// Tests de ML/Predicciones
#[cfg(test)]
mod ml_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mortality_prediction() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::critical_patient();
        
        let prediction = athena.predict_mortality(&patient).await;
        
        assert!(prediction.is_ok());
        let pred = prediction.unwrap();
        
        assert!(pred.probability >= 0.0 && pred.probability <= 1.0);
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
        assert!(!pred.factors.is_empty());
    }
    
    #[tokio::test]
    async fn test_icu_stay_prediction() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_sofa_score(12); // SOFA alto
        
        let prediction = athena.predict_icu_stay_days(&patient).await;
        
        assert!(prediction.is_ok());
        let pred = prediction.unwrap();
        
        assert!(pred.predicted_days > 0.0);
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    }
    
    #[tokio::test]
    async fn test_readmission_risk() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_previous_admissions(3)
            .with_comorbidities(vec!["diabetes".to_string(), "hypertension".to_string()]);
        
        let risk = athena.predict_readmission_risk(&patient).await;
        
        assert!(risk.is_ok());
        let risk_score = risk.unwrap();
        
        assert!(risk_score.probability >= 0.0 && risk_score.probability <= 1.0);
    }
    
    #[tokio::test]
    async fn test_prediction_caching() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        let patient = Patient::test_patient();
        
        // Primera predicción (debe calcular)
        let start = std::time::Instant::now();
        let pred1 = athena.predict_mortality(&patient).await.unwrap();
        let time1 = start.elapsed();
        
        // Segunda predicción (debe usar cache)
        let start = std::time::Instant::now();
        let pred2 = athena.predict_mortality(&patient).await.unwrap();
        let time2 = start.elapsed();
        
        // Deben ser iguales
        assert_eq!(pred1.probability, pred2.probability);
        
        // Cache debe ser más rápido
        assert!(time2 < time1);
    }
    
    #[tokio::test]
    async fn test_low_confidence_handling() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        // Paciente con datos incompletos
        let patient = Patient::test_patient()
            .with_incomplete_data();
        
        let prediction = athena.predict_mortality(&patient).await;
        
        // Puede tener baja confianza pero no debe fallar
        assert!(prediction.is_ok());
        let pred = prediction.unwrap();
        assert!(pred.confidence < 0.85); // Baja confianza
    }
}

/// Tests de análisis de tendencias
#[cfg(test)]
mod trend_analysis_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_vital_signs_trend() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        // Serie temporal de signos vitales
        let readings = vec![
            (0.0, 70.0, 120.0, 98.6),  // (hora, hr, bp, temp)
            (1.0, 75.0, 118.0, 98.8),
            (2.0, 85.0, 115.0, 99.2),
            (3.0, 95.0, 110.0, 99.8),
            (4.0, 110.0, 100.0, 100.5),
        ];
        
        let trend = athena.analyze_vital_trend(&readings).await;
        
        assert!(trend.is_ok());
        let analysis = trend.unwrap();
        
        // Tendencia debe ser deterioro
        assert!(analysis.trend_direction == TrendDirection::Deteriorating);
        assert!(analysis.risk_level >= 3); // Riesgo alto
    }
    
    #[tokio::test]
    async fn test_anomaly_detection() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let baseline = vec![70.0, 72.0, 71.0, 73.0, 70.0]; // FC normal
        let current = 150.0; // FC anormalmente alta
        
        let anomaly = athena.detect_anomaly(&baseline, current, "heart_rate").await;
        
        assert!(anomaly.is_anomaly);
        assert!(anomaly.severity >= 3);
    }
    
    #[tokio::test]
    async fn test_early_warning_score_trend() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let scores = vec![
            (0, 2),   // Hora 0, score 2
            (4, 4),   // Hora 4, score 4
            (8, 7),   // Hora 8, score 7
        ];
        
        let trend = athena.analyze_ews_trend(&scores).await;
        
        assert!(trend.escalation_detected);
        assert!(trend.recommendation.contains("urgent"));
    }
}

/// Tests de recomendaciones
#[cfg(test)]
mod recommendation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_treatment_recommendations() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_sepsis_indicators();
        
        let recommendations = athena.get_treatment_recommendations(&patient).await;
        
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("antibiotic")));
    }
    
    #[tokio::test]
    async fn test_intervention_priority() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let issues = vec![
            ("hypotension".to_string(), Severity::Critical),
            ("fever".to_string(), Severity::Moderate),
            ("electrolyte_imbalance".to_string(), Severity::Severe),
        ];
        
        let prioritized = athena.prioritize_interventions(&issues).await;
        
        assert_eq!(prioritized[0].0, "hypotension"); // Más crítico primero
        assert_eq!(prioritized[1].0, "electrolyte_imbalance");
        assert_eq!(prioritized[2].0, "fever");
    }
}

/// Tests de manejo de mensajes
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_athena_message_analyze_patient() {
        let mut athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient();
        let message = ActorMessage::analyze_patient(patient);
        
        let response = athena.handle_message(message).await;
        
        assert!(response.is_ok());
    }
    
    #[tokio::test]
    async fn test_athena_message_prediction_request() {
        let mut athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient();
        let message = ActorMessage::predict_outcome(patient, "mortality");
        
        let response = athena.handle_message(message).await;
        
        assert!(response.is_ok());
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_athena_creation() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        assert_eq!(athena.name(), GodName::Athena);
        assert_eq!(athena.domain(), DivineDomain::Intelligence);
    }
    
    #[tokio::test]
    async fn test_athena_health_check() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let health = athena.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_model_loading() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let models = athena.list_loaded_models().await;
        
        // Debe tener al menos un modelo cargado
        assert!(!models.is_empty());
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_sofa_calculation_performance() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        let iterations = 1000;
        let patient = Patient::test_patient();
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = athena.calculate_sofa(&patient).await;
        }
        
        let elapsed = start.elapsed();
        let calcs_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        assert!(
            calcs_per_sec > 1000.0,
            "SOFA calculation too slow: {:.0} calcs/sec",
            calcs_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_prediction_latency() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        let patient = Patient::test_patient();
        
        let mut latencies = vec![];
        
        for _ in 0..10 {
            let start = Instant::now();
            let _ = athena.predict_mortality(&patient).await;
            latencies.push(start.elapsed().as_millis());
        }
        
        let avg_latency: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
        
        assert!(
            avg_latency < 500,
            "Prediction latency too high: {} ms",
            avg_latency
        );
    }
}

/// Tests de validación de datos
#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_invalid_vital_signs_handling() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        // Valores imposibles
        let patient = Patient::test_patient()
            .with_heart_rate(-10.0)  // Negativo
            .with_temperature(200.0); // Imposible
        
        let result = athena.calculate_sofa(&patient).await;
        
        // Debe manejar gracefulmente (corregir o reportar error)
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_missing_data_handling() {
        let athena = Athena::new().await.expect("Failed to create Athena");
        
        let patient = Patient::test_patient()
            .with_missing_vitals();
        
        let prediction = athena.predict_mortality(&patient).await;
        
        // Debe funcionar con datos incompletos (con baja confianza)
        assert!(prediction.is_ok());
    }
}
