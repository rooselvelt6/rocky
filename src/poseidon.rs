use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoseidonEvent {
    PatientCreated(String), // ID del paciente
    PatientUpdated(String),
    PatientDeleted(String),
    AssessmentCreated { patient_id: String, scale: String },
}

#[derive(Clone)]
pub struct PoseidonHub {
    pub tx: broadcast::Sender<PoseidonEvent>,
}

impl PoseidonHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, event: PoseidonEvent) {
        let _ = self.tx.send(event);
    }
}
