/// 🏛️ OLYMPUS v12 - EL PANTÉÓN COMPLETO
/// 20 dioses con dominios mitológicos específicos

// Importar todos los dioses
pub mod artemis;
pub mod apollo;
pub mod poseidon;
pub mod iris;
pub mod zeus;
pub mod hera;
pub mod athena;
pub mod hades;
pub mod ares;
pub mod aphrodite;
pub mod hermes;
pub mod chronos;
pub mod hefesto;
pub mod dionysius; // Implementación unificada de Dionisio
pub mod hestia;
pub mod erinyes;
pub mod moirai;
pub mod chaos;
pub mod aurora;

// Exportar interfaces comunes
pub mod traits;

// Re-exportar estructuras de datos
pub use crate::models::*;

// Configuración del sistema
#[derive(Debug)]
pub struct OlympusConfig {
    pub version: String,
    pub environment: String,
    pub max_patients: u64,
    pub assessment_intervals: HashMap<String, u32>,
    pub retention_days: u32,
    pub auto_backup_enabled: bool,
}

impl Default for OlympusConfig {
    fn default() -> Self {
        Self {
            version: "12.0.0".to_string(),
            environment: "development".to_string(),
            max_patients: 10000,
            assessment_intervals: {
                "glasgow".to_string() => 4,
                "apache".to_string() => 24,
                "sofa".to_string() => 12,
                "saps".to_string() => 24,
                "news2".to_string() => 1,
            },
            retention_days: 255, // ~8 meses
            auto_backup_enabled: true,
        }
    }
}

#[async_trait]
pub trait OlympianGod {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::error>>;
    async fn handle_command(&mut self, command: String) -> Result<String, Box<dyn std::error::error>>;
    async fn get_status(&self) -> serde_json::Value>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::error>>;
    fn name(&self) -> &'static str;
}

// Estructura del sistema completo
#[derive(Debug)]
pub struct OlympusSystemV12 {
    pub config: OlympusConfig,
    pub gods: HashMap<String, Box<dyn OlympianGod>>,
    pub startup_time: chrono::DateTime<chrono::Utc>,
    pub is_running: bool,
    pub metrics: SystemMetrics,
}

#[derive(Debug)]
pub struct SystemMetrics {
    pub total_gods: u32,
    pub active_gods: u32,
    pub total_tasks_completed: u64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
}

impl OlympusSystemV12 {
    pub fn new() -> Self {
        Self {
            config: OlympusConfig::default(),
            gods: HashMap::new(),
            startup_time: chrono::Utc::now(),
            is_running: false,
            metrics: SystemMetrics {
                total_gods: 0,
                active_gods: 0,
                total_tasks_completed: 0,
                average_response_time_ms: 0.0,
                error_rate: 0.0,
                uptime_seconds: 0,
            },
        }
    }

