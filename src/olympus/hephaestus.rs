use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Hephaestus;

impl Hephaestus {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize_build(&self) {
        println!("🔨 Hephaestus: Forjando binarios con optimización de nivel 10.");
    }
}

#[async_trait]
impl GodActor for Hephaestus {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔨 Hephaestus: Forja alquímica activada.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
