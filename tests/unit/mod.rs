// tests/unit/mod.rs
// Test suite unitaria para OLYMPUS v15

pub mod zeus;
pub mod hades;
pub mod hestia;
pub mod hermes;
pub mod erinyes;
pub mod athena;
pub mod apollo;
pub mod poseidon;
pub mod ares;
pub mod artemis;
pub mod aurora;
pub mod chaos;
pub mod chronos;
pub mod demeter;
pub mod dionysus;
pub mod hefesto;
pub mod hera;
pub mod iris;
pub mod moirai;
pub mod nemesis;

use olympus::actors::{GodName, DivineDomain};
use std::sync::Once;

static INIT: Once = Once::new();

/// Inicialización global de tests
pub fn setup() {
    INIT.call_once(|| {
        // Inicializar tracing para tests
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    });
}

/// Helpers comunes para tests
pub mod helpers {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    /// Timeout por defecto para operaciones async
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
    
    /// Helper para ejecutar con timeout
    pub async fn with_timeout<T>(
        duration: Duration,
        future: impl std::future::Future<Output = T>
    ) -> T {
        timeout(duration, future)
            .await
            .expect("Operation timed out")
    }
    
    /// Genera datos de prueba únicos
    pub fn generate_test_id() -> String {
        format!("test-{}", uuid::Uuid::new_v4())
    }
}

#[cfg(test)]
mod common_tests {
    use super::*;
    
    #[test]
    fn test_god_name_variants() {
        let gods = vec![
            GodName::Zeus,
            GodName::Hades,
            GodName::Poseidon,
            GodName::Hermes,
            GodName::Erinyes,
            GodName::Hestia,
            GodName::Athena,
            GodName::Apollo,
            GodName::Artemis,
            GodName::Chronos,
            GodName::Ares,
            GodName::Hefesto,
            GodName::Iris,
            GodName::Moirai,
            GodName::Demeter,
            GodName::Chaos,
            GodName::Hera,
            GodName::Nemesis,
            GodName::Aurora,
            GodName::Aphrodite,
        ];
        
        assert_eq!(gods.len(), 20);
        
        for god in gods {
            let domain = god.domain();
            assert!(!format!("{:?}", domain).is_empty());
        }
    }
    
    #[test]
    fn test_divine_domain_coverage() {
        let domains = vec![
            DivineDomain::Governance,
            DivineDomain::Security,
            DivineDomain::Connectivity,
            DivineDomain::Messaging,
            DivineDomain::Monitoring,
            DivineDomain::Persistence,
            DivineDomain::Intelligence,
            DivineDomain::Events,
            DivineDomain::Search,
            DivineDomain::Scheduling,
            DivineDomain::ConflictResolution,
            DivineDomain::Construction,
            DivineDomain::Communication,
            DivineDomain::Lifecycle,
            DivineDomain::Resources,
            DivineDomain::Chaos,
            DivineDomain::Validation,
            DivineDomain::Compliance,
            DivineDomain::Maintenance,
            DivineDomain::UI,
        ];
        
        assert_eq!(domains.len(), 20);
    }
}
