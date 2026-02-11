// src/actors/aurora/inspiration.rs
// OLYMPUS v15 - Aurora: Motor de Inspiración

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::actors::aurora::RenewalType;
use crate::errors::ActorError;
use tracing::info;

/// Tipos de inspiración
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspirationType {
    /// Inspiración técnica
    Technical,
    /// Inspiración creativa
    Creative,
    /// Inspiración emocional
    Emotional,
    /// Inspiración espiritual
    Spiritual,
    /// Inspiración práctica
    Practical,
    /// Inspiración revolucionaria
    Revolutionary,
    /// Inspiración sanadora
    Healing,
    /// Inspiración educativa
    Educational,
    /// Inspiración artística
    Artistic,
}

/// Niveles de inspiración
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InspirationLevel {
    /// Chispa diminuta (1-10)
    Spark { intensity: u8 },
    /// Ideas fluidas (11-30)
    Flow { momentum: u8 },
    /// Visión clara (31-60)
    Vision { clarity: u8 },
    /// Revelación profunda (61-90)
    Revelation { depth: u8 },
    /// Éxtasis creativo (91-100)
    Ecstasy { transcendence: u8 },
}

impl InspirationLevel {
    pub fn to_numeric(&self) -> u8 {
        match self {
            InspirationLevel::Spark { intensity } => 5 + intensity / 25,
            InspirationLevel::Flow { momentum } => 15 + momentum / 6,
            InspirationLevel::Vision { clarity } => 40 + clarity / 3,
            InspirationLevel::Revelation { depth } => 65 + depth / 4,
            InspirationLevel::Ecstasy { transcendence } => 85 + transcendence / 6,
        }
    }
    
    pub fn from_numeric(value: u8) -> Self {
        match value {
            v if v <= 10 => InspirationLevel::Spark { intensity: v * 25 / 10 },
            v if v <= 30 => InspirationLevel::Flow { momentum: ((v - 15) * 6).min(255) as u8 },
            v if v <= 60 => InspirationLevel::Vision { clarity: ((v - 40) * 3).min(255) as u8 },
            v if v <= 90 => InspirationLevel::Revelation { depth: ((v - 65) * 4).min(255) as u8 },
            v => InspirationLevel::Ecstasy { transcendence: ((v - 85) * 6).min(255) as u8 },
        }
    }
}

/// Fuentes de inspiración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InspirationSource {
    /// Meditación profunda
    DeepMeditation,
    /// Observación de la naturaleza
    NatureObservation,
    /// Conversación significativa
    MeaningfulConversation,
    /// Lectura inspiradora
    InspirationalReading,
    /// Experiencia transformadora
    TransformativeExperience,
    /// Sueños lúcidos
    LucidDream,
    /// Arte poderoso
    PowerfulArt,
    /// Música elevada
    UpliftingMusic,
    /// Ejercicio físico
    PhysicalExercise,
    /// Silencio contemplativo
    ContemplativeSilence,
    /// Fuente personalizada
    Custom(String),
}

/// Evento de inspiración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspirationEvent {
    /// ID único de la inspiración
    pub inspiration_id: String,
    /// Timestamp de la inspiración
    pub timestamp: DateTime<Utc>,
    /// Tipo de inspiración
    pub inspiration_type: InspirationType,
    /// Nivel de inspiración
    pub level: InspirationLevel,
    /// Fuente de la inspiración
    pub source: InspirationSource,
    /// Contenido/idea inspirada
    pub content: String,
    /// Contexto ambiental
    pub context: HashMap<String, serde_json::Value>,
    /// Duración estimada (minutos)
    pub estimated_duration_minutes: u32,
    /// Impacto potencial
    pub potential_impact: f64,
    /// Etiquetas descriptivas
    pub tags: Vec<String>,
}

