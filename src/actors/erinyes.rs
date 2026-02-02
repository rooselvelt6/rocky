/// 🏹 Erinyes - Las Furias, Diosas de la Venganza y Justicia Retributiva
/// ⚡ Guardianas del orden moral y ejecutoras de justicia divina
/// 🔥 Gestiona penalizaciones, alertas de seguridad y acciones correctivas

use crate::actors::{OlympianGod, GodName, DivineDomain, OlympicResult, OlympianMessage};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// ⚡ Carga de crímenes y ofensas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeLoad {
    pub offender: String,
    pub offense_type: OffenseType,
    pub severity: u8,
    pub victim: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence: Vec<String>,
    pub divine_judgment: Option<JudgmentType>,
}

/// ⚔️ Tipos ofensas detectadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OffenseType {
    SecurityViolation,
    DataBreach,
    UnauthorizedAccess,
    PerformanceDegradation,
    ResourceAbuse,
    ProtocolInfraction,
    SacredBlasphemy,
}

/// 🔥 Tipos de juicio divino
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgmentType {
    Warning,
    TemporaryBan,
    ResourceLimitation,
    PerformanceThrottling,
    EternalBanishment,
    RitualPurification,
}

/// 🏹 Configuración de las Furias
#[derive(Debug, Clone)]
pub struct ErinyesConfig {
    pub enable_automatic_punishment: bool,
    pub max_warnings_before_ban: u8,
    pub punishment_severity_multiplier: f64,
    pub evidence_retention_days: u32,
    pub justice_speed_milliseconds: u64,
}

impl Default for ErinyesConfig {
    fn default() -> Self {
        Self {
            enable_automatic_punishment: true,
            max_warnings_before_ban: 3,
            punishment_severity_multiplier: 1.5,
            evidence_retention_days: 90,
            justice_speed_milliseconds: 100,
        }
    }
}

/// 📊 Estadísticas de justicia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JusticeStatistics {
    pub total_crimes_processed: u64,
    pub successful_punishments: u64,
    pub warnings_issued: u64,
    pub bans_executed: u64,
    pub average_judgment_time_ms: f64,
    pub most_common_offense: Option<OffenseType>,
}

/// 🏹 Erinyes V12 - Las Furias de la Justicia
pub struct ErinyesV12 {
    name: GodName,
    domain: DivineDomain,
    config: ErinyesConfig,
    crime_register: RwLock<Vec<CrimeLoad>>,
    active_punishments: RwLock<HashMap<String, JudgmentType>>,
    justice_statistics: RwLock<JusticeStatistics>,
    punishment_history: RwLock<Vec<HashMap<String, serde_json::Value>>>,
}

impl ErinyesV12 {
    /// 🏹 Crear nueva instancia de las Erinyes
    pub fn new() -> Self {
        Self {
            name: GodName::Erinyes,
            domain: DivineDomain::JusticeAndRetribution,
            config: ErinyesConfig::default(),
            crime_register: RwLock::new(Vec::new()),
            active_punishments: RwLock::new(HashMap::new()),
            justice_statistics: RwLock::new(JusticeStatistics {
                total_crimes_processed: 0,
                successful_punishments: 0,
                warnings_issued: 0,
                bans_executed: 0,
                average_judgment_time_ms: 0.0,
                most_common_offense: None,
            }),
            punishment_history: RwLock::new(Vec::new()),
        }
    }

    /// ⚡ Registrar nueva ofensa
    pub async fn register_crime(&self, crime: CrimeLoad) -> OlympicResult<String> {
        let crime_id = format!("crime_{}_{:?}", 
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), 
            crime.offense_type);
        
        // Registrar en el libro de crímenes
        let mut register = self.crime_register.write().await;
        register.push(crime.clone());
        
        // Actualizar estadísticas
        let mut stats = self.justice_statistics.write().await;
        stats.total_crimes_processed += 1;
        
        // Determinar juicio automáticamente si está habilitado
        if self.config.enable_automatic_punishment {
            drop(stats);
            drop(register);
            let judgment = self.deliver_judgment(&crime).await?;
            tracing::info!("🏹 Erinyes: Justicia ejecutada para {} - {:?}", crime.offender, judgment);
            return Ok(crime_id);
        }
        