    pub async fn initialize_all_gods(&mut self) -> Result<(), String> {
        let god_names = vec![
            "zeus", "hera", "hades", "poseidon", "artemis", "apollo", "athena", 
            "ares", "aphrodite", "hermes", "iris", "chronos", "hestia",
            "demeter", "dionysius", "erinyes", "moirai", "chaos", "aurora"
        ];

        for god_name in god_names {
            match god_name.as_str() {
                "zeus" => {
                    let god = crate::actors::zeus::ZeusV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "hera" => {
                    let god = crate::actors::hera::HeraV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "hades" => {
                    let god = crate::actors::hades::HadesV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "poseidon" => {
                    let god = crate::actors::poseidon::PoseidonV12::new().await?;
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "artemis" => {
                    let god = crate::actors::artemis::ArtemisV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "apollo" => {
                    let god = crate::actors::apollo::ApolloV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "athena" => {
                    let god = crate::actors::athena::AthenaV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "ares" => {
                    let god = crate::actors::ares::AresV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "aphrodite" => {
                    let god = crate::actors::aphrodite::AphroditeV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "hermes" => {
                    let god = crate::actors::hermes::HermesV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "iris" => {
                    let god = crate::actors::iris::IrisV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "chronos" => {
                    let god = crate::actors::chronos::ChronosV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "hestia" => {
                    let god = crate::actors::hestia::HestiaV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "demeter" => {
                    let god = crate::actors::demeter::DemeterV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "dionysius" => {
                    let god = crate::actors::dionysius::DionysusV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "hestia" => {
                    let god = crate::actors::hestia::HestiaV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "erinyes" => {
                    let god = crate::actors::erinyes::ErinyesV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "moirai" => {
                    let god = crate::actors::moirai::MoiraiV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "chaos" => {
                    let god = crate::actors::chaos::ChaosV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "aurora" => {
                    let god = crate::actors::aurora::AuroraV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "dionysius" => {
                    let god = crate::actors::dionysius::DionysusV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "erinyes" => {
                    let god = crate::actors::erinyes::ErinyesV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "moirai" => {
                    let god = crate::actors::moirai::MoiraiV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "chaos" => {
                    let god = crate::actors::chaos::ChaosV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
                "aurora" => {
                    let god = crate::actors::aurora::AuroraV12::new();
                    self.gods.insert(god_name.to_string(), Box::new(god));
                }
            }
                _ => {
                    tracing::error!("🏛️ Error: Dios desconocido: {}", god_name);
                    return Err(format!("Dios desconocido: {}", god_name));
                }
        }

        tracing::info!("🏛️ Inicializando {} dioses en Olympus v12...", god_names.len());
        
        self.is_running = true;
        self.startup_time = chrono::Utc::now();
        
        Ok(())
    }

    pub async fn start_all_gods(&mut self) -> Result<(), String> {
        let mut started_gods = 0;
        
        for (god_name, god) in self.gods.clone() {
            match god.start().await {
                Ok(_) => {
                    started_gods += 1;
                    tracing::info!("🏛️ {} iniciado exitosamente", god_name);
                }
                Err(e) => {
                    tracing::error!("🚨️ {} falló al iniciar: {}", god_name, e);
                }
            }
        }

        tracing::info!("🏛️ {} dioses iniciados exitosamente", started_gods);
        Ok(())
    }

    pub async fn stop_all_gods(&mut self) -> Result<(), String> {
        let mut stopped_gods = 0;
        
        for (god_name, god) in self.gods.clone() {
            match god.shutdown().await {
                Ok(_) => {
                    stopped_gods += 1;
                    tracing::info!("🏛️ {} detenido correctamente", god_name);
                }
                Err(e) => {
                    tracing::error!("🚨️ Error deteniendo {}: {}", god_name, e);
                }
            }
        }

        tracing::info!("🏛️ {} dioses detenidos", stopped_gods);
        Ok(())
    }

    pub fn get_system_status(&self) -> serde_json::Value {
        let uptime = self.startup_time.elapsed();
        
        serde_json::json!({
            "system": {
                "version": self.config.version,
                "status": if self.is_running { "running" } else { "stopped" },
                "uptime_seconds": uptime.num_seconds(),
                "total_gods": self.gods.len(),
                "active_gods": self.gods.values().filter(|g| g.is_active()).count(),
                "uptime_days": uptime.num_days(),
            }
        })
    }

    pub fn get_all_gods(&self) -> Vec<String> {
        self.gods.keys().cloned().collect()
    }

    pub fn get_god_by_name(&self, name: &str) -> Option<&Box<dyn OlympianGod>> {
        self.gods.get(name)
    }

    pub async fn send_command_to_all(&mut self, command: String) -> Result<Vec<String>, String> {
        let mut responses = Vec::new();
        
        for (god_name, god) in self.gods.clone() {
            match god.handle_command(command).await {
                Ok(response) => responses.push(format!("{}: {}", god_name, response)),
                Err(e) => responses.push(format!("{} error: {} - {}", god_name, e)),
            }
        }
        
        responses
    }

    pub fn update_metrics(&mut self, total_tasks: u64, average_response_time: f64) {
        self.metrics.total_tasks = total_tasks;
        self.metrics.average_response_time_ms = average_response_time;
    }

    pub fn shutdown(mut self) -> Result<(), String> {
        self.stop_all_gods().await?;
        self.is_running = false;
        tracing::info!("🏛️ Olympus v12 shutdown completo");
        
        Ok(())
    }
}

impl Drop for OlympusSystemV12 {
    fn drop(&mut self) {
        if self.is_running {
            tracing::info!("🏛️ Realizando shutdown ordenado de Olympus v12");
        }
    }
}

// Structuras comunes para todos los dioses
#[derive(Debug, Clone)]
pub struct GodActorBase {
    pub name: String,
    pub god_type: String,
    pub status: ActorStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub message_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub enum ActorStatus {
    Initializing,
    Running,
    Stopping,
    Error,
    Shutdown,
}

// Interfaz común para dioses (similar a GodActor pero sin lifecycles complejos)
#[async_trait]
pub trait OlympianGod: GodActorBase {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::error>>;
    async fn handle_command(&mut self, command: String) -> Result<String, Box<dyn std::error::error>>;
    async fn get_status(&self) -> serde_json::Value>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::error>>;
    async fn get_name(&self) -> &'static str;
    async fn heartbeat(&mut self) -> Result<(), Box<dyn std::error::error>>;
    fn name(&self) -> &'static str;
}

// Exportar todos los dioses
pub use crate::actors::{
    artemis_v12 as ArtemisV12,
    apollo_v12 as ApolloV12,
    poseidon_v12 as PoseidonV12,
    iris_v12 as IrisV12,
    zeus_v12 as ZeusV12,
    hera_v12 as HeraV12,
    athena_v12 as AthenaV12,
    hades_v12 as HadesV12,
    ares_v12 as AresV12,
    aphrodite_v12 as AphroditeV12,
    hermes_v12 as HermesV12,
    chronos_v12 as ChronosV12,
    hestia_v12 as HestiaV12,
    demeter_v12 as DemeterV12,
    dionysius_v12 as DionysusV12, // Implementación unificada
    erinyes_v12 as ErinyesV12,
    moirai_v12 as MoiraiV12,
    chaos_v12 as ChaosV12,
    aurora_v12 as AuroraV12,
};