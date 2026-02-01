use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;

pub struct Hera;

impl Hera {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_vital_signs(&self, temp: f32, map: f32, hr: f32) -> Result<(), String> {
        if temp < 20.0 || temp > 50.0 { return Err("Temperatura físicamente imposible".to_string()); }
        if map < 0.0 || map > 300.0 { return Err("Presión arterial fuera de límites biológicos".to_string()); }
        if hr < 0.0 || hr > 400.0 { return Err("Frecuencia cardíaca absurda".to_string()); }
        Ok(())
    }
}

#[async_trait]
impl GodActor for Hera {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("👑 Hera: Vigilando invariantes médicos...");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