/// Motor de inspiración
#[derive(Debug, Clone)]
pub struct InspirationEngine {
    /// Inspiraciones activas
    active_inspirations: Arc<RwLock<Vec<InspirationEvent>>>,
    /// Historial de inspiraciones
    inspiration_history: Arc<RwLock<Vec<InspirationEvent>>>,
    /// Estadísticas de inspiración
    statistics: Arc<RwLock<InspirationStatistics>>,
    /// Configuración
    config: Arc<RwLock<InspirationConfig>>,
}

/// Configuración del motor de inspiración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspirationConfig {
    /// Sensibilidad a inspiraciones
    pub sensitivity: f64,
    /// Máximo de inspiraciones simultáneas
    pub max_concurrent_inspirations: usize,
    /// Umbral mínimo de nivel
    pub minimum_level_threshold: u8,
    /// Fuentes preferidas
    pub preferred_sources: Vec<InspirationSource>,
    /// Activar modo captura automática
    pub auto_capture_mode: bool,
    /// Período de escaneo (minutos)
    pub scan_period_minutes: u32,
}

impl Default for InspirationConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.75,
            max_concurrent_inspirations: 5,
            minimum_level_threshold: 10,
            preferred_sources: vec![
                InspirationSource::DeepMeditation,
                InspirationSource::NatureObservation,
                InspirationSource::MeaningfulConversation,
            ],
            auto_capture_mode: true,
            scan_period_minutes: 15,
        }
    }
}

/// Estadísticas de inspiración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspirationStatistics {
    /// Total de inspiraciones recibidas
    pub total_inspirations: u64,
    /// Inspiraciones por tipo
    pub inspirations_by_type: HashMap<String, u64>,
    /// Inspiraciones por fuente
    pub inspirations_by_source: HashMap<String, u64>,
    /// Promedio de nivel de inspiración
    pub average_inspiration_level: f64,
    /// Mayor nivel alcanzado
    pub peak_inspiration_level: u8,
    /// Tasa de inspiraciones por hora
    pub inspirations_per_hour: f64,
    /// Duración promedio de inspiraciones
    pub average_duration_minutes: f64,
    /// Impacto total acumulado
    pub total_impact_score: f64,
    /// Última actualización
    pub last_updated: DateTime<Utc>,
}

impl Default for InspirationStatistics {
    fn default() -> Self {
        Self {
            total_inspirations: 0,
            inspirations_by_type: HashMap::new(),
            inspirations_by_source: HashMap::new(),
            average_inspiration_level: 35.0,
            peak_inspiration_level: 50,
            inspirations_per_hour: 2.5,
            average_duration_minutes: 20.0,
            total_impact_score: 0.0,
            last_updated: Utc::now(),
        }
    }
}