        tracing::info!("🏹 Erinyes: Crimen registrado: {} cometido por {}", 
            format!("{:?}", crime.offense_type), crime.offender);
        Ok(crime_id)
    }

    /// 🔥 Entregar juicio divino
    pub async fn deliver_judgment(&self, crime: &CrimeLoad) -> OlympicResult<JudgmentType> {
        let start_time = std::time::Instant::now();
        
        // Determinar el tipo de juicio basado en severidad y historial
        let judgment = match crime.severity {
            1..=3 => JudgmentType::Warning,
            4..=6 => JudgmentType::ResourceLimitation,
            7..=8 => JudgmentType::TemporaryBan,
            9..=9 => JudgmentType::PerformanceThrottling,
            10 => JudgmentType::EternalBanishment,
            _ => JudgmentType::Warning,
        };
        
        // Verificar número de advertencias previas
        let punishments = self.active_punishments.read().await;
        let previous_warnings = punishments.values()
            .filter(|j| matches!(j, JudgmentType::Warning))
            .count();
        
        let final_judgment = if previous_warnings >= self.config.max_warnings_before_ban as usize {
            JudgmentType::EternalBanishment
        } else {
            judgment
        };
        
        // Ejecutar castigo
        self.execute_punishment(&crime.offender, &final_judgment).await?;
        
        // Actualizar estadísticas
        let mut stats = self.justice_statistics.write().await;
        stats.successful_punishments += 1;
        
        match final_judgment {
            JudgmentType::Warning => stats.warnings_issued += 1,
            JudgmentType::EternalBanishment | JudgmentType::TemporaryBan => stats.bans_executed += 1,
            _ => {}
        }
        
        let elapsed = start_time.elapsed().as_millis() as f64;
        stats.average_judgment_time_ms = (stats.average_judgment_time_ms * (stats.successful_punishments - 1) as f64 + elapsed) 
            / stats.successful_punishments as f64;
        
        // Registrar en historial
        let mut history = self.punishment_history.write().await;
        history.push(HashMap::from([
            ("timestamp".to_string(), serde_json::json!(chrono::Utc::now())),
            ("offender".to_string(), serde_json::json!(crime.offender)),
            ("offense".to_string(), serde_json::json!(crime.offense_type)),
            ("judgment".to_string(), serde_json::json!(final_judgment)),
            ("judgment_time_ms".to_string(), serde_json::json!(elapsed)),
        ]));
        
        Ok(final_judgment)
    }

    /// ⚔️ Ejecutar castigo
    pub async fn execute_punishment(&self, offender: &str, judgment: &JudgmentType) -> OlympicResult<()> {
        let mut punishments = self.active_punishments.write().await;
        punishments.insert(offender.to_string(), judgment.clone());
        
        match judgment {
            JudgmentType::Warning => {
                tracing::warn!("🏹 Erinyes: ADVERTENCIA emitida para {}", offender);
            }
            JudgmentType::TemporaryBan => {
                tracing::error!("🏹 Erinyes: BAN TEMPORAL ejecutado para {}", offender);
            }
            JudgmentType::EternalBanishment => {
                tracing::error!("🏹 Erinyes: BAN ETERNO ejecutado para {}", offender);
            }
            JudgmentType::ResourceLimitation => {
                tracing::warn!("🏹 Erinyes: Recursos limitados para {}", offender);
            }
            JudgmentType::PerformanceThrottling => {
                tracing::warn!("🏹 Erinyes: Rendimiento reducido para {}", offender);
            }
            JudgmentType::RitualPurification => {
                tracing::info!("🏹 Erinyes: Ritual de purificación iniciado para {}", offender);
            }
        }
        
        Ok(())
    }

    /// 🔍 Investigar patrón criminal
    pub async fn investigate_criminal_pattern(&self, offender: &str) -> OlympicResult<HashMap<String, serde_json::Value>> {
        let register = self.crime_register.read().await;
        let punishments = self.active_punishments.read().await;
        
        let offender_crimes: Vec<&CrimeLoad> = register.iter()
            .filter(|c| c.offender == offender)
            .collect();
        
        let pattern_analysis = HashMap::from([
            ("offender".to_string(), serde_json::json!(offender)),
            ("total_crimes".to_string(), serde_json::json!(offender_crimes.len())),
            ("most_common_offense".to_string(), {
                let mut offense_counts = HashMap::new();
                for crime in &offender_crimes {
                    *offense_counts.entry(format!("{:?}", crime.offense_type)).or_insert(0) += 1;
                }
                let most_common = offense_counts.iter().max_by_key(|(_, &count)| count);
                serde_json::json!(most_common.map(|(offense, _)| offense))
            }),
            ("current_punishment".to_string(), serde_json::json!(punishments.get(offender))),
            ("average_severity".to_string(), serde_json::json!(
                if offender_crimes.is_empty() { 0.0 } else { 
                    offender_crimes.iter().map(|c| c.severity as f64).sum::<f64>() / offender_crimes.len() as f64 
                }
            )),
        ]);
        
        Ok(pattern_analysis)
    }

    /// 📊 Obtener estadísticas de justicia
    pub async fn get_justice_statistics(&self) -> OlympicResult<JusticeStatistics> {
        let stats = self.justice_statistics.read().await;
        Ok(stats.clone())
    }

    /// 🔥 Realizar escaneo de ofensas
    pub async fn scan_for_offenses(&self) -> OlympicResult<Vec<CrimeLoad>> {
        let mut detected_crimes = Vec::new();
        
        // Simular detección de ofensas en el sistema
        let offenses = vec![
            CrimeLoad {
                offender: "system_anomaly".to_string(),
                offense_type: OffenseType::SecurityViolation,
                severity: 3,
                victim: Some("system_integrity".to_string()),
                timestamp: chrono::Utc::now(),
                evidence: vec!["unusual_access_pattern".to_string()],
                divine_judgment: None,
            },
            CrimeLoad {
                offender: "performance_degrader".to_string(),
                offense_type: OffenseType::PerformanceDegradation,
                severity: 4,
                victim: Some("user_experience".to_string()),
                timestamp: chrono::Utc::now(),
                evidence: vec!["high_response_time".to_string(), "resource_exhaustion".to_string()],
                divine_judgment: None,
            },
        ];
        
        for offense in offenses {
            detected_crimes.push(offense.clone());
            self.register_crime(offense).await?;
        }
        
        tracing::info!("🏹 Erinyes: Escaneo completado, {} ofensas detectadas", detected_crimes.len());
        Ok(detected_crimes)
    }

    /// ⚖️ Absolver transgresor
    pub async fn absolve_transgressor(&self, offender: &str) -> OlympicResult<()> {
        let mut punishments = self.active_punishments.write().await;
        if punishments.remove(offender).is_some() {
            tracing::info!("🏹 Erinyes: {} ha sido absuelto, castigos levantados", offender);
            
            // Registrar absolución
            let mut history = self.punishment_history.write().await;
            history.push(HashMap::from([
                ("timestamp".to_string(), serde_json::json!(chrono::Utc::now())),
                ("offender".to_string(), serde_json::json!(offender)),
                ("action".to_string(), serde_json::json!("absolution")),
            ]));
        }
        Ok(())
    }
}

