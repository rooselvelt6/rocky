use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonEvent {
    PatientCreated(String),
    PatientUpdated(String),
    PatientDeleted(String),
    AssessmentCreated { patient_id: String, scale: String },
    SystemAlert(String),
}

pub struct Poseidon {
    pub tx: broadcast::Sender<PoseidonEvent>,
}

impl Poseidon {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, event: PoseidonEvent) {
        let _ = self.tx.send(event);
    }
    
    pub async fn connect_db() -> Result<surrealdb::Surreal<surrealdb::engine::any::Any>, Box<dyn std::error::Error>> {
        // Lógica movida de db.rs
        let db = surrealdb::engine::any::connect("file:uci.db").await?;
        db.use_ns("uci").use_db("main").await?;
        println!("🔱 Poseidon: Conexión a las profundidades (DB) establecida.");
        Ok(db)
    }
}

#[async_trait]
impl GodActor for Poseidon {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔱 Poseidon: Gestor de eventos y persistencia activo.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
