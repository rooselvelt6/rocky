use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct Moirai {
    handles: HashMap<String, JoinHandle<()>>,
}

impl Moirai {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, handle: JoinHandle<()>) {
        self.handles.insert(name.to_string(), handle);
    }
}
