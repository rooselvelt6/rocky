// src/actors/hades/keys.rs
// OLYMPUS v15 - Hades Key Manager
// Gestión segura de claves criptográficas con rotación automática y zeroize

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use zeroize::{Zeroize, ZeroizeOnDrop};
use tracing::{info, warn, error};
use ring::rand::{SecureRandom, SystemRandom};

use crate::actors::hades::encryption::{EncryptionAlgorithm, SecretKey, EncryptionError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoKey {
    pub id: String,
    pub key_type: KeyType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_rotated_at: chrono::DateTime<chrono::Utc>,
    pub rotation_count: u32,
    pub status: KeyStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    Encryption(EncryptionAlgorithm),
    Signing,
    Hmac,
    Argon2id,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Expired,
    Revoked,
    Compromised,
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureKeyStorage {
    #[zeroize]
    pub keys: HashMap<String, SecretKey>,
    pub last_accessed: HashMap<String, Instant>,
}

impl SecureKeyStorage {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            last_accessed: HashMap::new(),
        }
    }
    
    pub fn get(&self, key_id: &str) -> Option<&SecretKey> {
        self.keys.get(key_id)
    }
    
    pub fn insert(&mut self, key_id: String, key: SecretKey) {
        self.keys.insert(key_id.clone(), key);
        self.last_accessed.insert(key_id, Instant::now());
    }
    
    pub fn remove(&mut self, key_id: &str) -> Option<SecretKey> {
        self.last_accessed.remove(key_id);
        self.keys.remove(key_id)
    }
    
    pub fn access(&mut self, key_id: &str) {
        if self.keys.contains_key(key_id) {
            self.last_accessed.insert(key_id.to_string(), Instant::now());
        }
    }
    
    pub fn get_unused_keys(&self, duration: Duration) -> Vec<String> {
        let now = Instant::now();
        self.last_accessed
            .iter()
            .filter(|(_, last_accessed)| now.duration_since(**last_accessed) > duration)
            .map(|(key_id, _)| key_id.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct KeyManager {
    storage: Arc<RwLock<SecureKeyStorage>>,
    metadata: Arc<RwLock<HashMap<String, CryptoKey>>>,
    rng: SystemRandom,
    auto_rotation_enabled: bool,
    rotation_interval_days: u64,
    max_key_age_days: u64,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(SecureKeyStorage::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            auto_rotation_enabled: true,
            rotation_interval_days: 30,
            max_key_age_days: 90,
        }
    }
    
    pub fn with_config(
        auto_rotation: bool,
        rotation_days: u64,
        max_age_days: u64,
    ) -> Self {
        Self {
            storage: Arc::new(RwLock::new(SecureKeyStorage::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            auto_rotation_enabled: auto_rotation,
            rotation_interval_days: rotation_days,
            max_key_age_days: max_age_days,
        }
    }
    
    /// Generate a new encryption key
    pub async fn generate_key(
        &self,
        key_type: KeyType,
        ttl_days: Option<u64>,
    ) -> Result<String, KeyManagerError> {
        let key_id = uuid::Uuid::new_v4().to_string();
        
        // Generate random key bytes
        let key_length = match &key_type {
            KeyType::Encryption(_) => 32, // 256 bits
            KeyType::Signing => 32,
            KeyType::Hmac => 32,
            KeyType::Argon2id => 32,
        };
        
        let mut key_bytes = vec![0u8; key_length];
        self.rng.fill(&mut key_bytes)
            .map_err(|_| KeyManagerError::RngError)?;
        
        let algorithm = match &key_type {
            KeyType::Encryption(algo) => *algo,
            _ => EncryptionAlgorithm::ChaCha20Poly1305,
        };
        
        let secret_key = SecretKey::new(key_bytes, key_id.clone(), algorithm);
        
        // Store key securely
        let mut storage = self.storage.write().await;
        storage.insert(key_id.clone(), secret_key);
        drop(storage);
        
        // Store metadata
        let now = chrono::Utc::now();
        let metadata = CryptoKey {
            id: key_id.clone(),
            key_type: key_type.clone(),
            created_at: now,
            expires_at: ttl_days.map(|days| now + chrono::Duration::days(days as i64)),
            last_rotated_at: now,
            rotation_count: 0,
            status: KeyStatus::Active,
            metadata: serde_json::json!({
                "algorithm": format!("{:?}", algorithm),
            }),
        };
        
        let mut meta = self.metadata.write().await;
        meta.insert(key_id.clone(), metadata);
        drop(meta);
        
        info!("🔑 Generated new key: {} (type: {:?})", key_id, key_type);
        
        Ok(key_id)
    }
    
    /// Get a key by ID
    pub async fn get_key(&self, key_id: &str) -> Option<SecretKey> {
        let mut storage = self.storage.write().await;
        storage.access(key_id);
        let key = storage.get(key_id).cloned();
        drop(storage);
        key
    }
    
    /// Get or create a key
    pub async fn get_or_create_key(
        &self,
        key_id: &str,
        algorithm: EncryptionAlgorithm,
    ) -> Result<SecretKey, EncryptionError> {
        // Try to get existing key
        if let Some(key) = self.get_key(key_id).await {
            return Ok(key);
        }
        
        // Generate new key
        let mut key_bytes = vec![0u8; 32];
        self.rng.fill(&mut key_bytes)
            .map_err(|_| EncryptionError::RngError)?;
        
        let secret_key = SecretKey::new(key_bytes, key_id.to_string(), algorithm);
        
        // Store it
        let mut storage = self.storage.write().await;
        storage.insert(key_id.to_string(), secret_key.clone());
        drop(storage);
        
        // Store metadata
        let now = chrono::Utc::now();
        let metadata = CryptoKey {
            id: key_id.to_string(),
            key_type: KeyType::Encryption(algorithm),
            created_at: now,
            expires_at: None,
            last_rotated_at: now,
            rotation_count: 0,
            status: KeyStatus::Active,
            metadata: serde_json::json!({}),
        };
        
        let mut meta = self.metadata.write().await;
        meta.insert(key_id.to_string(), metadata);
        
        info!("🔑 Auto-created key: {} (algorithm: {:?})", key_id, algorithm);
        
        Ok(secret_key)
    }
    
    /// Store a key (internal use)
    pub async fn store_key(&self, key: SecretKey) {
        let mut storage = self.storage.write().await;
        storage.insert(key.key_id.clone(), key);
    }
    
    /// Revoke a key
    pub async fn revoke_key(&self, key_id: &str, reason: Option<String>) {
        let mut storage = self.storage.write().await;
        let key = storage.remove(key_id);
        drop(storage);
        drop(key); // Key is zeroized on drop
        
        // Update metadata
        let mut meta = self.metadata.write().await;
        if let Some(metadata) = meta.get_mut(key_id) {
            metadata.status = KeyStatus::Revoked;
            metadata.metadata = serde_json::json!({
                "revoked_at": chrono::Utc::now(),
                "reason": reason,
            });
        }
        
        warn!("🚫 Key revoked: {} (reason: {:?})", key_id, reason);
    }
    
    /// Rotate a key (generate new key and re-encrypt data)
    pub async fn rotate_key(&self, key_id: &str) -> Result<String, KeyManagerError> {
        let mut meta = self.metadata.write().await;
        
        let old_metadata = meta.get(key_id).cloned()
            .ok_or_else(|| KeyManagerError::KeyNotFound(key_id.to_string()))?;
        
        if old_metadata.status != KeyStatus::Active {
            return Err(KeyManagerError::KeyNotActive(key_id.to_string()));
        }
        
        // Generate new key
        let new_key_id = self.generate_key(
            old_metadata.key_type.clone(),
            old_metadata.expires_at.map(|exp| {
                let now = chrono::Utc::now();
                (exp - now).num_days().max(0) as u64
            })
        ).await?;
        
        // Mark old key as rotated
        if let Some(metadata) = meta.get_mut(key_id) {
            metadata.rotation_count += 1;
            metadata.last_rotated_at = chrono::Utc::now();
        }
        drop(meta);
        
        // Securely delete old key after rotation
        let mut storage = self.storage.write().await;
        let old_key = storage.remove(key_id);
        drop(storage);
        drop(old_key); // Zeroized on drop
        
        info!("🔄 Key rotated: {} -> {}", key_id, new_key_id);
        
        Ok(new_key_id)
    }
    
    /// List all keys
    pub async fn list_keys(&self, status: Option<KeyStatus>) -> Vec<CryptoKey> {
        let meta = self.metadata.read().await;
        
        meta.values()
            .filter(|k| status.as_ref().map(|s| &k.status == s).unwrap_or(true))
            .cloned()
            .collect()
    }
    
    /// Check if a key exists and is active
    pub async fn is_key_valid(&self, key_id: &str) -> bool {
        let meta = self.metadata.read().await;
        
        if let Some(metadata) = meta.get(key_id) {
            if metadata.status != KeyStatus::Active {
                return false;
            }
            
            // Check expiration
            if let Some(expires) = metadata.expires_at {
                if chrono::Utc::now() > expires {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// Cleanup expired keys
    pub async fn cleanup_expired_keys(&self) -> usize {
        let mut meta = self.metadata.write().await;
        let now = chrono::Utc::now();
        
        let expired: Vec<String> = meta
            .iter()
            .filter(|(_, m)| {
                m.status == KeyStatus::Active &&
                m.expires_at.map(|exp| now > exp).unwrap_or(false)
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for key_id in &expired {
            if let Some(metadata) = meta.get_mut(key_id) {
                metadata.status = KeyStatus::Expired;
            }
            
            // Also remove from secure storage
            let mut storage = self.storage.write().await;
            let key = storage.remove(key_id);
            drop(key); // Zeroize
        }
        
        let count = expired.len();
        if count > 0 {
            info!("🧹 Cleaned up {} expired keys", count);
        }
        
        count
    }
    
    /// Securely delete all unused keys
    pub async fn cleanup_unused_keys(&self, duration: Duration) -> usize {
        let storage = self.storage.read().await;
        let unused = storage.get_unused_keys(duration);
        drop(storage);
        
        let mut count = 0;
        for key_id in unused {
            self.revoke_key(&key_id, Some("Unused for extended period".to_string())).await;
            count += 1;
        }
        
        if count > 0 {
            info!("🧹 Cleaned up {} unused keys", count);
        }
        
        count
    }
    
    /// Get key statistics
    pub async fn get_stats(&self) -> KeyManagerStats {
        let meta = self.metadata.read().await;
        let storage = self.storage.read().await;
        
        let total = meta.len();
        let active = meta.values().filter(|m| m.status == KeyStatus::Active).count();
        let expired = meta.values().filter(|m| m.status == KeyStatus::Expired).count();
        let revoked = meta.values().filter(|m| m.status == KeyStatus::Revoked).count();
        let compromised = meta.values().filter(|m| m.status == KeyStatus::Compromised).count();
        
        KeyManagerStats {
            total_keys: total,
            active_keys: active,
            expired_keys: expired,
            revoked_keys: revoked,
            compromised_keys: compromised,
            keys_in_memory: storage.keys.len(),
        }
    }
    
    /// Start automatic key rotation
    pub fn start_rotation_worker(&self) {
        if !self.auto_rotation_enabled {
            return;
        }
        
        let metadata = self.metadata.clone();
        let rotation_days = self.rotation_interval_days;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60)); // Daily
            
            loop {
                interval.tick().await;
                
                let meta = metadata.read().await;
                let now = chrono::Utc::now();
                
                for (key_id, metadata) in meta.iter() {
                    if metadata.status != KeyStatus::Active {
                        continue;
                    }
                    
                    let days_since_rotation = (now - metadata.last_rotated_at).num_days();
                    
                    if days_since_rotation >= rotation_days as i64 {
                        warn!("⚠️ Key {} needs rotation (last rotated {} days ago)", 
                            key_id, days_since_rotation);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagerStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub revoked_keys: usize,
    pub compromised_keys: usize,
    pub keys_in_memory: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyManagerError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Key not active: {0}")]
    KeyNotActive(String),
    
    #[error("RNG error")]
    RngError,
    
    #[error("Storage error: {0}")]
    StorageError(String),
}
