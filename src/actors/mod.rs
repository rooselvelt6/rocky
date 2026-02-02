/// 🏛️ OLYMPUS v12 - EL PANTÉÓN DIVINO COMPLETO
/// 20 dioses con dominios especializados bajo arquitectura OTP
/// Cada dios basado en su mitología con funciones específicas

// 🔥 TRINIDAD SUPREMA (3 dioses)
pub mod zeus;      // 🏛️ Zeus - Rey del Olimpo y Supervisor Principal
pub mod hera;        // 👑 Hera - Reina de los Dioses, Guardiana de Invariantes
pub mod hades;        // 🔱 Hades - Dios del Inframundo y Criptografía

// 🏛️ DIOSITAS CLÍNICAS (4 dioses)
pub mod athena;       // 🦉 Athena - Diosa de la Sabiduría y Estrategia Clínica
pub mod apollo;        // ☀️ Apollo - Dios de las Artes, Música y Conocimiento
pub mod artemis;       // 🏹 Artemis - Diosa de la Caza y Protección
pub mod hermes;        // 👟 Hermes - Mensajero Divino y Rapidez

// 🌊 DIOSITAS TÉCNICAS (4 dioses)
pub mod poseidon;      // 🌊 Poseidón - Dios de los Mares y Bases de Datos
pub mod demeter;       // 🌾 Demeter - Diosa de la Agricultura y Recursos
pub mod dionysius;     // 🍷️ Dionisio - Dios del Vino, Fiestas y Análisis (implementación unificada)

// 🌊 DIOSITAS OPERACIONALES (6 dioses)
pub mod iris;          // 🕊️ Iris - Diosa del Arcoíris y Comunicación
pub mod ares;        // ⚔️ Ares - Dios de la Guerra y Conflictos
pub mod aphrodite;      // 💕️ Aphrodite - Diosa de la Belleza y el Amor

// 🌊 DIOSITAS SISTEMAS (6 dioses)
pub mod chronos;        // ⏰️ Chronos - Dios del Tiempo y Destino
pub mod hefesto;        // 🔥 Hefesto - Dios de la Forja y Sistemas
pub mod hestia;        // 🏛️ Hestia - Diosa del Hogar y Configuración
pub mod erinyes;      // 🏹 Erinyes - Diosas de la Venganza y Justicia Retributiva
pub mod moirai;       // 🧵 Moirai - Diosas del Destino y Hilos de la Vida
pub mod chaos;        // 🌀 Chaos - Dios del Caos y Testing
pub mod aurora;       // 🌅 Aurora - Diosa del Amanecer y Nuevos Comienzos

// Actor interfaces for external systems
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GodName {
    Zeus, Hera, Hades, Poseidon, Artemis, Apollo, Athena, Ares, Aphrodite, Hermes,
    Chronos, Hestia, Demeter, Dionysus, Iris, Erinyes, Moirai, Chaos, Aurora
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DivineDomain {
    SystemConfig, JusticeAndRetribution, DestinyAndFate, ChaosEngineering, HopeAndRenewal,
    DataAnalysis, Security, Communication, Warfare, Strategy, Healing,
    TimeManagement, ResourceManagement, PerformanceMonitoring, Innovation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympianMessage {
    pub sender: GodName,
    pub command: String,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub type OlympicResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[async_trait]
pub trait OlympianGod: Send + Sync {
    async fn process_message(&self, message: OlympianMessage) -> OlympicResult<OlympianMessage>;
    fn get_name(&self) -> GodName;
    fn get_domain(&self) -> DivineDomain;
    async fn get_status(&self) -> OlympicResult<serde_json::Value>;
}

// Re-export para uso externo
pub use zeus::*;
pub use hera::*;
pub use hades::*;
pub use athena::*;
pub use apollo::*;
pub use artemis::*;
pub use hermes::*;
pub use poseidon::*;
pub use demeter::*;
pub use dionysius::*;
pub use iris::*;
pub use ares::*;
pub use aphrodite::*;
pub use chronos::*;
pub use hefesto::*;
pub use hestia::*;
pub use erinyes::*;
pub use moirai::*;
pub use chaos::*;
pub use aurora::*;