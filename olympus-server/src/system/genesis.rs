// src/system/genesis.rs
// OLYMPUS v16 - Genesis
// Bootloader que instancía y lanza el Panteón completo sobre Ractor.

use std::sync::Arc;
use tracing::{info, error};
use ractor::Actor;

use crate::actors::GodName;
use crate::traits::ActorConfig;
use crate::infrastructure::{ValkeyStore, SurrealStore}; 

// Importación de Dioses
use crate::actors::zeus::Zeus;
use crate::actors::hades::Hades;
use crate::actors::poseidon::Poseidon;
use crate::actors::erinyes::Erinyes;
use crate::actors::hermes::Hermes;
use crate::actors::hera::Hera;
use crate::actors::artemis::Artemis;
use crate::actors::apollo::Apollo;
use crate::actors::athena::Athena;
use crate::actors::ares::Ares;
use crate::actors::aphrodite::Aphrodite;
use crate::actors::hefesto::Hefesto;
use crate::actors::dionysus::Dionysus;
use crate::actors::demeter::Demeter;
use crate::actors::hestia::Hestia;
use crate::actors::chronos::Chronos;
use crate::actors::iris::Iris;
use crate::actors::moirai::Moirai;
use crate::actors::chaos::Chaos;
use crate::actors::aurora::Aurora;
use crate::actors::nemesis::Nemesis;

pub struct Genesis;

impl Genesis {
    pub async fn ignite() -> Result<(), Box<dyn std::error::Error>> {
        info!("✨ GENESIS: Iniciando secuencia de ignición Ractor v16...");

        // 1. Infraestructura Base
        let valkey = Arc::new(ValkeyStore::default());
        let surreal = Arc::new(SurrealStore::default());

        // 2. Spawn de Actores
        // NOTA: En Ractor, spawn retorna (ActorRef, JoinHandle)
        
        // --- ZEUS (Gobernador Supremo) ---
        let (zeus_ref, _) = Actor::spawn(Some("Zeus".to_string()), Zeus, ActorConfig::default()).await?;
        info!("⚡ Zeus igniting as Root Supervisor...");

        // --- TRINIDAD Y SEGURIDAD (Hijos de Zeus) ---
        let (_hades_ref, _) = Actor::spawn_linked(Some("Hades".to_string()), Hades, (), zeus_ref.clone().into()).await?;
        let (_poseidon_ref, _) = Actor::spawn_linked(Some("Poseidon".to_string()), Poseidon, valkey.clone(), zeus_ref.clone().into()).await?;
        let (_erinyes_ref, _) = Actor::spawn_linked(Some("Erinyes".to_string()), Erinyes, valkey.clone(), zeus_ref.clone().into()).await?;
        
        // --- CLINICAL AND MESSAGING ---
        let (_hermes_ref, _) = Actor::spawn_linked(Some("Hermes".to_string()), Hermes, (), zeus_ref.clone().into()).await?;
        let (_athena_ref, _) = Actor::spawn_linked(Some("Athena".to_string()), Athena, (), zeus_ref.clone().into()).await?;
        let (_apollo_ref, _) = Actor::spawn_linked(Some("Apollo".to_string()), Apollo, (), zeus_ref.clone().into()).await?;
        let (_artemis_ref, _) = Actor::spawn_linked(Some("Artemis".to_string()), Artemis, (), zeus_ref.clone().into()).await?;
        
        // --- GOVERNMENT AND RULES ---
        let (_hera_ref, _) = Actor::spawn_linked(Some("Hera".to_string()), Hera, (), zeus_ref.clone().into()).await?;
        let (_ares_ref, _) = Actor::spawn_linked(Some("Ares".to_string()), Ares, (), zeus_ref.clone().into()).await?;
        let (_hefesto_ref, _) = Actor::spawn_linked(Some("Hefesto".to_string()), Hefesto, (), zeus_ref.clone().into()).await?;
        
        // --- SPECIALIZED ---
        let (_chronos_ref, _) = Actor::spawn_linked(Some("Chronos".to_string()), Chronos, (), zeus_ref.clone().into()).await?;
        let (_moirai_ref, _) = Actor::spawn_linked(Some("Moirai".to_string()), Moirai, (), zeus_ref.clone().into()).await?;
        let (_chaos_ref, _) = Actor::spawn_linked(Some("Chaos".to_string()), Chaos, (), zeus_ref.clone().into()).await?;
        let (_aurora_ref, _) = Actor::spawn_linked(Some("Aurora".to_string()), Aurora, (), zeus_ref.clone().into()).await?;
        let (_aphrodite_ref, _) = Actor::spawn_linked(Some("Aphrodite".to_string()), Aphrodite, (), zeus_ref.clone().into()).await?;
        let (_iris_ref, _) = Actor::spawn_linked(Some("Iris".to_string()), Iris, (), zeus_ref.clone().into()).await?;
        
        // --- ANALYSIS AND PERSISTENCE ---
        let (_demeter_ref, _) = Actor::spawn_linked(Some("Demeter".to_string()), Demeter, ActorConfig::default(), zeus_ref.clone().into()).await?;
        let (_dionysus_ref, _) = Actor::spawn_linked(Some("Dionysus".to_string()), Dionysus, ActorConfig::default(), zeus_ref.clone().into()).await?;
        let (_hestia_ref, _) = Actor::spawn_linked(Some("Hestia".to_string()), Hestia, (valkey.clone(), surreal.clone()), zeus_ref.clone().into()).await?;
        let (_nemesis_ref, _) = Actor::spawn_linked(Some("Nemesis".to_string()), Nemesis, (), zeus_ref.clone().into()).await?;


        info!("🌌 GENESIS: All 21 Gods have been successfully spawned in Ractor.");
        
        Ok(())
    }
}
