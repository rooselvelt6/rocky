use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Aphrodite;

impl Aphrodite {
    pub fn new() -> Self {
        Self
    }

    pub fn check_harmony(&self) {
        println!("🐚 Aphrodite: Verificando la armonía de la interfaz UCI...");
    }
}

#[async_trait]
impl GodActor for Aphrodite {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🐚 Aphrodite: La belleza y claridad dominan el sistema.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
