pub struct Chaos;

impl Chaos {
    pub fn new() -> Self {
        Self
    }

    pub fn normalize(&self, input: &str) -> String {
        // Lógica de limpieza extraordinaria
        input.trim().to_lowercase()
    }
}
