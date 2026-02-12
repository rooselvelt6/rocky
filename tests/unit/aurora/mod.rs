// tests/unit/aurora/mod.rs
// Tests unitarios para Aurora - Renovación y Mantenimiento

use olympus::actors::aurora::{Aurora, AuroraConfig, Dawn, Hope, Inspiration, Opportunities};
use olympus::actors::aurora::types::{RenewalType, HopeLevel, InspirationType, Opportunity};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

// Module: Dawn System
#[cfg(test)]
mod dawn_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dawn_renewal_cycle() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let renewal = aurora.dawn.create_renewal_cycle(RenewalType::System)
            .with_schedule("0 0 * * 0") // Weekly
            .with_priority(Priority::High);
        
        let result = aurora.schedule_renewal(renewal).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_system_renewal() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let result = aurora.dawn.execute_system_renewal().await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().optimized_components > 0);
    }
    
    #[tokio::test]
    async fn test_memory_renewal() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let before = aurora.get_memory_usage().await;
        
        aurora.dawn.execute_memory_renewal().await.unwrap();
        
        let after = aurora.get_memory_usage().await;
        
        // Debe liberar algo de memoria
        assert!(after <= before);
    }
    
    #[tokio::test]
    async fn test_cache_renewal() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Llenar cache con datos viejos
        aurora.fill_cache_with_old_data().await;
        
        let old_entries_before = aurora.get_cache_old_entries().await;
        
        aurora.dawn.execute_cache_renewal().await.unwrap();
        
        let old_entries_after = aurora.get_cache_old_entries().await;
        
        assert!(old_entries_after < old_entries_before);
    }
}

// Module: Hope Manager
#[cfg(test)]
mod hope_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hope_initialization() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let hope_level = aurora.hope.get_current_level().await;
        
        assert!(hope_level >= HopeLevel::Neutral);
        assert!(hope_level <= HopeLevel::Absolute);
    }
    
    #[tokio::test]
    async fn test_positive_event_boosts_hope() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let before = aurora.hope.get_current_level().await;
        
        aurora.hope.record_positive_event("System recovery successful").await;
        
        let after = aurora.hope.get_current_level().await;
        
        assert!(after > before || after == HopeLevel::Absolute);
    }
    
    #[tokio::test]
    async fn test_negative_event_reduces_hope() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Establecer nivel alto inicial
        aurora.hope.set_level(HopeLevel::High).await;
        
        let before = aurora.hope.get_current_level().await;
        
        aurora.hope.record_negative_event("Service degradation").await;
        
        let after = aurora.hope.get_current_level().await;
        
        assert!(after < before || after == HopeLevel::Despair);
    }
    
    #[tokio::test]
    async fn test_hope_resilience_tracking() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Múltiples eventos
        for i in 0..10 {
            if i % 2 == 0 {
                aurora.hope.record_positive_event("Success").await;
            } else {
                aurora.hope.record_negative_event("Failure").await;
            }
        }
        
        let stats = aurora.hope.get_resilience_stats().await;
        
        assert_eq!(stats.positive_events, 5);
        assert_eq!(stats.negative_events, 5);
        assert!(stats.recovery_rate >= 0.0 && stats.recovery_rate <= 1.0);
    }
}

// Module: Inspiration Engine
#[cfg(test)]
mod inspiration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_capture_inspiration() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let inspiration = aurora.inspiration.capture(
            InspirationType::Technical,
            "Refactor using async/await for better performance"
        ).await;
        
        assert!(inspiration.is_some());
        assert_eq!(inspiration.unwrap().inspiration_type, InspirationType::Technical);
    }
    
    #[tokio::test]
    async fn test_inspiration_evaluation() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let inspiration = aurora.inspiration.capture(
            InspirationType::Creative,
            "New dashboard design with real-time metrics"
        ).await.unwrap();
        
        let evaluation = aurora.inspiration.evaluate(&inspiration).await;
        
        assert!(evaluation.feasibility >= 0.0 && evaluation.feasibility <= 1.0);
        assert!(evaluation.impact >= 0.0 && evaluation.impact <= 1.0);
    }
    
    #[tokio::test]
    async fn test_inspiration_prioritization() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Capturar múltiples inspiraciones
        aurora.inspiration.capture(InspirationType::Technical, "Idea 1").await;
        aurora.inspiration.capture(InspirationType::Creative, "Idea 2").await;
        aurora.inspiration.capture(InspirationType::Practical, "Idea 3").await;
        
        let prioritized = aurora.inspiration.get_prioritized_inspirations().await;
        
        assert_eq!(prioritized.len(), 3);
        // Debe estar ordenado por prioridad
    }
    
    #[tokio::test]
    async fn test_inspiration_intensity_levels() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let spark = aurora.inspiration.capture_with_intensity(
            "Small improvement",
            InspirationIntensity::Spark
        ).await.unwrap();
        
        let revelation = aurora.inspiration.capture_with_intensity(
            "Major breakthrough",
            InspirationIntensity::Revelation
        ).await.unwrap();
        
        assert!(revelation.priority > spark.priority);
    }
}

// Module: Opportunity Detector
#[cfg(test)]
mod opportunities_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_detect_technical_opportunity() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Simular métricas que indican oportunidad
        aurora.simulate_slow_queries().await;
        
        let opportunities = aurora.opportunities.scan_for_technical().await;
        
        assert!(!opportunities.is_empty());
        assert!(opportunities.iter().any(|o| o.opportunity_type == OpportunityType::Technical));
    }
    
    #[tokio::test]
    async fn test_detect_business_opportunity() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        // Simular feedback de usuarios
        aurora.simulate_user_feedback("Need mobile app").await;
        
        let opportunities = aurora.opportunities.scan_for_business().await;
        
        assert!(!opportunities.is_empty());
    }
    
    #[tokio::test]
    async fn test_opportunity_evaluation() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let opportunity = Opportunity::new()
            .with_type(OpportunityType::Learning)
            .with_description("Adopt Rust for system components")
            .with_estimated_effort(Duration::from_days(30))
            .with_estimated_impact(Impact::High);
        
        let evaluation = aurora.opportunities.evaluate(&opportunity).await;
        
        assert!(evaluation.roi > 0.0);
        assert!(evaluation.risk_level >= RiskLevel::Low && evaluation.risk_level <= RiskLevel::Critical);
    }
    
    #[tokio::test]
    async fn test_opportunity_lifecycle() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let opp_id = aurora.opportunities.create(
            Opportunity::new()
                .with_type(OpportunityType::Personal)
                .with_description("Learn new technology")
        ).await.unwrap();
        
        // Cambiar estado
        aurora.opportunities.update_status(&opp_id, OpportunityStatus::InProgress).await.unwrap();
        
        let opp = aurora.opportunities.get(&opp_id).await.unwrap();
        assert_eq!(opp.status, OpportunityStatus::InProgress);
        
        // Completar
        aurora.opportunities.update_status(&opp_id, OpportunityStatus::Completed).await.unwrap();
        
        let completed = aurora.opportunities.get(&opp_id).await.unwrap();
        assert_eq!(completed.status, OpportunityStatus::Completed);
        assert!(completed.completed_at.is_some());
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_aurora_creation() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        assert_eq!(aurora.name(), GodName::Aurora);
        assert_eq!(aurora.domain(), DivineDomain::Maintenance);
    }
    
    #[tokio::test]
    async fn test_aurora_health_check() {
        let aurora = Aurora::new().await.expect("Failed to create Aurora");
        
        let health = aurora.health_check().await;
        assert!(health.is_healthy());
    }
}
