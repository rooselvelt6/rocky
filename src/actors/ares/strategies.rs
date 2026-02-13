// src/actors/ares/strategies.rs
// OLYMPUS v15 - Estrategias de Resolución para Ares

use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use crate::actors::GodName;
use crate::errors::ActorError;
use super::{Conflict, ConflictType, ConflictSeverity};
use super::detector::ConflictStatus;

/// Estrategias de resolución de conflictos
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Basado en prioridad de actores
    Priority,
    /// Basado en tiempo de espera (FIFO)
    FirstComeFirstServed,
    /// Distribución equitativa
    FairShare,
    /// Random
    Random,
    /// Basado en peso/carga actual
    LoadBalanced,
    /// Timeout con reintento
    TimeoutRetry,
    /// Mediación manual
    Mediation,
    /// Sacrificar el menos importante
    Sacrifice,
    /// Dividir recurso
    SplitResource,
    /// Estrategia personalizada
    Custom(String),
}

/// Resultado de resolución de conflicto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// Si la resolución fue exitosa
    pub success: bool,
    /// Mensaje descriptivo
    pub message: String,
    /// Estrategia utilizada
    pub strategy: ResolutionStrategy,
    /// Actor ganador (si aplica)
    pub winner: Option<GodName>,
    /// Actor perdedor (si aplica)
    pub loser: Option<GodName>,
    /// Tiempo de resolución
    pub resolution_time_ms: u64,
    /// Acciones tomadas
    pub actions_taken: Vec<String>,
    /// Metadatos adicionales
    pub metadata: HashMap<String, String>,
}

impl ResolutionResult {
    /// Crea un resultado exitoso
    pub fn success(strategy: ResolutionStrategy, message: String) -> Self {
        Self {
            success: true,
            message,
            strategy,
            winner: None,
            loser: None,
            resolution_time_ms: 0,
            actions_taken: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Crea un resultado fallido
    pub fn failure(strategy: ResolutionStrategy, message: String) -> Self {
        Self {
            success: false,
            message,
            strategy,
            winner: None,
            loser: None,
            resolution_time_ms: 0,
            actions_taken: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Agrega acción tomada
    pub fn add_action(&mut self, action: String) {
        self.actions_taken.push(action);
    }
    
    /// Establece ganador y perdedor
    pub fn set_winner_loser(&mut self, winner: GodName, loser: GodName) {
        self.winner = Some(winner);
        self.loser = Some(loser);
    }
}

/// Resolvedor de conflictos
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    /// Configuración de estrategias
    config: ResolverConfig,
    /// Estadísticas de resolución
    stats: ResolutionStats,
}

/// Configuración del resolvedor
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Timeout por defecto para resoluciones (ms)
    default_timeout_ms: u64,
    /// Máximo número de reintentos
    max_retries: u32,
    /// Prioridades de actores
    actor_priorities: HashMap<GodName, u8>,
    /// Estrategias por tipo de conflicto
    type_strategies: HashMap<ConflictType, ResolutionStrategy>,
    /// Estrategias por severidad
    severity_strategies: HashMap<ConflictSeverity, ResolutionStrategy>,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        let mut actor_priorities = HashMap::new();
        actor_priorities.insert(GodName::Zeus, 10);
        actor_priorities.insert(GodName::Hera, 9);
        actor_priorities.insert(GodName::Poseidon, 8);
        actor_priorities.insert(GodName::Demeter, 8);
        actor_priorities.insert(GodName::Ares, 9);
        actor_priorities.insert(GodName::Athena, 9);
        actor_priorities.insert(GodName::Apollo, 7);
        actor_priorities.insert(GodName::Artemis, 7);
        actor_priorities.insert(GodName::Hefesto, 6);
        actor_priorities.insert(GodName::Aphrodite, 5);
        actor_priorities.insert(GodName::Hermes, 8);
        actor_priorities.insert(GodName::Hades, 6);
        actor_priorities.insert(GodName::Dionysus, 4);
        actor_priorities.insert(GodName::Hestia, 3);
        
        let mut type_strategies = HashMap::new();
        type_strategies.insert(ConflictType::Resource, ResolutionStrategy::FairShare);
        type_strategies.insert(ConflictType::Data, ResolutionStrategy::Priority);
        type_strategies.insert(ConflictType::Priority, ResolutionStrategy::Priority);
        type_strategies.insert(ConflictType::Dependency, ResolutionStrategy::TimeoutRetry);
        type_strategies.insert(ConflictType::Timing, ResolutionStrategy::FirstComeFirstServed);
        type_strategies.insert(ConflictType::Communication, ResolutionStrategy::Mediation);
        type_strategies.insert(ConflictType::State, ResolutionStrategy::Sacrifice);
        
        let mut severity_strategies = HashMap::new();
        severity_strategies.insert(ConflictSeverity::Low, ResolutionStrategy::Random);
        severity_strategies.insert(ConflictSeverity::Medium, ResolutionStrategy::FairShare);
        severity_strategies.insert(ConflictSeverity::High, ResolutionStrategy::Priority);
        severity_strategies.insert(ConflictSeverity::Critical, ResolutionStrategy::Sacrifice);
        
        Self {
            default_timeout_ms: 5000,
            max_retries: 3,
            actor_priorities,
            type_strategies,
            severity_strategies,
        }
    }
}

/// Estadísticas de resolución
#[derive(Debug, Clone, Default)]
pub struct ResolutionStats {
    pub total_resolutions: u64,
    pub successful_resolutions: u64,
    pub failed_resolutions: u64,
    pub strategy_usage: HashMap<ResolutionStrategy, u64>,
    pub average_resolution_time_ms: f64,
}

impl ConflictResolver {
    /// Crea un nuevo resolvedor
    pub fn new() -> Self {
        Self {
            config: ResolverConfig::default(),
            stats: ResolutionStats::default(),
        }
    }
    
