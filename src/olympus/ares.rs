use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Ares;

impl Ares {
    pub fn new() -> Self {
        Self
    }

    pub fn capture_resources(&self) {
        println!("⚔️ Ares: Capturando recursos de CPU para tareas vitales.");
    }
}

#[async_trait]
impl GodActor for Ares {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚔️ Ares: El poder de la CPU está bajo mi mando.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
