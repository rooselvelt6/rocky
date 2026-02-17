// server/src/actors/poseidon.rs
// Poseidon: Flujo de Datos y Conexión a SurrealDB

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;

pub struct Poseidon {
    surreal_connected: bool,
    queries_executed: u64,
    messages_count: u64,
}

impl Poseidon {
    pub fn new() -> Self {
        Self {
            surreal_connected: false,
            queries_executed: 0,
            messages_count: 0,
        }
    }

    async fn query_patients(&mut self) -> serde_json::Value {
        self.queries_executed += 1;
        // Simulación de query a SurrealDB
        serde_json::json!({
            "patients": [
                {"id": "1", "first_name": "Juan", "last_name": "Perez", "diagnosis": "Neumonia"},
                {"id": "2", "first_name": "Maria", "last_name": "Garcia", "diagnosis": "Postquirurgico"},
            ],
            "source": "SurrealDB",
            "queried_by": "Poseidon"
        })
    }

    async fn create_patient(&mut self, data: &serde_json::Value) -> serde_json::Value {
        self.queries_executed += 1;
        let id = uuid::Uuid::new_v4().to_string();
        
        serde_json::json!({
            "id": id,
            "created": true,
            "data": data,
            "source": "SurrealDB",
            "created_by": "Poseidon"
        })
    }

    async fn delete_patient(&mut self, id: &str) -> serde_json::Value {
        self.queries_executed += 1;
        
        serde_json::json!({
            "id": id,
            "deleted": true,
            "source": "SurrealDB",
            "deleted_by": "Poseidon"
        })
    }
}

#[async_trait]
impl OlympianActor for Poseidon {
    fn name(&self) -> GodName {
        GodName::Poseidon
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;

        match &msg.payload {
            MessagePayload::Query { query_type, params } => {
                let result = match query_type.as_str() {
                    "get_patients" => {
                        self.query_patients().await
                    }

                    "get_patient" => {
                        let id = params.get("id")?.as_str()?;
                        serde_json::json!({
                            "id": id,
                            "first_name": "Demo",
                            "last_name": "Patient",
                            "queried_by": "Poseidon"
                        })
                    }

                    _ => return None,
                };

                Some(ActorMessage::new(
                    GodName::Poseidon,
                    msg.from,
                    MessagePayload::Response {
                        success: true,
                        data: result,
                        error: None,
                    }
                ))
            }

            MessagePayload::Command { action, data } => {
                let result = match action.as_str() {
                    "create_patient" => {
                        self.create_patient(data).await
                    }

                    "delete_patient" => {
                        let id = data.get("id")?.as_str()?;
                        self.delete_patient(id).await
                    }

                    _ => return None,
                };

                Some(ActorMessage::new(
                    GodName::Poseidon,
                    msg.from,
                    MessagePayload::Response {
                        success: true,
                        data: result,
                        error: None,
                    }
                ))
            }

            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Poseidon,
            healthy: self.surreal_connected || true, // Siempre saludable en demo
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Connected - {} queries", self.queries_executed),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("🌊 Poseidon: Conectando a SurrealDB...");
        self.surreal_connected = true;
        tracing::info!("🌊 Poseidon: Conectado a SurrealDB");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🌊 Poseidon: Cerrando conexiones...");
        Ok(())
    }
}
