use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;
use std::fs;
use std::path::Path;

pub struct Demeter;

impl Demeter {
    pub fn new() -> Self {
        Self
    }

    pub fn enforce_order(&self) {
        println!("🌾 Demeter: Asegurando el orden quirúrgico...");
        
        // 1. Crear directorios si no existen
        let dirs = ["reports/pdf", "reports/csv", "reports/txt", "reports/technical", "reports/audit"];
        for dir in dirs {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("🌾 Demeter: Error al crear santuario {}: {}", dir, e);
            }
        }

        // 2. Limpieza de raíz de archivos sueltos
        self.clean_root();
    }

    fn clean_root(&self) {
        let root = Path::new(".");
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let extension = path.extension().and_then(|e| e.to_str());
                    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

                    match extension {
                        Some("txt") | Some("log") => {
                            self.move_to_sanctuary(&path, "reports/txt", filename);
                        }
                        Some("csv") => {
                            self.move_to_sanctuary(&path, "reports/csv", filename);
                        }
                        Some("md") if filename != "README.md" && filename != "Cargo.toml" => {
                             // Documentación técnica suelta
                            self.move_to_sanctuary(&path, "reports/technical", filename);
                        }
                         _ => {}
                    }
                }
            }
        }
    }

    fn move_to_sanctuary(&self, from: &Path, to_dir: &str, filename: &str) {
        let dest = Path::new(to_dir).join(filename);
        if let Err(e) = fs::rename(from, &dest) {
            eprintln!("🌾 Demeter: No pude mover {} al santuario {}: {}", filename, to_dir, e);
        } else {
            println!("🌾 Demeter: Archivo {} movido al santuario {}.", filename, to_dir);
        }
    }
}

#[async_trait]
impl GodActor for Demeter {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.enforce_order();
        println!("🌾 Demeter: Jerarquía de archivos purificada y en orden total.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
