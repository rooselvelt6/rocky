// src/actors/aurora/hope.rs
// OLYMPUS v15 - Aurora: Sistema de Gestión de Esperanza

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::actors::aurora::RenewalType;
use tracing::info;

/// Niveles de esperanza
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HopeLevel {
    /// Desesperanza completa (0%)
    Despair,
    /// Baja esperanza (1-25%)
    Low,
    /// Esperanza moderada (26-50%)
    Moderate,
    /// Esperanza alta (51-75%)
    High,
    /// Esperanza muy alta (76-99%)
    VeryHigh,
    /// Esperanza absoluta (100%)
    Absolute,
}

impl HopeLevel {
    pub fn to_percentage(&self) -> f64 {
        match self {
            HopeLevel::Despair => 0.0,
            HopeLevel::Low => 25.0,
            HopeLevel::Moderate => 50.0,
            HopeLevel::High => 75.0,
            HopeLevel::VeryHigh => 90.0,
            HopeLevel::Absolute => 100.0,
        }
    }
    
    pub fn from_percentage(percentage: f64) -> Self {
        match percentage {
            p if p <= 0.0 => HopeLevel::Despair,
            p if p <= 25.0 => HopeLevel::Low,
            p if p <= 50.0 => HopeLevel::Moderate,
            p if p <= 75.0 => HopeLevel::High,
            p if p <= 95.0 => HopeLevel::VeryHigh,
            _ => HopeLevel::Absolute,
        }
    }
}

/// Evento de esperanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopeEvent {
    /// ID único del evento
    pub event_id: String,
    /// Timestamp del evento
    pub timestamp: DateTime<Utc>,
    /// Tipo de evento
    pub event_type: HopeEventType,
    /// Impacto en la esperanza
    pub hope_impact: f64,
    /// Descripción
    pub description: String,
    /// Contexto
    pub context: HashMap<String, serde_json::Value>,
}

/// Tipos de eventos de esperanza
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HopeEventType {
    /// Renovación exitosa
    SuccessfulRenewal,
    /// Recuperación de fallo
    Recovery,
    /// Nuevo descubrimiento
    Discovery,
    /// Inspiración recibida
    Inspiration,
    /// Obstáculo superado
    ObstacleOvercome,
    /// Hit personal alcanzado
    MilestoneReached,
    /// Acto de bondad
    KindnessAct,
    /// Progreso técnico
    TechnicalProgress,
    /// Conexión humana
    HumanConnection,
}

/// Gestor de esperanza
#[derive(Debug, Clone)]
pub struct HopeManager {
    /// Nivel actual de esperanza
    current_level: Arc<RwLock<f64>>,
    /// Historial de eventos de esperanza
    hope_events: Arc<RwLock<Vec<HopeEvent>>>,
    /// Estadísticas de esperanza
    statistics: Arc<RwLock<HopeStatistics>>,
    /// Configuración
    config: Arc<RwLock<HopeConfig>>,
}

/// Configuración del gestor de esperanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopeConfig {
    /// Nivel inicial de esperanza
    pub initial_hope_level: f64,
    /// Tasa de decaimiento natural por hora
    pub natural_decay_rate: f64,
    /// Umbral para activar eventos positivos
    pub positive_event_threshold: f64,
    /// Máximo de eventos a guardar
    pub max_events_retained: usize,
    /// Activar modo resiliencia automático
    pub auto_resilience_mode: bool,
}

impl Default for HopeConfig {
    fn default() -> Self {
        Self {
            initial_hope_level: 75.0,
            natural_decay_rate: 0.1,
            positive_event_threshold: 50.0,
            max_events_retained: 1000,
            auto_resilience_mode: true,
        }
    }
}

/// Estadísticas de esperanza
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopeStatistics {
    /// Eventos totales registrados
    pub total_events: u64,
    /// Eventos por tipo
    pub events_by_type: HashMap<String, u64>,
    /// Promedio de nivel de esperanza (últimas 24h)
    pub average_hope_24h: f64,
    /// Mayor nivel alcanzado
    pub peak_hope_level: f64,
    /// Menor nivel registrado
    pub lowest_hope_level: f64,
    /// Tasa de recuperación de esperanza
    pub recovery_rate: f64,
    /// Última actualización
    pub last_updated: DateTime<Utc>,
}

impl Default for HopeStatistics {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            average_hope_24h: 75.0,
            peak_hope_level: 75.0,
            lowest_hope_level: 75.0,
            recovery_rate: 1.0,
            last_updated: Utc::now(),
        }
    }
}

