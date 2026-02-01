use crate::olympus::{GodCommand, IrisSignal};
use tokio::sync::{mpsc, broadcast};

pub struct Erinyes {
    iris_rx: broadcast::Receiver<IrisSignal>,
}

impl Erinyes {
    pub fn new(iris_rx: broadcast::Receiver<IrisSignal>) -> Self {
        Self { iris_rx }
    }

    pub async fn run(&mut self) {
        while let Ok(signal) = self.iris_rx.recv().await {
            match signal {
                IrisSignal::Panic(msg) => {
                    println!("🦇 Erinyes: ¡Pánico detectado! -> {}", msg);
                    // Lógica de restauración
                }
                _ => {}
            }
        }
    }
}
