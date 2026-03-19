// server/src/actors/hermes.rs
// Hermes: Mensajería y Routing

use async_trait::async_trait;
use super::{ActorMessage, GodName, OlympianActor};
use ractor::{Actor, ActorRef, ActorProcessingErr};

pub struct Hermes;

impl Hermes {
    pub fn new() -> Self {
        Self
    }
}

impl Actor for Hermes {
    type Msg = ActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("📨 Hermes v16: Router de alta frecuencia (Ractor) listo.");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, _state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        tracing::debug!("📨 Hermes enrutando: {:?} -> {:?}", msg.from, msg.to);
        Ok(())
    }
}

impl OlympianActor for Hermes {
    fn name(&self) -> GodName {
        GodName::Hermes
    }

    async fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("📨 Hermes: Enrutador apagado.");
        Ok(())
    }
}
