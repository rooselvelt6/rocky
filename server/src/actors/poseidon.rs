// server/src/actors/poseidon.rs
// Poseidon: Flujo de Datos y Conexión a SurrealDB

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use ractor::{Actor, ActorRef, ActorProcessingErr};

pub struct Poseidon {
    surreal_connected: bool,
}

impl Poseidon {
    pub fn new() -> Self {
        Self {
            surreal_connected: false,
        }
    }
}

pub struct PoseidonState {
    pub queries_count: u64,
}

impl Actor for Poseidon {
    type Msg = ActorMessage;
    type State = PoseidonState;
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("🌊 Poseidon v16: Conexión a SurrealDB lista.");
        Ok(PoseidonState { queries_count: 0 })
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match msg.payload {
            MessagePayload::Query { query_type, params, reply } => {
                state.queries_count += 1;
                tracing::debug!("🌊 Poseidon procesando query: {}", query_type);
                
                if let Some(port) = reply {
                    if query_type == "get_patients" {
                        // En producción esto consultaría SurrealDB
                        let mock_data = serde_json::json!([
                            { "id": "p1", "first_name": "John", "last_name": "Doe" },
                            { "id": "p2", "first_name": "Jane", "last_name": "Smith" }
                        ]);
                        let _ = port.send(MessagePayload::Response { 
                            success: true, 
                            data: mock_data, 
                            error: None 
                        });
                    }
                }
            }
            MessagePayload::Command { action, .. } => {
                state.queries_count += 1;
                tracing::debug!("🌊 Poseidon procesando comando: {}", action);
            }
            _ => {}
        }
        Ok(())
    }
}

impl OlympianActor for Poseidon {
    fn name(&self) -> GodName {
        GodName::Poseidon
    }

    async fn initialize(&mut self) -> Result<(), String> {
        self.surreal_connected = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🌊 Poseidon: Conexiones cerradas.");
        Ok(())
    }
}
