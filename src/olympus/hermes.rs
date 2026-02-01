use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Hermes;

impl Hermes {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_external(&self, target: &str, data: &str) -> Result<(), String> {
        println!("🕊️  Hermes: Enviando datos a {}: {}", target, data);
        Ok(())
    }
}

#[async_trait]
impl GodActor for Hermes {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🕊️  Hermes: Alas desplegadas; red activa.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
