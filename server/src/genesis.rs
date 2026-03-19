use crate::actors::*;
use std::collections::HashMap;
use ractor::{Actor, ActorRef};

pub struct OlympusGenesis;

impl OlympusGenesis {
    pub async fn ignite() -> Result<HashMap<GodName, ActorRef<ActorMessage>>, Box<dyn std::error::Error>> {
        tracing::info!("✨ GENESIS v16: Iniciando secuencia de ignición Ractor...");

        let mut actors = HashMap::new();

        // Macro para simplificar el spawn de dioses en v16
        macro_rules! spawn_god {
            ($name:ident, $actor:expr) => {{
                let (actor_ref, _handle) = Actor::spawn(None, $actor, ()).await?;
                actors.insert(GodName::$name, actor_ref);
                tracing::info!("⚡ {} desplegado (Ractor)", stringify!($name));
            }};
        }

        // === TRINIDAD PRINCIPAL ===
        spawn_god!(Zeus, Zeus::new());
        spawn_god!(Hades, Hades::new());
        spawn_god!(Poseidon, Poseidon::new());

        // === DIOSES CLAVE ===
        spawn_god!(Athena, Athena::new());
        spawn_god!(Hermes, Hermes::new());
        spawn_god!(Hestia, Hestia::new());
        spawn_god!(Erinyes, Erinyes::new());
        spawn_god!(Aphrodite, Aphrodite::new());

        // === DIOSES MENORES ===
        spawn_god!(Apollo, Apollo::new());
        spawn_god!(Artemis, Artemis::new());
        spawn_god!(Hera, Hera::new());
        spawn_god!(Ares, Ares::new());
        spawn_god!(Hefesto, Hefesto::new());
        spawn_god!(Chronos, Chronos::new());
        spawn_god!(Moirai, Moirai::new());
        spawn_god!(Chaos, Chaos::new());
        spawn_god!(Aurora, Aurora::new());
        spawn_god!(Iris, Iris::new());
        spawn_god!(Demeter, Demeter::new());
        spawn_god!(Dionysus, Dionysus::new());

        tracing::info!("🌌 GENESIS v16: {} Dioses activos en el tejido Ractor.", actors.len());
        
        Ok(actors)
    }
}
