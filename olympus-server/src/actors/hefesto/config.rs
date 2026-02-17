// src/actors/hefesto/config.rs
// OLYMPUS v15 - Gestión de Configuraciones

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manager de configuraciones
#[derive(Debug, Clone)]
pub struct ConfigManager {
    configs: HashMap<String, ConfigEntry>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<ConfigEntry> {
        self.configs.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value, description: Option<String>) {
        let entry = ConfigEntry {
            key: key.to_string(),
            value,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
        };
        self.configs.insert(key.to_string(), entry);
    }

    pub fn delete(&mut self, key: &str) -> Option<ConfigEntry> {
        self.configs.remove(key)
    }

    pub fn list_all(&self) -> Vec<ConfigEntry> {
        self.configs.values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.configs.len()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub entries: Vec<ConfigEntry>,
    pub exported_at: DateTime<Utc>,
}
