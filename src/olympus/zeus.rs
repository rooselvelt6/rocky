use crate::olympus::{GodActor, GodCommand, IrisSignal};
use tokio::sync::{mpsc, broadcast};
use async_trait::async_trait;

pub struct Zeus {
    iris_tx: broadcast::Sender<IrisSignal>,
    // Futuras deidades dependientes
}

impl Zeus {
    pub fn new(iris_tx: broadcast::Sender<IrisSignal>) -> Self {
        Self { iris_tx }
    }
}

#[async_trait]
impl GodActor for Zeus {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚡ Zeus: Iniciando Orquestación Soberana...");
        self.iris_tx.send(IrisSignal::SystemReady)?;
        Ok(())
    }

    async fn handle_command(&mut self, cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        match cmd {
            GodCommand::Start => self.start().await?,
            _ => println!("⚡ Zeus: Comando no implementado"),
        }
        Ok(())
    }
}
