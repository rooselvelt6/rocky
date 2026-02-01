use crate::olympus::IrisSignal;
use tokio::sync::broadcast;

pub struct Iris {
    tx: broadcast::Sender<IrisSignal>,
}

impl Iris {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn sender(&self) -> broadcast::Sender<IrisSignal> {
        self.tx.clone()
    }
}
