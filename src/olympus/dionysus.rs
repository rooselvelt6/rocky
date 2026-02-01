use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Dionysus;

impl Dionysus {
    pub fn new() -> Self {
        Self
    }

    pub fn swap_module(&self, module: &str) {
        println!("🍇 Dionysus: Intercambiando módulo {} en caliente...", module);
    }
}

#[async_trait]
impl GodActor for Dionysus {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🍇 Dionysus: Modularidad dinámica lista para el éxtasis.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