impl HopeManager {
    /// Crea un nuevo gestor de esperanza
    pub fn new(config: HopeConfig) -> Self {
        Self {
            current_level: Arc::new(RwLock::new(config.initial_hope_level)),
            hope_events: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(HopeStatistics::default())),
            config: Arc::new(RwLock::new(config)),
        }
    }
    
    /// Obtiene el nivel actual de esperanza
    pub async fn get_hope_level(&self) -> f64 {
        *self.current_level.read().await
    }
    
    /// Establece el nivel de esperanza
    pub async fn set_hope_level(&self, level: f64) {
        let mut current = self.current_level.write().await;
        let old_level = *current;
        *current = level.clamp(0.0, 100.0);
        
        // Actualizar estadísticas
        self.update_statistics(old_level, level).await;
        
        info!("🌈 Nivel de esperanza actualizado: {:.1}% -> {:.1}%", old_level, level);
    }
    
    /// Incrementa la esperanza
    pub async fn increase_hope(&self, amount: f64, reason: &str, context: HashMap<String, serde_json::Value>) {
        let mut current = self.current_level.write().await;
        let old_level = *current;
        *current = (*current + amount).clamp(0.0, 100.0);
        
        // Registrar evento positivo
        let event = HopeEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: HopeEventType::Inspiration,
            hope_impact: amount,
            description: reason.to_string(),
            context,
        };
        
        self.register_event(event).await;
        self.update_statistics(old_level, *current).await;
        
        info!("🌈 Esperanza incrementada en {:.1}%: {}", amount, reason);
    }
    
    /// Decrementa la esperanza (evento negativo)
    pub async fn decrease_hope(&self, amount: f64, reason: &str, context: HashMap<String, serde_json::Value>) {
        let mut current = self.current_level.write().await;
        let old_level = *current;
        *current = (*current - amount).clamp(0.0, 100.0);
        
        // Registrar evento negativo
        let event = HopeEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: HopeEventType::ObstacleOvercome,
            hope_impact: -amount,
            description: reason.to_string(),
            context,
        };
        
        self.register_event(event).await;
        self.update_statistics(old_level, *current).await;
        
        info!("🌈 Esperanza decrementada en {:.1}%: {}", amount, reason);
    }
    
    /// Aplica decaimiento natural de esperanza
    pub async fn apply_natural_decay(&self) {
        let config = self.config.read().await;
        let decay_rate = config.natural_decay_rate;
        
        if decay_rate > 0.0 {
            let mut current = self.current_level.write().await;
            let old_level = *current;
            *current = (*current - decay_rate).max(0.0);
            
            self.update_statistics(old_level, *current).await;
            
            if *current < old_level {
                info!("🌈 Aplicado decaimiento natural: {:.1}% -> {:.1}%", old_level, *current);
            }
        }
    }
    
    /// Recupera esperanza automáticamente si está por debajo del umbral
    pub async fn auto_recovery(&self) {
        let config = self.config.read().await;
        
        if !config.auto_resilience_mode {
            return;
        }
        
        let current_level = self.get_hope_level().await;
        
        if current_level < config.positive_event_threshold {
            let recovery_amount = (config.positive_event_threshold - current_level) * 0.5;
            
            let mut current = self.current_level.write().await;
            *current = (*current + recovery_amount).clamp(0.0, 100.0);
            
            let event = HopeEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: HopeEventType::Recovery,
                hope_impact: recovery_amount,
                description: "Recuperación automática de esperanza".to_string(),
                context: std::collections::HashMap::from([
                    ("trigger_level".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(current_level).unwrap())),
                    ("recovery_amount".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(recovery_amount).unwrap())),
                ]),
            };
            
            self.register_event(event).await;
            self.update_statistics(current_level, *current).await;
            
            info!("🌈 Recuperación automática aplicada: +{:.1}%", recovery_amount);
        }
    }
    
    /// Obtiene estadísticas actuales
    pub async fn get_statistics(&self) -> HopeStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Obtiene eventos recientes
    pub async fn get_recent_events(&self, limit: usize) -> Vec<HopeEvent> {
        let events = self.hope_events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }
    
    /// Registra un evento de esperanza
    async fn register_event(&self, event: HopeEvent) {
        let mut events = self.hope_events.write().await;
        events.push(event);
        
        // Limitar cantidad de eventos
        let config = self.config.read().await;
        if events.len() > config.max_events_retained {
            let len_to_keep = config.max_events_retained;
            let current_len = events.len();
            events.drain(0..current_len - len_to_keep);
        }
    }
    
    /// Actualiza estadísticas internas
    async fn update_statistics(&self, old_level: f64, new_level: f64) {
        let mut stats = self.statistics.write().await;
        
        // Actualizar peaks
        stats.peak_hope_level = stats.peak_hope_level.max(new_level);
        stats.lowest_hope_level = stats.lowest_hope_level.min(new_level);
        
        // Calcular tasa de recuperación
        if old_level < new_level {
            stats.recovery_rate = (stats.recovery_rate * 0.9) + (new_level - old_level) * 0.1;
        }
        
        stats.last_updated = Utc::now();
    }
}