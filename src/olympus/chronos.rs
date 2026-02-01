use crate::olympus::{GodActor, GodCommand, IrisSignal};
use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

pub struct Chronos {
    iris_tx: broadcast::Sender<IrisSignal>,
}

impl Chronos {
    pub fn new(iris_tx: broadcast::Sender<IrisSignal>) -> Self {
        Self { iris_tx }
    }

    pub async fn heartbeat_loop(&self) {
        let mut ticker = interval(Duration::from_millis(1000));
        loop {
            ticker.tick().await;
            let _ = self.iris_tx.send(IrisSignal::Heartbeat("Luz v10".to_string()));
        }
    }
}

#[async_trait]
impl GodActor for Chronos {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⌛ Chronos: Reloj maestro sincronizado.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
