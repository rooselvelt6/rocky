// server/src/actors/minor_gods.rs
// Dioses menores del Olimpo - Implementaciones básicas

use super::{ActorMessage, GodName, OlympianActor};
use async_trait::async_trait;
use ractor::{Actor, ActorRef, ActorProcessingErr};

macro_rules! define_minor_god {
    ($name:ident, $domain:expr) => {
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl Actor for $name {
            type Msg = ActorMessage;
            type State = ();
            type Arguments = ();

            async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
                tracing::info!(concat!("✨ ", stringify!($name), " v16: Desplegado en el dominio ", $domain));
                Ok(())
            }

            async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, _state: &mut Self::State) -> Result<(), ActorProcessingErr> {
                tracing::debug!(concat!("✨ ", stringify!($name), ": Procesando mensaje de {:?}"), msg.from);
                Ok(())
            }
        }

        impl OlympianActor for $name {
            fn name(&self) -> GodName {
                GodName::$name
            }

            async fn initialize(&mut self) -> Result<(), String> {
                Ok(())
            }

            async fn shutdown(&mut self) -> Result<(), String> {
                tracing::info!(concat!("✨ ", stringify!($name), ": Detenido."));
                Ok(())
            }
        }
    };
}

define_minor_god!(Apollo, "Events");
define_minor_god!(Artemis, "Search");
define_minor_god!(Hera, "Validation");
define_minor_god!(Ares, "ConflictResolution");
define_minor_god!(Hefesto, "Configuration");
define_minor_god!(Chronos, "Scheduling");
define_minor_god!(Moirai, "Predictions");
define_minor_god!(Chaos, "Testing");
define_minor_god!(Aurora, "NewBeginnings");
define_minor_god!(Iris, "Communications");
define_minor_god!(Demeter, "Resources");
define_minor_god!(Dionysus, "Analysis");

