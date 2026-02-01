use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Hestia;

impl Hestia {
    pub fn new() -> Self {
        Self
    }

    pub fn get_secret(&self, key: &str) -> Option<String> {
        std::env::var(format!("OLYMPUS_{}", key)).ok()
    }
}

#[async_trait]
impl GodActor for Hestia {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔥 Hestia: Protegiendo el hogar y los secretos.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
