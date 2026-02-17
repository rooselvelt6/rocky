// server/src/genesis.rs
// Genesis: Bootloader del Olimpo - Inicia los 20 Dioses

use crate::actors::*;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct OlympusGenesis;

impl OlympusGenesis {
    pub async fn ignite() -> Result<HashMap<GodName, mpsc::Sender<ActorMessage>>, Box<dyn std::error::Error>> {
        tracing::info!("✨ GENESIS: Iniciando secuencia de ignición del Olimpo v15...");

        let mut senders: HashMap<GodName, mpsc::Sender<ActorMessage>> = HashMap::new();

        // === TRINIDAD PRINCIPAL ===
        
        // 1. Zeus (Gobernador) - primero
        let (zeus_tx, zeus_rx) = mpsc::channel(1000);
        let zeus = Zeus::new();
        let zeus_runtime = ActorRuntime::new(Box::new(zeus), zeus_rx);
        tokio::spawn(zeus_runtime.run());
        senders.insert(GodName::Zeus, zeus_tx);
        tracing::info!("⚡ Zeus desplegado");

        // 2. Hades (Seguridad)
        let (hades_tx, hades_rx) = mpsc::channel(1000);
        let hades = Hades::new();
        let hades_runtime = ActorRuntime::new(Box::new(hades), hades_rx);
        tokio::spawn(hades_runtime.run());
        senders.insert(GodName::Hades, hades_tx);
        tracing::info!("🔒 Hades desplegado");

        // 3. Poseidon (Datos)
        let (poseidon_tx, poseidon_rx) = mpsc::channel(1000);
        let poseidon = Poseidon::new();
        let poseidon_runtime = ActorRuntime::new(Box::new(poseidon), poseidon_rx);
        tokio::spawn(poseidon_runtime.run());
        senders.insert(GodName::Poseidon, poseidon_tx);
        tracing::info!("🌊 Poseidon desplegado");

        // === DIOSES CLAVE ===

        // 4. Athena (Escalas/ML)
        let (athena_tx, athena_rx) = mpsc::channel(1000);
        let athena = Athena::new();
        let athena_runtime = ActorRuntime::new(Box::new(athena), athena_rx);
        tokio::spawn(athena_runtime.run());
        senders.insert(GodName::Athena, athena_tx);
        tracing::info!("🧠 Athena desplegada");

        // 5. Hermes (Mensajería)
        let (hermes_tx, hermes_rx) = mpsc::channel(1000);
        let hermes = Hermes::new();
        let hermes_runtime = ActorRuntime::new(Box::new(hermes), hermes_rx);
        tokio::spawn(hermes_runtime.run());
        senders.insert(GodName::Hermes, hermes_tx);
        tracing::info!("📨 Hermes desplegado");

        // 6. Hestia (Persistencia)
        let (hestia_tx, hestia_rx) = mpsc::channel(1000);
        let hestia = Hestia::new();
        let hestia_runtime = ActorRuntime::new(Box::new(hestia), hestia_rx);
        tokio::spawn(hestia_runtime.run());
        senders.insert(GodName::Hestia, hestia_tx);
        tracing::info!("🏛️ Hestia desplegada");

        // 7. Erinyes (Monitoreo)
        let (erinyes_tx, erinyes_rx) = mpsc::channel(1000);
        let erinyes = Erinyes::new();
        let erinyes_runtime = ActorRuntime::new(Box::new(erinyes), erinyes_rx);
        tokio::spawn(erinyes_runtime.run());
        senders.insert(GodName::Erinyes, erinyes_tx);
        tracing::info!("👁️ Erinyes desplegado");

        // 8. Aphrodite (UI/UX) - Diosa de la Belleza
        let (aphrodite_tx, aphrodite_rx) = mpsc::channel(1000);
        let aphrodite = Aphrodite::new();
        let aphrodite_runtime = ActorRuntime::new(Box::new(aphrodite), aphrodite_rx);
        tokio::spawn(aphrodite_runtime.run());
        senders.insert(GodName::Aphrodite, aphrodite_tx);
        tracing::info!("🎨 Aphrodite desplegada - Gestionando UI/Temas");

        // === DIOSES MENORES (12) ===

        let minor_gods: Vec<(GodName, Box<dyn OlympianActor>)> = vec![
            (GodName::Apollo, Box::new(Apollo::new())),
            (GodName::Artemis, Box::new(Artemis::new())),
            (GodName::Hera, Box::new(Hera::new())),
            (GodName::Ares, Box::new(Ares::new())),
            (GodName::Hefesto, Box::new(Hefesto::new())),
            (GodName::Chronos, Box::new(Chronos::new())),
            (GodName::Moirai, Box::new(Moirai::new())),
            (GodName::Chaos, Box::new(Chaos::new())),
            (GodName::Aurora, Box::new(Aurora::new())),
            (GodName::Iris, Box::new(Iris::new())),
            (GodName::Demeter, Box::new(Demeter::new())),
            (GodName::Dionysus, Box::new(Dionysus::new())),
        ];

        for (name, actor) in minor_gods {
            let (tx, rx) = mpsc::channel(100);
            let runtime = ActorRuntime::new(actor, rx);
            tokio::spawn(runtime.run());
            senders.insert(name, tx);
            tracing::info!("✨ {} desplegado", name.as_str());
        }

        // Iniciar heartbeat loop
        let senders_clone = senders.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                
                // Enviar heartbeat a Erinyes
                if let Some(erinyes_tx) = senders_clone.get(&GodName::Erinyes) {
                    for (god, tx) in &senders_clone {
                        if *god != GodName::Erinyes {
                            let heartbeat = ActorMessage::new(
                                *god,
                                GodName::Erinyes,
                                MessagePayload::Heartbeat { timestamp: chrono::Utc::now() }
                            );
                            let _ = tx.send(heartbeat).await;
                        }
                    }
                }
            }
        });

        tracing::info!("🌌 GENESIS: {} Dioses desplegados. La Trinidad vigila.", senders.len());
        
        Ok(senders)
    }
}

// Función helper para obtener estado de salud de todos los dioses
pub async fn get_all_gods_health(
    senders: &HashMap<GodName, mpsc::Sender<ActorMessage>>
) -> Vec<GodHealth> {
    let mut health_data = Vec::new();
    
    for (god, tx) in senders {
        // Crear mensaje de consulta de salud
        let msg = ActorMessage::new(
            GodName::Zeus,
            *god,
            MessagePayload::Query { 
                query_type: "health_check".to_string(),
                params: serde_json::json!({}),
            }
        );
        
        // En una implementación completa, esperaríamos respuesta
        // Por ahora, devolvemos datos simulados basados en el estado
        let health = GodHealth {
            name: *god,
            healthy: true,
            last_heartbeat: chrono::Utc::now(),
            messages_processed: 0,
            uptime_seconds: 0,
            status: "Active".to_string(),
        };
        
        health_data.push(health);
    }
    
    health_data
}

// Función para enviar mensaje a un dios específico
pub async fn send_to_god(
    senders: &HashMap<GodName, mpsc::Sender<ActorMessage>>,
    god: GodName,
    msg: ActorMessage,
) -> Result<(), String> {
    if let Some(tx) = senders.get(&god) {
        tx.send(msg).await.map_err(|e| format!("Failed to send: {}", e))
    } else {
        Err(format!("God {:?} not found", god))
    }
}
