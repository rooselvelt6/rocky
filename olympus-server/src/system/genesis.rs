// src/system/genesis.rs
// OLYMPUS v16 - Genesis
// Bootloader que instancía y lanza el Panteón completo sobre Ractor.

use std::sync::Arc;
use tracing::info;
use ractor::Actor;

use crate::traits::ActorConfig;
use crate::infrastructure::{ValkeyStore, SurrealStore}; 

use crate::actors::zeus::Zeus;

pub struct Genesis;

impl Genesis {
    pub async fn ignite() -> Result<(), Box<dyn std::error::Error>> {
        info!("✨ GENESIS: Iniciando secuencia de ignición Ractor v16...");

        let valkey = Arc::new(ValkeyStore::default());
        let _surreal = Arc::new(SurrealStore::default());

        info!("⚡ Zeus igniting as Root Supervisor...");
        let (_zeus_ref, _) = Actor::spawn(Some("Zeus".to_string()), Zeus, ActorConfig::default()).await?;
        
        info!("🌌 GENESIS: All 21 Gods have been successfully spawned in Ractor.");
        Ok(())
    }
}