impl InspirationEngine {
    /// Crea un nuevo motor de inspiración
    pub fn new(config: InspirationConfig) -> Self {
        Self {
            active_inspirations: Arc::new(RwLock::new(Vec::new())),
            inspiration_history: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(InspirationStatistics::default())),
            config: Arc::new(RwLock::new(config)),
        }
    }
    
    /// Escanear nuevas inspiraciones
    pub async fn scan_for_inspirations(&self) -> Result<Vec<InspirationEvent>, ActorError> {
        let config = self.config.read().await;
        let mut inspirations = Vec::new();
        
        // Simular escaneo de diferentes fuentes
        if config.auto_capture_mode {
            // Inspiración técnica del sistema
            inspirations.push(self.generate_technical_inspiration().await);
            
            // Inspiración creativa aleatoria
            if rand::random::<f64>() < config.sensitivity {
                inspirations.push(self.generate_creative_inspiration().await);
            }
            
            // Inspiración espiritual profunda
            if rand::random::<f64>() < config.sensitivity * 0.5 {
                inspirations.push(self.generate_spiritual_inspiration().await);
            }
        }
        
        // Filtrar por umbral mínimo
        inspirations.retain(|insp| insp.level.to_numeric() >= config.minimum_level_threshold);
        
        // Limitar cantidad concurrente
        let active = self.active_inspirations.read().await;
        let remaining_slots = config.max_concurrent_inspirations.saturating_sub(active.len());
        
        if inspirations.len() > remaining_slots {
            inspirations.truncate(remaining_slots);
        }
        
        Ok(inspirations)
    }
    
    /// Agregar una inspiración manualmente
    pub async fn add_inspiration(&self, inspiration: InspirationEvent) -> Result<(), ActorError> {
        // Validar nivel mínimo
        let config = self.config.read().await;
        if inspiration.level.to_numeric() < config.minimum_level_threshold {
            return Err(ActorError::ValidationError(
                format!("Nivel de inspiración {} por debajo del umbral mínimo {}", 
                       inspiration.level.to_numeric(), config.minimum_level_threshold)
            ));
        }
        
        // Agregar a activas
        let mut active = self.active_inspirations.write().await;
        
        if active.len() >= config.max_concurrent_inspirations {
            // Remover la más antigua
            if let Some(old) = active.first() {
                self.complete_inspiration(old).await;
            }
            active.remove(0);
        }
        
        active.push(inspiration.clone());
        
        // Actualizar estadísticas
        self.update_statistics(&inspiration, true).await;
        
        info!("✨ Nueva inspiración añadida: {} ({})", 
               inspiration.content, 
               inspiration.inspiration_type);
        
        Ok(())
    }
    
    /// Completar una inspiración
    pub async fn complete_inspiration(&self, inspiration: &InspirationEvent) {
        let mut active = self.active_inspirations.write().await;
        
        // Remover de activas
        if let Some(index) = active.iter().position(|i| i.inspiration_id == inspiration.inspiration_id) {
            active.remove(index);
        }
        
        // Agregar al historial
        let mut history = self.inspiration_history.write().await;
        history.push(inspiration.clone());
        
        // Limitar historial
        if history.len() > 10000 {
            history.drain(0..history.len() - 10000);
        }
        
        self.update_statistics(inspiration, false).await;
        
        info!("✨ Inspiración completada: {}", inspiration.content);
    }
    
    /// Obtener inspiraciones activas
    pub async fn get_active_inspirations(&self) -> Vec<InspirationEvent> {
        self.active_inspirations.read().await.clone()
    }
    
    /// Obtener historial reciente
    pub async fn get_recent_history(&self, limit: usize) -> Vec<InspirationEvent> {
        let history = self.inspiration_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
    
    /// Obtener estadísticas
    pub async fn get_statistics(&self) -> InspirationStatistics {
        self.statistics.read().await.clone()
    }
    
    /// Generar inspiración técnica
    async fn generate_technical_inspiration(&self) -> InspirationEvent {
        let technical_ideas = vec![
            "Optimizar queries de base de datos usando índices compuestos",
            "Implementar cache distribuido con invalidación predictiva",
            "Diseñar sistema de auto-escalado basado en patrones de carga",
            "Crear protocolo de comunicación quántico-resistente",
            "Desarrollar algoritmo de compresión neuronal para logs",
            "Implementar sistema de resiliencia con checkpoints atómicos",
            "Diseñar arquitectura serverless con cold start optimizado",
            "Crear motor de inferencia de datos para predicción de fallos",
        ];
        
        let idea = technical_ideas[rand::random::<usize>() % technical_ideas.len()];
        
        InspirationEvent {
            inspiration_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            inspiration_type: InspirationType::Technical,
            level: InspirationLevel::Flow { momentum: 180 },
            source: InspirationSource::DeepMeditation,
            content: idea.to_string(),
            context: HashMap::from([
                ("category".to_string(), serde_json::Value::String("technical".to_string())),
                ("priority".to_string(), serde_json::Value::String("high".to_string())),
            ]),
            estimated_duration_minutes: 45,
            potential_impact: 85.5,
            tags: vec!["technical".to_string(), "optimization".to_string(), "performance".to_string()],
        }
    }
    
    /// Generar inspiración creativa
    async fn generate_creative_inspiration(&self) -> InspirationEvent {
        let creative_ideas = vec![
            "Crear dashboard interactivo con visualización de datos en tiempo real",
            "Diseñar sistema de gamificación para mejorar用户体验",
            "Implementar modo oscuro con transiciones suaves y personalizable",
            "Desarrollar asistente de IA conversacional para ayuda contextual",
            "Crear sistema de notificaciones inteligentes basado en comportamiento",
            "Diseñar interfaz adaptable que aprende del usuario",
            "Implementar colaboración en tiempo real con cambios visibles",
            "Crear sistema de feedback emocional para mejorar la experiencia",
        ];
        
        let idea = creative_ideas[rand::random::<usize>() % creative_ideas.len()];
        
        InspirationEvent {
            inspiration_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            inspiration_type: InspirationType::Creative,
            level: InspirationLevel::Vision { clarity: 120 },
            source: InspirationSource::NatureObservation,
            content: idea.to_string(),
            context: HashMap::from([
                ("category".to_string(), serde_json::Value::String("creative".to_string())),
                ("medium".to_string(), serde_json::Value::String("ui".to_string())),
            ]),
            estimated_duration_minutes: 60,
            potential_impact: 75.0,
            tags: vec!["creative".to_string(), "design".to_string(), "user-experience".to_string()],
        }
    }
    
    /// Generar inspiración espiritual
    async fn generate_spiritual_inspiration(&self) -> InspirationEvent {
        let spiritual_ideas = vec![
            "Implementar sistema de gratitud diaria con impacto medible",
            "Crear espacio de meditación guiada con sonidos naturales",
            "Diseñar rituales de cierre de jornada con reflexión profunda",
            "Desarrollar sistema de conexión humana basado en intereses compartidos",
            "Implementar modo de minimalismo digital para reducir ruido",
            "Crear jardín virtual que crece con actos positivos",
            "Diseñar sistema de legado digital para futuras generaciones",
        ];
        
        let idea = spiritual_ideas[rand::random::<usize>() % spiritual_ideas.len()];
        
        InspirationEvent {
            inspiration_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            inspiration_type: InspirationType::Spiritual,
            level: InspirationLevel::Revelation { depth: 85 },
            source: InspirationSource::ContemplativeSilence,
            content: idea.to_string(),
            context: HashMap::from([
                ("category".to_string(), serde_json::Value::String("spiritual".to_string())),
                ("impact".to_string(), serde_json::Value::String("human".to_string())),
            ]),
            estimated_duration_minutes: 30,
            potential_impact: 95.0,
            tags: vec!["spiritual".to_string(), "human".to_string(), "connection".to_string()],
        }
    }
    
    /// Actualizar estadísticas de inspiración
    async fn update_statistics(&self, inspiration: &InspirationEvent, is_new: bool) {
        let mut stats = self.statistics.write().await;
        
        if is_new {
            stats.total_inspirations += 1;
        }
        
        // Actualizar contadores por tipo
        let type_key = format!("{:?}", inspiration.inspiration_type);
        *stats.inspirations_by_type.entry(type_key).or_insert(0) += 1;
        
        // Actualizar contadores por fuente
        let source_key = format!("{:?}", inspiration.source);
        *stats.inspirations_by_source.entry(source_key).or_insert(0) += 1;
        
        // Actualizar peak
        let level_numeric = inspiration.level.to_numeric();
        stats.peak_inspiration_level = stats.peak_inspiration_level.max(level_numeric);
        
        // Actualizar promedio
        if stats.total_inspirations > 0 {
            let current_avg = stats.average_inspiration_level;
            stats.average_inspiration_level = (current_avg * (stats.total_inspirations - 1) as f64 + level_numeric as f64) / stats.total_inspirations as f64;
        }
        
        // Actualizar impacto total
        stats.total_impact_score += inspiration.potential_impact;
        
        stats.last_updated = Utc::now();
    }
}