// tests/integration/mod.rs
// Tests de integración para OLYMPUS v15

pub mod actor_interaction;
pub mod supervision;
pub mod messaging;
pub mod persistence;
pub mod security_flow;
pub mod websocket;

use olympus::system::genesis::Genesis;
use olympus::actors::{GodName, DivineDomain};

/// Setup común para tests de integración
pub async fn setup_test_olympus() -> Genesis {
    Genesis::new_test_instance().await
        .expect("Failed to create test Olympus instance")
}

/// Teardown de tests de integración
pub async fn teardown_test_olympus(genesis: Genesis) {
    genesis.shutdown().await
        .expect("Failed to shutdown Olympus");
}

/// Helpers para tests de integración
pub mod helpers {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    pub const INTEGRATION_TIMEOUT: Duration = Duration::from_secs(30);
    
    pub async fn wait_for_actor_ready(genesis: &Genesis, god: GodName) -> bool {
        timeout(Duration::from_secs(5), async {
            loop {
                if genesis.is_actor_ready(god).await {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }).await.unwrap_or(false)
    }
    
    pub async fn wait_for_all_actors(genesis: &Genesis) -> bool {
        let gods = vec![
            GodName::Zeus, GodName::Hades, GodName::Hestia,
            GodName::Hermes, GodName::Erinyes, GodName::Athena,
            GodName::Apollo, GodName::Artemis, GodName::Poseidon,
        ];
        
        for god in gods {
            if !wait_for_actor_ready(genesis, god).await {
                return false;
            }
        }
        true
    }
}