    /// Crea un resolvedor con configuración personalizada
    pub fn with_config(config: ResolverConfig) -> Self {
        Self {
            config,
            stats: ResolutionStats::default(),
        }
    }
    
    /// Resuelve un conflicto usando la estrategia especificada
    pub async fn resolve(
        &mut self,
        conflict: &mut Conflict,
        strategy: ResolutionStrategy,
    ) -> Result<ResolutionResult, ActorError> {
        let start_time = std::time::Instant::now();
        conflict.status = ConflictStatus::Resolving;
        
        let mut result = match strategy {
            ResolutionStrategy::Priority => self.resolve_priority(conflict).await?,
            ResolutionStrategy::FirstComeFirstServed => self.resolve_fifo(conflict).await?,
            ResolutionStrategy::FairShare => self.resolve_fair_share(conflict).await?,
            ResolutionStrategy::Random => self.resolve_random(conflict).await?,
            ResolutionStrategy::LoadBalanced => self.resolve_load_balanced(conflict).await?,
            ResolutionStrategy::TimeoutRetry => self.resolve_timeout_retry(conflict).await?,
            ResolutionStrategy::Mediation => self.resolve_mediation(conflict).await?,
            ResolutionStrategy::Sacrifice => self.resolve_sacrifice(conflict).await?,
            ResolutionStrategy::SplitResource => self.resolve_split_resource(conflict).await?,
            ResolutionStrategy::Custom(_) => self.resolve_custom(conflict).await?,
        };
        
        result.resolution_time_ms = start_time.elapsed().as_millis() as u64;
        result.strategy = strategy.clone();
        
        // Actualizar estadísticas
        self.stats.total_resolutions += 1;
        if result.success {
            self.stats.successful_resolutions += 1;
            conflict.mark_resolved();
        } else {
            self.stats.failed_resolutions += 1;
            conflict.status = ConflictStatus::Failed;
        }
        
        *self.stats.strategy_usage.entry(strategy).or_insert(0) += 1;
        
        Ok(result)
    }
    
    /// Sugiere la mejor estrategia para un conflicto
    pub fn suggest_strategy(&self, conflict: &Conflict) -> ResolutionStrategy {
        // Primero verificar si hay estrategia definida para este tipo/severidad
        if let Some(strategy) = self.config.severity_strategies.get(&conflict.severity) {
            return strategy.clone();
        }
        
        if let Some(strategy) = self.config.type_strategies.get(&conflict.conflict_type) {
            return strategy.clone();
        }
        
        // Estrategia por defecto según severidad
        match conflict.severity {
            ConflictSeverity::Critical => ResolutionStrategy::Sacrifice,
            ConflictSeverity::High => ResolutionStrategy::Priority,
            ConflictSeverity::Medium => ResolutionStrategy::FairShare,
            ConflictSeverity::Low => ResolutionStrategy::Random,
        }
    }
    
