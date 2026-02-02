/// 🏛️ Hestia - Diosa del Hogar, Fuego Sagrado y Configuración del Sistema
/// 🔥 Guardiana del Fuego del Olimpo y administradora de configuración central
/// ⚡ Gestiona configuración, parámetros del sistema y estado del hogar digital

use crate::actors::{OlympianGod, GodName, DivineDomain, OlympicResult, OlympianMessage};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 🔥 Configuración del hogar sagrado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HearthConfig {
    pub temperature: f64,
    pub warmth_level: u8,
    pub sanctuary_status: String,
    pub family_members: Vec<String>,
    pub ritual_schedule: HashMap<String, String>,
}

/// 🏛️ Parámetros del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParameters {
    pub max_connections: u32,
    pub session_timeout_minutes: u32,
    pub backup_frequency_hours: u32,
    pub log_retention_days: u32,
    pub performance_thresholds: HashMap<String, f64>,
}

/// 🔥 Estado del fuego sagrado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredFireStatus {
    pub is_burning: bool,
    pub intensity: f64,
    pub fuel_level: f64,
    pub last_rekindled: chrono::DateTime<chrono::Utc>,
    pub守护者: String,
}

/// 🏛️ Configuración completa de Hestia
#[derive(Debug, Clone)]
pub struct HestiaConfig {
    pub enable_auto_backup: bool,
    pub enable_system_monitoring: bool,
    pub sanctuary_maintenance_interval: u64,
    pub fire_rekindle_threshold: f64,
}

impl Default for HestiaConfig {
    fn default() -> Self {
        Self {
            enable_auto_backup: true,
            enable_system_monitoring: true,
            sanctuary_maintenance_interval: 3600, // 1 hora
            fire_rekindle_threshold: 0.2,
        }
    }
}

/// 🏛️ Hestia V12 - Diosa del Hogar y Configuración
pub struct HestiaV12 {
    name: GodName,
    domain: DivineDomain,
    config: HestiaConfig,
    hearth_config: RwLock<HearthConfig>,
    system_params: RwLock<SystemParameters>,
    sacred_fire: RwLock<SacredFireStatus>,
    configuration_history: RwLock<Vec<HashMap<String, serde_json::Value>>>,
}

impl HestiaV12 {
    /// 🏛️ Crear nueva instancia de Hestia
    pub fn new() -> Self {
        let initial_hearth = HearthConfig {
            temperature: 21.5,
            warmth_level: 7,
            sanctuary_status: "activo".to_string(),
            family_members: vec![
                "zeus".to_string(),
                "hera".to_string(),
                "athena".to_string(),
                "apollo".to_string(),
                "artemis".to_string(),
            ],
            ritual_schedule: HashMap::new(),
        };

        let initial_params = SystemParameters {
            max_connections: 1000,
            session_timeout_minutes: 30,
            backup_frequency_hours: 6,
            log_retention_days: 30,
            performance_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("cpu_usage".to_string(), 80.0);
                thresholds.insert("memory_usage".to_string(), 85.0);
                thresholds.insert("response_time_ms".to_string(), 500.0);
                thresholds
            },
        };

        let initial_fire = SacredFireStatus {
            is_burning: true,
            intensity: 1.0,
            fuel_level: 1.0,
            last_rekindled: chrono::Utc::now(),
            守护者: "Hestia".to_string(),
        };

