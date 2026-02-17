// server/src/actors/minor_gods.rs
// Dioses menores del Olimpo - Implementaciones básicas

use super::{ActorMessage, GodHealth, GodName, MessagePayload, OlympianActor};
use async_trait::async_trait;
use chrono::Utc;

macro_rules! define_minor_god {
    ($name:ident, $domain:expr, $action:expr) => {
        pub struct $name {
            messages_count: u64,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    messages_count: 0,
                }
            }
        }

        #[async_trait]
        impl OlympianActor for $name {
            fn name(&self) -> GodName {
                GodName::$name
            }

            async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
                self.messages_count += 1;

                // Dioses menores solo procesan mensajes simples
                tracing::debug!(concat!("✨ ", stringify!($name), ": Procesando mensaje de {:?}"), msg.from);

                match &msg.payload {
                    MessagePayload::Heartbeat { .. } => {
                        // Responder con heartbeat
                        Some(ActorMessage::new(
                            GodName::$name,
                            msg.from,
                            MessagePayload::Heartbeat { timestamp: Utc::now() }
                        ))
                    }
                    _ => None
                }
            }

            async fn health(&self) -> GodHealth {
                GodHealth {
                    name: GodName::$name,
                    healthy: true,
                    last_heartbeat: Utc::now(),
                    messages_processed: self.messages_count,
                    uptime_seconds: 0,
                    status: $action.to_string(),
                }
            }

            async fn initialize(&mut self) -> Result<(), String> {
                tracing::info!(concat!("✨ ", stringify!($name), ": {} - Iniciando..."), $domain);
                Ok(())
            }

            async fn shutdown(&mut self) -> Result<(), String> {
                tracing::info!(concat!("✨ ", stringify!($name), ": {} - Deteniendo..."), $domain);
                Ok(())
            }
        }
    };
}

define_minor_god!(Apollo, "Events", "Logging events");
define_minor_god!(Artemis, "Search", "Indexing");
define_minor_god!(Hera, "Validation", "Validating");
define_minor_god!(Ares, "ConflictResolution", "Resolving conflicts");
define_minor_god!(Hefesto, "Configuration", "Configuring");
define_minor_god!(Chronos, "Scheduling", "Scheduling tasks");
define_minor_god!(Moirai, "Predictions", "Predicting");
define_minor_god!(Chaos, "Testing", "Testing chaos");
define_minor_god!(Aurora, "NewBeginnings", "Renewing");
// Aphrodite tiene su propia implementación completa en aphrodite.rs
define_minor_god!(Iris, "Communications", "Communicating");
define_minor_god!(Demeter, "Resources", "Managing resources");
define_minor_god!(Dionysus, "Analysis", "Analyzing");