#[async_trait]
impl OlympianGod for ErinyesV12 {
    async fn process_message(&self, message: OlympianMessage) -> OlympicResult<OlympianMessage> {
        match message.command.as_str() {
            "register_crime" => {
                if let Some(crime_data) = message.metadata.get("crime") {
                    let crime: CrimeLoad = serde_json::from_value(crime_data.clone())?;
                    let crime_id = self.register_crime(crime).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "crime_registered".to_string(),
                        data: serde_json::json!({"crime_id": crime_id}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing crime data".into())
                }
            }
            "investigate" => {
                if let Some(offender) = message.metadata.get("offender").and_then(|o| o.as_str()) {
                    let pattern = self.investigate_criminal_pattern(offender).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "investigation_complete".to_string(),
                        data: serde_json::to_value(pattern)?,
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing offender name".into())
                }
            }
            "scan_offenses" => {
                let crimes = self.scan_for_offenses().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "scan_complete".to_string(),
                    data: serde_json::to_value(crimes)?,
                    metadata: HashMap::new(),
                })
            }
            "get_statistics" => {
                let stats = self.get_justice_statistics().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "statistics_ready".to_string(),
                    data: serde_json::to_value(stats)?,
                    metadata: HashMap::new(),
                })
            }
            "absolve" => {
                if let Some(offender) = message.metadata.get("offender").and_then(|o| o.as_str()) {
                    self.absolve_transgressor(offender).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "absolved".to_string(),
                        data: serde_json::json!({"offender": offender}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing offender name".into())
                }
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
        let stats = self.get_justice_statistics().await?;
        let active_punishments = self.active_punishments.read().await;
        let crime_count = self.crime_register.read().await.len();
        
        Ok(serde_json::json!({
            "god": "Erinyes",
            "domain": "JusticeAndRetribution",
            "justice_statistics": stats,
            "active_punishments_count": active_punishments.len(),
            "total_crimes_registered": crime_count,
            "status": "Vigilant and Just"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crime_registration() {
        let erinyes = ErinyesV12::new();
        let crime = CrimeLoad {
            offender: "test_offender".to_string(),
            offense_type: OffenseType::SecurityViolation,
            severity: 5,
            victim: Some("victim".to_string()),
            timestamp: chrono::Utc::now(),
            evidence: vec!["test_evidence".to_string()],
            divine_judgment: None,
        };
        
        let result = erinyes.register_crime(crime).await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_punishment_execution() {
        let erinyes = ErinyesV12::new();
        let result = erinyes.execute_punishment("test", &JudgmentType::Warning).await.unwrap();
        assert_eq!(result, ());
    }
}