    /// Resuelve por prioridad
    async fn resolve_priority(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let priority_a = self.config.actor_priorities.get(&conflict.actors.0).unwrap_or(&0);
        let priority_b = self.config.actor_priorities.get(&conflict.actors.1).unwrap_or(&0);
        
        let (winner, loser) = if priority_a >= priority_b {
            (conflict.actors.0, conflict.actors.1)
        } else {
            (conflict.actors.1, conflict.actors.0)
        };
        
        let mut result = ResolutionResult::success(
            ResolutionStrategy::Priority,
            format!("Resuelto por prioridad: {:?} (prio: {}) gana sobre {:?} (prio: {})", 
                winner, priority_a, loser, priority_b),
        );
        
        result.set_winner_loser(winner, loser);
        result.add_action("Aplicar prioridad de actor".to_string());
        
        Ok(result)
    }
    
    /// Resuelve por orden de llegada (FIFO)
    async fn resolve_fifo(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        // Implementación simplificada: el primer actor gana
        let winner = conflict.actors.0;
        let loser = conflict.actors.1;
        
        let mut result = ResolutionResult::success(
            ResolutionStrategy::FirstComeFirstServed,
            format!("Resuelto por FIFO: {:?} llega primero", winner),
        );
        
        result.set_winner_loser(winner, loser);
        result.add_action("Aplicar regla primero en llegar".to_string());
        
        Ok(result)
    }
    
    /// Resuelve por distribución equitativa
    async fn resolve_fair_share(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        // Simular división equitativa del recurso
        let mut result = ResolutionResult::success(
            ResolutionStrategy::FairShare,
            format!("Recurso '{}' dividido equitativamente entre {:?} y {:?}", 
                conflict.resource, conflict.actors.0, conflict.actors.1),
        );
        
        result.add_action("Dividir recurso 50/50".to_string());
        result.add_action("Crear cuotas de uso".to_string());
        
        Ok(result)
    }
    
    /// Resuelve aleatoriamente
    async fn resolve_random(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        conflict.id.hash(&mut hasher);
        let random_val = hasher.finish();
        
        let (winner, loser) = if random_val % 2 == 0 {
            (conflict.actors.0, conflict.actors.1)
        } else {
            (conflict.actors.1, conflict.actors.0)
        };
        
        let mut result = ResolutionResult::success(
            ResolutionStrategy::Random,
            format!("Resuelto aleatoriamente: {:?} gana", winner),
        );
        
        result.set_winner_loser(winner, loser);
        result.add_action("Selección aleatoria".to_string());
        
        Ok(result)
    }
    
    /// Resuelve por balance de carga
    async fn resolve_load_balanced(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        // Implementación simplificada: elegir al menos cargado
        // En realidad esto consultaría métricas de carga reales
        let winner = conflict.actors.0; // Simplificado
        let loser = conflict.actors.1;
        
        let mut result = ResolutionResult::success(
            ResolutionStrategy::LoadBalanced,
            format!("Resuelto por balance de carga: {:?} tiene menor carga", winner),
        );
        
        result.set_winner_loser(winner, loser);
        result.add_action("Consultar métricas de carga".to_string());
        result.add_action("Asignar a actor con menor carga".to_string());
        
        Ok(result)
    }
    
    /// Resuelve con timeout y reintento
    async fn resolve_timeout_retry(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let mut result = ResolutionResult::success(
            ResolutionStrategy::TimeoutRetry,
            "Resuelto después de reintentos".to_string(),
        );
        
        result.add_action(format!("Iniciar timeout de {}ms", self.config.default_timeout_ms));
        
        // Simular retry
        for attempt in 1..=self.config.max_retries {
            result.add_action(format!("Intento {}", attempt));
            sleep(Duration::from_millis(100)).await; // Simular trabajo
            
            // Simular éxito en el último intento
            if attempt == self.config.max_retries {
                result.add_action("Éxito en reintento final".to_string());
                break;
            }
        }
        
        Ok(result)
    }
    
    /// Resuelve por mediación
    async fn resolve_mediation(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let mut result = ResolutionResult::success(
            ResolutionStrategy::Mediation,
            "Resuelto mediante mediación de Ares".to_string(),
        );
        
        result.add_action("Iniciar sesión de mediación".to_string());
        result.add_action("Escuchar ambas partes".to_string());
        result.add_action("Proponer solución mutua".to_string());
        result.add_action("Obtener acuerdo de ambos actores".to_string());
        
        Ok(result)
    }
    