        Self {
            name: GodName::Hestia,
            domain: DivineDomain::SystemConfig,
            config: HestiaConfig::default(),
            hearth_config: RwLock::new(initial_hearth),
            system_params: RwLock::new(initial_params),
            sacred_fire: RwLock::new(initial_fire),
            configuration_history: RwLock::new(Vec::new()),
        }
    }

    /// 🔥 Mantener el fuego sagrado
    pub async fn maintain_sacred_fire(&self) -> OlympicResult<SacredFireStatus> {
        let mut fire = self.sacred_fire.write().await;
        
        // Consumir燃料 lentamente
        fire.fuel_level = (fire.fuel_level - 0.01).max(0.0);
        
        // Reavivar si es necesario
        if fire.fuel_level < self.config.fire_rekindle_threshold {
            fire.fuel_level = 1.0;
            fire.intensity = 1.0;
            fire.last_rekindled = chrono::Utc::now();
            tracing::info!("🏛️ Hestia: Fuego sagrado reavivado");
        }
        
        // Ajustar intensidad según燃料
        fire.intensity = fire.fuel_level;
        fire.is_burning = fire.fuel_level > 0.0;
        
        Ok(fire.clone())
    }

    /// 🏛️ Actualizar configuración del hogar
    pub async fn update_hearth_config(&self, new_config: HearthConfig) -> OlympicResult<()> {
        let mut hearth = self.hearth_config.write().await;
        *hearth = new_config;
        
        // Guardar en historial
        let mut history = self.configuration_history.write().await;
        history.push(HashMap::from([
            ("timestamp".to_string(), serde_json::json!(chrono::Utc::now())),
            ("config_type".to_string(), serde_json::json!("hearth")),
            ("config".to_string(), serde_json::to_value(&*hearth).unwrap()),
        ]));
        
        tracing::info!("🏛️ Hestia: Configuración del hogar actualizada");
        Ok(())
    }

    /// ⚙️ Actualizar parámetros del sistema
    pub async fn update_system_parameters(&self, new_params: SystemParameters) -> OlympicResult<()> {
        let mut params = self.system_params.write().await;
        *params = new_params;
        
        // Guardar en historial
        let mut history = self.configuration_history.write().await;
        history.push(HashMap::from([
            ("timestamp".to_string(), serde_json::json!(chrono::Utc::now())),
            ("config_type".to_string(), serde_json::json!("system")),
            ("config".to_string(), serde_json::to_value(&*params).unwrap()),
        ]));
        
        tracing::info!("🏛️ Hestia: Parámetros del sistema actualizados");
        Ok(())
    }

    /// 🏛️ Realizar mantenimiento del santuario
    pub async fn perform_sanctuary_maintenance(&self) -> OlympicResult<HashMap<String, String>> {
        let mut results = HashMap::new();
        
        // Mantenimiento del fuego
        match self.maintain_sacred_fire().await {
            Ok(fire_status) => {
                results.insert("fire_maintenance".to_string(), 
                    format!("Fuego: {} (Intensidad: {:.2})", 
                        if fire_status.is_burning { "Activo" } else { "Inactivo" }, 
                        fire_status.intensity));
            }
            Err(e) => {
                results.insert("fire_maintenance".to_string(), format!("Error: {}", e));
            }
        }
        
        // Limpieza del hogar
        let hearth = self.hearth_config.read().await;
        results.insert("hearth_cleaning".to_string(), 
            format!("Hogar limpio, temperatura: {:.1}°C", hearth.temperature));
        
        // Verificación de parámetros
        let params = self.system_params.read().await;
        results.insert("parameter_check".to_string(), 
            format!("Parámetros verificados, {} configuraciones activas", 
                params.performance_thresholds.len()));
        
        tracing::info!("🏛️ Hestia: Mantenimiento del santuario completado");
        Ok(results)
    }

    /// 📊 Obtener estado completo del sistema
    pub async fn get_system_status(&self) -> OlympicResult<serde_json::Value> {
        let hearth = self.hearth_config.read().await;
        let params = self.system_params.read().await;
        let fire = self.sacred_fire.read().await;
        let history = self.configuration_history.read().await;
        
        Ok(serde_json::json!({
            "god": "Hestia",
            "domain": "SystemConfig",
            "hearth": hearth,
            "system_parameters": params,
            "sacred_fire": fire,
            "configurations_history_count": history.len(),
            "status": "Maintaining sanctuary"
        }))
    }

    /// 📈 Obtener historial de configuraciones
    pub async fn get_configuration_history(&self) -> OlympicResult<Vec<HashMap<String, serde_json::Value>>> {
        let history = self.configuration_history.read().await;
        Ok(history.clone())
    }

    /// 🏛️ Agregar miembro familiar
    pub async fn add_family_member(&self, member_name: &str) -> OlympicResult<()> {
        let mut hearth = self.hearth_config.write().await;
        if !hearth.family_members.contains(&member_name.to_string()) {
            hearth.family_members.push(member_name.to_string());
            tracing::info!("🏛️ Hestia: Nuevo miembro familiar agregado: {}", member_name);
        }
        Ok(())
    }

    /// 🗓️ Programar ritual
    pub async fn schedule_ritual(&self, ritual_name: &str, schedule: &str) -> OlympicResult<()> {
        let mut hearth = self.hearth_config.write().await;
        hearth.ritual_schedule.insert(ritual_name.to_string(), schedule.to_string());
        tracing::info!("🏛️ Hestia: Ritual '{}' programado para: {}", ritual_name, schedule);
        Ok(())
    }
}

