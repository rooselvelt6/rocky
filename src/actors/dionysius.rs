/// 🍷️ Dionisio - Dios del Vino, Fiestas y Análisis de Datos
/// 🔥 Especializado en procesamiento de datos estadísticos y análisis predictivo
/// ⚡ Gestiona análisis complejos y transformaciones de datos con estilo divino

use crate::actors::{OlympianGod, GodName, DivineDomain, OlympicResult, OlympianMessage};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 🍷️ Datos del vino y análisis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineAnalysis {
    pub vintage: String,
    pub quality_score: f64,
    pub region: String,
    pub grape_type: String,
    pub analysis_metrics: HashMap<String, f64>,
}

/// 🎊 Datos de fiesta y celebración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelebrationMetrics {
    pub participants: usize,
    pub intensity_level: f64,
    pub duration_minutes: usize,
    pub satisfaction_score: f64,
}

/// 📊 Configuración de análisis dionisíaco
#[derive(Debug, Clone)]
pub struct DionysusConfig {
    pub enable_advanced_statistics: bool,
    pub enable_predictive_models: bool,
    pub wine_analysis_depth: usize,
    pub celebration_tracking: bool,
}

impl Default for DionysusConfig {
    fn default() -> Self {
        Self {
            enable_advanced_statistics: true,
            enable_predictive_models: true,
            wine_analysis_depth: 5,
            celebration_tracking: true,
        }
    }
}

/// 🍷️ Dionisio V12 - Dios del Vino y Análisis
pub struct DionysusV12 {
    name: GodName,
    domain: DivineDomain,
    config: DionysusConfig,
    wine_registry: RwLock<HashMap<String, WineAnalysis>>,
    celebration_history: RwLock<Vec<CelebrationMetrics>>,
}

impl DionysusV12 {
    /// 🍷️ Crear nueva instancia de Dionisio
    pub fn new() -> Self {
        Self {
            name: GodName::Dionysus,
            domain: DivineDomain::DataAnalysis,
            config: DionysusConfig::default(),
            wine_registry: RwLock::new(HashMap::new()),
            celebration_history: RwLock::new(Vec::new()),
        }
    }

    /// 🍷️ Analizar calidad del vino
    pub async fn analyze_wine_quality(&self, vintage: &str, quality_score: f64) -> OlympicResult<WineAnalysis> {
        let analysis = WineAnalysis {
            vintage: vintage.to_string(),
            quality_score,
            region: "Olympus Vineyards".to_string(),
            grape_type: "Divine Grapes".to_string(),
            analysis_metrics: self.calculate_wine_metrics(quality_score).await,
        };

        // Registrar en el sistema
        self.wine_registry.write().await.insert(vintage.to_string(), analysis.clone());

        Ok(analysis)
    }

    /// 🎊 Organizar celebración
    pub async fn organize_celebration(&self, participants: usize, intensity: f64) -> OlympicResult<CelebrationMetrics> {
        let celebration = CelebrationMetrics {
            participants,
            intensity_level: intensity,
            duration_minutes: (participants as f64 * intensity * 10.0) as usize,
            satisfaction_score: self.calculate_satisfaction(participants, intensity).await,
        };

        // Registrar en historial
        self.celebration_history.write().await.push(celebration.clone());

        Ok(celebration)
    }

    /// 📊 Realizar análisis predictivo
    pub async fn predictive_analysis(&self, data_points: &[f64]) -> OlympicResult<Vec<f64>> {
        if !self.config.enable_predictive_models {
            return Ok(vec![]);
        }

        let predictions = self.calculate_predictions(data_points).await;
        Ok(predictions)
    }

    /// 🍷️ Calcular métricas del vino
    async fn calculate_wine_metrics(&self, quality_score: f64) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        metrics.insert("divinity_score".to_string(), quality_score * 1.2);
        metrics.insert("maturity_level".to_string(), quality_score * 0.8);
        metrics.insert("complexity".to_string(), quality_score * 1.5);
        metrics.insert("balance".to_string(), quality_score * 0.9);
        
        if self.config.enable_advanced_statistics {
            metrics.insert("aromatic_intensity".to_string(), quality_score * 1.1);
            metrics.insert("finish_length".to_string(), quality_score * 0.7);
        }
        