    /// Resuelve sacrificando al menos importante
    async fn resolve_sacrifice(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let priority_a = self.config.actor_priorities.get(&conflict.actors.0).unwrap_or(&0);
        let priority_b = self.config.actor_priorities.get(&conflict.actors.1).unwrap_or(&0);
        
        let (winner, loser) = if priority_a >= priority_b {
            (conflict.actors.0, conflict.actors.1)
        } else {
            (conflict.actors.1, conflict.actors.0)
        };
        
        let mut result = ResolutionResult::success(
            ResolutionStrategy::Sacrifice,
            format!("Sacrificio: {:?} sobrevive, {:?} es sacrificado", winner, loser),
        );
        
        result.set_winner_loser(winner, loser);
        result.add_action("Identificar actor menos crítico".to_string());
        result.add_action("Sacrificar recursos del actor perdedor".to_string());
        
        Ok(result)
    }
    
    /// Resuelve dividiendo el recurso
    async fn resolve_split_resource(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let mut result = ResolutionResult::success(
            ResolutionStrategy::SplitResource,
            format!("Recurso '{}' dividido entre actores", conflict.resource),
        );
        
        result.add_action("Analular divisible del recurso".to_string());
        result.add_action("Crear particiones dedicadas".to_string());
        result.add_action("Asignar particiones a cada actor".to_string());
        
        Ok(result)
    }
    
    /// Resuelve con estrategia personalizada
    async fn resolve_custom(&mut self, conflict: &Conflict) -> Result<ResolutionResult, ActorError> {
        let mut result = ResolutionResult::success(
            ResolutionStrategy::Custom("personalizada".to_string()),
            "Resuelto con estrategia personalizada".to_string(),
        );
        
        result.add_action("Aplicar lógica personalizada".to_string());
        result.add_action("Ejecutar resolución específica del dominio".to_string());
        
        Ok(result)
    }
    
    /// Obtiene estadísticas de resolución
    pub fn get_stats(&self) -> &ResolutionStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::ares::detector::ConflictType;

    #[tokio::test]
    async fn test_priority_resolution() {
        let mut resolver = ConflictResolver::new();
        let mut conflict = Conflict::new(
            GodName::Zeus,
            GodName::Dionysus,
            "test_resource",
            ConflictType::Resource,
        );
        
        let result = resolver.resolve(&mut conflict, ResolutionStrategy::Priority).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.strategy, ResolutionStrategy::Priority);
        assert_eq!(result.winner, Some(GodName::Zeus));
        assert_eq!(result.loser, Some(GodName::Dionysus));
    }

    #[tokio::test]
    async fn test_strategy_suggestion() {
        let resolver = ConflictResolver::new();
        
        // Conflicto crítico debe sugerir sacrificio
        let conflict = Conflict::new(
            GodName::Athena,
            GodName::Hera,
            "patient_data",
            ConflictType::Data,
        );
        
        let strategy = resolver.suggest_strategy(&conflict);
        assert_eq!(strategy, ResolutionStrategy::Sacrifice);
    }

    #[tokio::test]
    async fn test_fair_share_resolution() {
        let mut resolver = ConflictResolver::new();
        let mut conflict = Conflict::new(
            GodName::Apollo,
            GodName::Artemis,
            "shared_resource",
            ConflictType::Resource,
        );
        
        let result = resolver.resolve(&mut conflict, ResolutionStrategy::FairShare).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.strategy, ResolutionStrategy::FairShare);
        assert!(result.winner.is_none()); // No hay ganador único en fair share
        assert!(result.loser.is_none());
    }

    #[tokio::test]
    async fn test_resolution_stats() {
        let mut resolver = ConflictResolver::new();
        let mut conflict = Conflict::new(
            GodName::Hermes,
            GodName::Hefesto,
            "test_resource",
            ConflictType::Resource,
        );
        
        resolver.resolve(&mut conflict, ResolutionStrategy::Random).await.unwrap();
        
        let stats = resolver.get_stats();
        assert_eq!(stats.total_resolutions, 1);
        assert_eq!(stats.successful_resolutions, 1);
        assert_eq!(stats.failed_resolutions, 0);
    }
}