#[async_trait]
impl OlympianGod for HestiaV12 {
    async fn process_message(&self, message: OlympianMessage) -> OlympicResult<OlympianMessage> {
        match message.command.as_str() {
            "maintain_fire" => {
                let fire_status = self.maintain_sacred_fire().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "fire_maintained".to_string(),
                    data: serde_json::to_value(fire_status)?,
                    metadata: HashMap::new(),
                })
            }
            "update_hearth" => {
                if let Some(config) = message.metadata.get("config") {
                    let hearth_config: HearthConfig = serde_json::from_value(config.clone())?;
                    self.update_hearth_config(hearth_config).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "hearth_updated".to_string(),
                        data: serde_json::json!({"status": "success"}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing hearth config".into())
                }
            }
            "update_system" => {
                if let Some(params) = message.metadata.get("parameters") {
                    let system_params: SystemParameters = serde_json::from_value(params.clone())?;
                    self.update_system_parameters(system_params).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "system_updated".to_string(),
                        data: serde_json::json!({"status": "success"}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing system parameters".into())
                }
            }
            "maintenance" => {
                let results = self.perform_sanctuary_maintenance().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "maintenance_completed".to_string(),
                    data: serde_json::to_value(results)?,
                    metadata: HashMap::new(),
                })
            }
            "get_status" => {
                let status = self.get_system_status().await?;
                Ok(OlympianMessage {
                    sender: self.name.clone(),
                    command: "status_ready".to_string(),
                    data: status,
                    metadata: HashMap::new(),
                })
            }
            "add_family_member" => {
                if let Some(member) = message.metadata.get("member").and_then(|m| m.as_str()) {
                    self.add_family_member(member).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "member_added".to_string(),
                        data: serde_json::json!({"member": member}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing member name".into())
                }
            }
            "schedule_ritual" => {
                if let (Some(ritual), Some(schedule)) = (
                    message.metadata.get("ritual").and_then(|r| r.as_str()),
                    message.metadata.get("schedule").and_then(|s| s.as_str())
                ) {
                    self.schedule_ritual(ritual, schedule).await?;
                    Ok(OlympianMessage {
                        sender: self.name.clone(),
                        command: "ritual_scheduled".to_string(),
                        data: serde_json::json!({"ritual": ritual, "schedule": schedule}),
                        metadata: HashMap::new(),
                    })
                } else {
                    Err("Missing ritual or schedule".into())
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
        self.get_system_status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sacred_fire_maintenance() {
        let hestia = HestiaV12::new();
        let result = hestia.maintain_sacred_fire().await.unwrap();
        assert!(result.is_burning);
        assert!(result.intensity > 0.0);
    }

    #[tokio::test]
    async fn test_add_family_member() {
        let hestia = HestiaV12::new();
        hestia.add_family_member("test_god").await.unwrap();
        let hearth = hestia.hearth_config.read().await;
        assert!(hearth.family_members.contains(&"test_god".to_string()));
    }
}