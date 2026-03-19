// server/src/actors/zeus.rs
// Zeus: Gobernador Supremo y Supervisor del Olimpo

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use std::collections::HashMap;
use ractor::{Actor, ActorRef, ActorProcessingErr};

pub struct Zeus {
    supervised_actors: HashMap<GodName, bool>,
    restart_count: HashMap<GodName, u32>,
}

impl Zeus {
    pub fn new() -> Self {
        let mut supervised = HashMap::new();
        for god in [
            GodName::Hades, GodName::Poseidon, GodName::Athena,
            GodName::Hermes, GodName::Hestia, GodName::Erinyes,
            GodName::Apollo, GodName::Artemis, GodName::Hera,
            GodName::Ares, GodName::Hefesto, GodName::Chronos,
            GodName::Moirai, GodName::Chaos, GodName::Aurora,
            GodName::Aphrodite, GodName::Iris, GodName::Demeter,
            GodName::Dionysus,
        ] {
            supervised.insert(god, true);
        }

        Self {
            supervised_actors: supervised,
            restart_count: HashMap::new(),
        }
    }

    async fn handle_supervision(&mut self, from: GodName, healthy: bool) {
        if let Some(status) = self.supervised_actors.get_mut(&from) {
            *status = healthy;
            if !healthy {
                let count = self.restart_count.entry(from).or_insert(0);
                *count += 1;
                tracing::warn!("⚡ Zeus: {:?} fallando (v16: ruteo Ractor activo)", from);
            }
        }
    }
}

impl Actor for Zeus {
    type Msg = ActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("⚡ Zeus v16: Iniciando trono de supervisión...");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, _state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match &msg.payload {
            MessagePayload::Heartbeat { .. } => {
                // En v16, Ractor puede manejar esto con monitoreo nativo, 
                // pero mantenemos compatibilidad de mensajes
                tracing::debug!("⚡ Zeus: Heartbeat de {:?}", msg.from);
            }
            MessagePayload::Query { query_type, .. } => {
                if query_type == "supervision_status" {
                    // Respuesta vía reply si fuera necesario, o simplemente log
                    tracing::info!("⚡ Zeus: Consulta de estado recibida");
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl OlympianActor for Zeus {
    fn name(&self) -> GodName {
        GodName::Zeus
    }

    async fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("⚡ Zeus: Trono cerrado.");
        Ok(())
    }
}
