// server/src/actors/erinyes.rs
// Erinyes: Monitoreo, Heartbeats y Alertas

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use chrono::Utc;
use std::collections::HashMap;
use ractor::{Actor, ActorRef, ActorProcessingErr};

pub struct Erinyes {
    heartbeats: HashMap<GodName, i64>,
}

impl Erinyes {
    pub fn new() -> Self {
        let mut heartbeats = HashMap::new();
        for god in [
            GodName::Zeus, GodName::Hades, GodName::Poseidon,
            GodName::Athena, GodName::Hermes, GodName::Hestia,
            GodName::Apollo, GodName::Artemis, GodName::Hera,
            GodName::Ares, GodName::Hefesto, GodName::Chronos,
            GodName::Moirai, GodName::Chaos, GodName::Aurora,
            GodName::Aphrodite, GodName::Iris, GodName::Demeter,
            GodName::Dionysus, GodName::Erinyes,
        ] {
            heartbeats.insert(god, Utc::now().timestamp());
        }

        Self { heartbeats }
    }
}

pub struct ErinyesState {
    pub active_alerts: u64,
}

impl Actor for Erinyes {
    type Msg = ActorMessage;
    type State = ErinyesState;
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("👁️ Erinyes v16: Monitoreo infra-red de Ractor iniciado.");
        Ok(ErinyesState { active_alerts: 0 })
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        if let MessagePayload::Heartbeat { .. } = msg.payload {
            tracing::debug!("💓 Erinyes: Heartbeat v16 de {:?}", msg.from);
        }
        Ok(())
    }
}

impl OlympianActor for Erinyes {
    fn name(&self) -> GodName {
        GodName::Erinyes
    }

    async fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("👁️ Erinyes: Monitoreo finalizado.");
        Ok(())
    }
}