        metrics
    }

    /// 🎊 Calcular satisfacción de celebración
    async fn calculate_satisfaction(&self, participants: usize, intensity: f64) -> f64 {
        let base_satisfaction = (participants as f64).sqrt() * intensity * 0.1;
        let bonus = if intensity > 0.8 { 0.2 } else { 0.0 };
        (base_satisfaction + bonus).min(1.0)
    }

    /// 📊 Calcular predicciones
    async fn calculate_predictions(&self, data_points: &[f64]) -> Vec<f64> {
        if data_points.len() < 2 {
            return vec![];
        }

        // Simple trend analysis with divine intuition
        let last_two = &data_points[data_points.len()-2..];
        let trend = last_two[1] - last_two[0];
        
        (1..=5).map(|i| {
            last_two[1] + (trend * i as f64) * 0.8 // Divine accuracy adjustment
        }).collect()
    }

    /// 🍷️ Obtener mejor vino del registro
    pub async fn get_best_wine(&self) -> OlympicResult<Option<WineAnalysis>> {
        let registry = self.wine_registry.read().await;
        let best = registry.values()
            .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
            .cloned();
        Ok(best)
    }

    /// 📊 Obtener estadísticas de celebraciones
    pub async fn get_celebration_stats(&self) -> OlympicResult<HashMap<String, f64>> {
        let history = self.celebration_history.read().await;
        if history.is_empty() {
            return Ok(HashMap::new());
        }

        let total_participants: usize = history.iter().map(|c| c.participants).sum();
        let avg_satisfaction: f64 = history.iter().map(|c| c.satisfaction_score).sum::<f64>() / history.len() as f64;
        let total_intensity: f64 = history.iter().map(|c| c.intensity_level).sum();

        let mut stats = HashMap::new();
        stats.insert("total_celebrations".to_string(), history.len() as f64);
        stats.insert("total_participants".to_string(), total_participants as f64);
        stats.insert("average_satisfaction".to_string(), avg_satisfaction);
        stats.insert("total_intensity".to_string(), total_intensity);

        Ok(stats)
    }
}

#[async_trait]
impl OlympianGod for DionysusV12 {
    async fn process_message(&self, message: OlympianMessage) -> OlympicResult<OlympianMessage> {
        match message.command.as_str() {
            "analyze_wine" => {
                if let (Some(vintage), Some(quality)) = (
                    message.metadata.get("vintage").and_then(|v| v.as_str()),
                    message.metadata.get("quality").and_then(|q| q.as_f64())
                ) {
                    let analysis = self.analyze_wine_quality(vintage, quality).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "wine_analysis_complete".to_string(),
                        data: serde_json::to_value(analysis)?,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing vintage or quality parameters".into())
                }
            }
            "organize_celebration" => {
                if let (Some(participants), Some(intensity)) = (
                    message.metadata.get("participants").and_then(|p| p.as_u64()),
                    message.metadata.get("intensity").and_then(|i| i.as_f64())
                ) {
                    let celebration = self.organize_celebration(participants as usize, intensity).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "celebration_organized".to_string(),
                        data: serde_json::to_value(celebration)?,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing participants or intensity parameters".into())
                }
            }
            "predictive_analysis" => {
                if let Some(data) = message.metadata.get("data_points") {
                    let data_points: Vec<f64> = serde_json::from_value(data.clone())?;
                    let predictions = self.predictive_analysis(&data_points).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "predictions_ready".to_string(),
                        data: serde_json::to_value(predictions)?,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing data_points parameter".into())
                }
            }
            "get_stats" => {
                let stats = self.get_celebration_stats().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "stats_ready".to_string(),
                    data: serde_json::to_value(stats)?,
                    metadata: HashMap::new(),
                })
            }
            _ => Err(format!("Unknown command: {}", message.command).into()),
        }
    }

    fn get_name(&self) -> GodName {
        self.name.clone()
    }

    fn get_domain(&self) -> DivineDomain {
        self.domain.clone()
    }

    async fn get_status(&self) -> OlympicResult<serde_json::Value> {
        let wine_count = self.wine_registry.read().await.len();
        let celebration_count = self.celebration_history.read().await.len();
        
        Ok(serde_json::json!({
            "god": "Dionysus",
            "domain": "DataAnalysis",
            "wines_registered": wine_count,
            "celebrations_hosted": celebration_count,
            "predictive_models_enabled": self.config.enable_predictive_models,
            "status": "Celebrating and Analyzing"
        }))
    }
}

/// 🍷️ Tipo alias para compatibilidad
pub type DionysiusV12 = DionysusV12;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wine_analysis() {
        let dionysus = DionysusV12::new();
        let result = dionysus.analyze_wine_quality("2024", 0.85).await.unwrap();
        assert_eq!(result.vintage, "2024");
        assert_eq!(result.quality_score, 0.85);
    }

    #[tokio::test]
    async fn test_celebration_organization() {
        let dionysus = DionysusV12::new();
        let result = dionysus.organize_celebration(50, 0.9).await.unwrap();
        assert_eq!(result.participants, 50);
        assert_eq!(result.intensity_level, 0.9);
    }
}