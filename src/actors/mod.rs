/// 🏛️ OLYMPUS v13 - EL PANTÉÓN DIVINO COMPLETO
/// 20 dioses con dominios especializados bajo arquitectura OTP
/// Sistema autoregenerativo tolerante a fallos

// 🔥 TRINIDAD SUPREMA (3 dioses)
pub mod zeus;           // ⚡ Zeus - Rey del Olimpo y Supervisor Supremo
pub mod erinyes;        // 🏹 Erinyes - Diosa de la Venganza y Recuperación
pub mod poseidon;       // 🌊 Poseidón - Dios de los Mares y Flujo de Datos

// 🏛️ DIOSITAS CLÍNICAS (4 dioses)
pub mod athena;         // 🦉 Athena - Diosa de la Sabiduría Clínica
pub mod apollo;         // ☀️ Apollo - Dios de las Artes y Eventos
pub mod artemis;        // 🏹 Artemis - Diosa de la Caza y Búsqueda
pub mod hermes;         // 👟 Hermes - Mensajero Divino y Routing

// 🔐 DIOSITAS DE SEGURIDAD (2 dioses)
pub mod hades;          // 🔱 Hades - Dios del Inframundo y Seguridad
pub mod hera;           // 👑 Hera - Reina de los Dioses y Validación

// ⚔️ DIOSITAS DE GOBIERNO (2 dioses)
pub mod ares;           // ⚔️ Ares - Dios de la Guerra y Conflictos
pub mod hefesto;        // 🔥 Hefesto - Dios de la Forja y Configuración

// ⏰️ DIOSITAS DE TIEMPO (1 dios)
pub mod chronos;        // ⏰️ Chronos - Dios del Tiempo y Scheduling

// 🧵 DIOSITAS DE DESTINO (1 dios)
pub mod moirai;         // 🧵 Moirai - Diosas del Destino y Predicciones

// 🌀 DIOSITAS DE CAOS (1 dios)
pub mod chaos;          // 🌀 Chaos - Dios del Caos y Testing

// 🌅 DIOSITAS DE ESPERANZA (1 dios)
pub mod aurora;         // 🌅 Aurora - Diosa del Amanecer y Nuevos Inicios

// 💕 DIOSITAS DE BELLEZA (1 dios)
pub mod aphrodite;      // 💕 Aphrodite - Diosa de la Belleza y UI

// 🕊️ DIOSITAS DE COMUNICACIÓN (1 dios)
pub mod iris;           // 🕊️ Iris - Diosa del Arcoíris y Comunicaciones

// 🌾 DIOSITAS DE RECURSOS (1 dios)
pub mod demeter;        // 🌾 Demeter - Diosa de la Agricultura y Recursos

// 🍷 DIOSITAS DE ANÁLISIS (1 dios)
pub mod dionysus;       // 🍷 Dionisio - Dios del Vino y Análisis

// 🏠 PERSISTENCIA (1 dios)
pub mod hestia;         // 🏠 Hestia - Diosa del Hogar y Persistencia

// 🦋 DIOSITAS DE JUSTICIA (1 dios)
pub mod nemesis;        // 🦋 Némesis - Diosa de la Justicia Legal y Cumplimiento

// Actor interfaces for v13
use serde::{Deserialize, Serialize};

// Enum de todos los nombres de dioses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GodName {
    #[default]
    Zeus,
    Erinyes,
    Poseidon,
    Athena,
    Apollo,
    Artemis,
    Hermes,
    Hades,
    Hera,
    Ares,
    Hefesto,
    Chronos,
    Moirai,
    Chaos,
    Aurora,
    Aphrodite,
    Iris,
    Demeter,
    Dionysus,
    Nemesis,
    Hestia,
}

impl std::fmt::Display for GodName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GodName::Zeus => write!(f, "Zeus"),
            GodName::Erinyes => write!(f, "Erinyes"),
            GodName::Poseidon => write!(f, "Poseidon"),
            GodName::Athena => write!(f, "Athena"),
            GodName::Apollo => write!(f, "Apollo"),
            GodName::Artemis => write!(f, "Artemis"),
            GodName::Hermes => write!(f, "Hermes"),
            GodName::Hades => write!(f, "Hades"),
            GodName::Hera => write!(f, "Hera"),
            GodName::Ares => write!(f, "Ares"),
            GodName::Hefesto => write!(f, "Hefesto"),
            GodName::Chronos => write!(f, "Chronos"),
            GodName::Moirai => write!(f, "Moirai"),
            GodName::Chaos => write!(f, "Chaos"),
            GodName::Aurora => write!(f, "Aurora"),
            GodName::Aphrodite => write!(f, "Aphrodite"),
            GodName::Iris => write!(f, "Iris"),
            GodName::Demeter => write!(f, "Demeter"),
            GodName::Dionysus => write!(f, "Dionysus"),
            GodName::Hestia => write!(f, "Hestia"),
            GodName::Nemesis => write!(f, "Nemesis"),
        }
    }
}

// Dominio de cada dios
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivineDomain {
    Governance,          // Zeus
    Integrity,           // Erinyes
    DataFlow,            // Poseidon
    Clinical,            // Athena
    Events,              // Apollo
    Search,              // Artemis
    Messaging,           // Hermes
    Security,            // Hades
    Validation,          // Hera
    ConflictResolution,  // Ares
    Configuration,       // Hefesto
    Scheduling,          // Chronos
    Predictions,         // Moirai
    Testing,             // Chaos
    NewBeginnings,       // Aurora
    UI,                  // Aphrodite
    Communications,      // Iris
    Resources,           // Demeter
    Analysis,            // Dionysus
    Persistence,         // Hestia
    LegalCompliance,      // Nemesis
}

// Estado del Olimpo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympusState {
    pub initialized: bool,
    pub uptime_seconds: u64,
    pub active_gods: Vec<GodName>,
    pub dead_gods: Vec<GodName>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub system_status: SystemStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Critical,
    Emergency,
}

impl Default for OlympusState {
    fn default() -> Self {
        Self {
            initialized: false,
            uptime_seconds: 0,
            active_gods: Vec::new(),
            dead_gods: Vec::new(),
            last_health_check: chrono::Utc::now(),
            system_status: SystemStatus::Healthy,
        }
    }
}

// Métricas globales del Olimpo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympusMetrics {
    pub total_messages_processed: u64,
    pub total_errors: u64,
    pub total_restarts: u64,
    pub total_recoveries: u64,
    pub average_recovery_time_ms: u64,
    pub dead_letters_count: u64,
    pub memory_usage_mb: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for OlympusMetrics {
    fn default() -> Self {
        Self {
            total_messages_processed: 0,
            total_errors: 0,
            total_restarts: 0,
            total_recoveries: 0,
            average_recovery_time_ms: 0,
            dead_letters_count: 0,
            memory_usage_mb: 0.0,
            last_update: chrono::Utc::now(),
        }
    }
}

