// src/actors/hades/encryption.rs
// OLYMPUS v15 - Hades Encryption Service
// Cifrado real con AES-256-GCM y ChaCha20-Poly1305 usando ring y chacha20poly1305

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use ring::aead::{Nonce, UnboundKey, AES_256_GCM, NonceSequence, OpeningKey, SealingKey, BoundKey, Aad};
use ring::rand::{SecureRandom, SystemRandom};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce as ChaChaNonce};
use chacha20poly1305::aead::{Aead, KeyInit};
use zeroize::{Zeroize, ZeroizeOnDrop};
use tracing::{info, warn};

use crate::actors::hades::keys::KeyManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
    pub algorithm: EncryptionAlgorithm,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Zeroize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        EncryptionAlgorithm::ChaCha20Poly1305 // Default to ChaCha20 for better performance
    }
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    #[zeroize]
    pub key_bytes: Vec<u8>,
    pub key_id: String,
    pub algorithm: EncryptionAlgorithm,
}

impl SecretKey {
    pub fn new(key_bytes: Vec<u8>, key_id: String, algorithm: EncryptionAlgorithm) -> Self {
        Self {
            key_bytes,
            key_id,
            algorithm,
        }
    }
    
    pub fn len(&self) -> usize {
        self.key_bytes.len()
    }
}

#[derive(Debug)]
pub struct EncryptionService {
    key_manager: Arc<RwLock<KeyManager>>,
    default_algorithm: EncryptionAlgorithm,
    #[allow(dead_code)]
    rng: SystemRandom,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            key_manager: Arc::new(RwLock::new(KeyManager::new())),
            default_algorithm: EncryptionAlgorithm::default(),
            rng: SystemRandom::new(),
        }
    }
    
    pub fn with_algorithm(algorithm: EncryptionAlgorithm) -> Self {
        Self {
            key_manager: Arc::new(RwLock::new(KeyManager::new())),
            default_algorithm: algorithm,
            rng: SystemRandom::new(),
        }
    }
    
    /// Encrypt data using the specified algorithm
    pub async fn encrypt(
        &self,
        data: &[u8],
        key_id: Option<&str>,
        algorithm: Option<EncryptionAlgorithm>,
    ) -> Result<EncryptedData, EncryptionError> {
        let algo = algorithm.unwrap_or(self.default_algorithm);
        
        // Get or generate key
        let key_manager = self.key_manager.read().await;
        let key_id = key_id.map(|s| s.to_string()).unwrap_or_else(|| {
            uuid::Uuid::new_v4().to_string()
        });
        
        let key = key_manager.get_or_create_key(&key_id, algo).await?;
        drop(key_manager);
        
        match algo {
            EncryptionAlgorithm::Aes256Gcm => {
                self.encrypt_aes256_gcm(data, &key, &key_id).await
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.encrypt_chacha20_poly1305(data, &key, &key_id).await
            }
        }
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, data: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        // Get key
        let key_manager = self.key_manager.read().await;
        let key = key_manager.get_key(&data.key_id).await
            .ok_or_else(|| EncryptionError::KeyNotFound(data.key_id.clone()))?;
        drop(key_manager);
        
        match data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.decrypt_aes256_gcm(data, &key).await
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.decrypt_chacha20_poly1305(data, &key).await
            }
        }
    }
    
    /// Encrypt using AES-256-GCM via ring
    async fn encrypt_aes256_gcm(
        &self,
        data: &[u8],
        key: &SecretKey,
        key_id: &str,
    ) -> Result<EncryptedData, EncryptionError> {
        if key.key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeySize);
        }
        
        // Generate nonce (96 bits for AES-GCM)
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| EncryptionError::RngError)?;
        
        // Create sealing key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key.key_bytes)
            .map_err(|_| EncryptionError::KeyCreationFailed)?;
        
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut sealing_key = SealingKey::new(unbound_key, AesGcmNonceSequence::new(nonce_bytes));
        
        // Encrypt in-place
        let mut ciphertext = data.to_vec();
        let tag = sealing_key.seal_in_place_separate_tag(Aad::empty(), &mut ciphertext)
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        // Append tag to ciphertext
        ciphertext.extend_from_slice(tag.as_ref());
        
        info!("🔐 Data encrypted with AES-256-GCM (key_id: {})", key_id);
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            key_id: key_id.to_string(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            metadata: serde_json::json!({
                "sealed": true,
                "algorithm": "AES-256-GCM",
            }),
        })
    }
    
    /// Decrypt using AES-256-GCM via ring
    async fn decrypt_aes256_gcm(
        &self,
        data: &EncryptedData,
        key: &SecretKey,
    ) -> Result<Vec<u8>, EncryptionError> {
        if key.key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeySize);
        }
        
        if data.nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonce);
        }
        
        let nonce_bytes: [u8; 12] = data.nonce.clone().try_into()
            .map_err(|_| EncryptionError::InvalidNonce)?;
        
        // Create opening key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key.key_bytes)
            .map_err(|_| EncryptionError::KeyCreationFailed)?;
        
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut opening_key = OpeningKey::new(unbound_key, AesGcmNonceSequence::new(nonce_bytes));
        
        // Decrypt
        let mut ciphertext = data.ciphertext.clone();
        let plaintext = opening_key.open_in_place(Aad::empty(), &mut ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        info!("🔓 Data decrypted with AES-256-GCM (key_id: {})", data.key_id);
        
        Ok(plaintext.to_vec())
    }
    
    /// Encrypt using ChaCha20-Poly1305 via chacha20poly1305 crate
    async fn encrypt_chacha20_poly1305(
        &self,
        data: &[u8],
        key: &SecretKey,
        key_id: &str,
    ) -> Result<EncryptedData, EncryptionError> {
        if key.key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeySize);
        }
        
        // Generate nonce (96 bits for ChaCha20-Poly1305)
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| EncryptionError::RngError)?;
        
        // Create cipher
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key_bytes));
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        info!("🔐 Data encrypted with ChaCha20-Poly1305 (key_id: {})", key_id);
        
        Ok(EncryptedData {
            ciphertext: ciphertext.to_vec(),
            nonce: nonce_bytes.to_vec(),
            key_id: key_id.to_string(),
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            metadata: serde_json::json!({
                "sealed": true,
                "algorithm": "ChaCha20-Poly1305",
            }),
        })
    }
    
    /// Decrypt using ChaCha20-Poly1305 via chacha20poly1305 crate
    async fn decrypt_chacha20_poly1305(
        &self,
        data: &EncryptedData,
        key: &SecretKey,
    ) -> Result<Vec<u8>, EncryptionError> {
        if key.key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeySize);
        }
        
        if data.nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonce);
        }
        
        // Create cipher
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key_bytes));
        let nonce = ChaChaNonce::from_slice(&data.nonce);
        
        // Decrypt
        let plaintext = cipher.decrypt(nonce, data.ciphertext.as_ref())
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        info!("🔓 Data decrypted with ChaCha20-Poly1305 (key_id: {})", data.key_id);
        
        Ok(plaintext.to_vec())
    }
    
    /// Encrypt a string and return base64-encoded result
    pub async fn encrypt_string(
        &self,
        plaintext: &str,
        key_id: Option<&str>,
        algorithm: Option<EncryptionAlgorithm>,
    ) -> Result<String, EncryptionError> {
        let encrypted = self.encrypt(plaintext.as_bytes(), key_id, algorithm).await?;
        let json = serde_json::to_string(&encrypted)
            .map_err(|_| EncryptionError::SerializationError)?;
        Ok(base64::encode(json))
    }
    
    /// Decrypt a base64-encoded encrypted string
    pub async fn decrypt_string(&self, ciphertext: &str) -> Result<String, EncryptionError> {
        let json = base64::decode(ciphertext)
            .map_err(|_| EncryptionError::Base64Error)?;
        let encrypted: EncryptedData = serde_json::from_slice(&json)
            .map_err(|_| EncryptionError::DeserializationError)?;
        let plaintext = self.decrypt(&encrypted).await?;
        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::Utf8Error)
    }
    
    /// Generate a new encryption key
    pub async fn generate_key(&self, algorithm: EncryptionAlgorithm) -> Result<String, EncryptionError> {
        let mut key_bytes = vec![0u8; 32]; // 256 bits
        self.rng.fill(&mut key_bytes)
            .map_err(|_| EncryptionError::RngError)?;
        
        let key_id = uuid::Uuid::new_v4().to_string();
        let key = SecretKey::new(key_bytes, key_id.clone(), algorithm);
        
        let mut key_manager = self.key_manager.write().await;
        key_manager.store_key(key).await;
        
        info!("🔑 Generated new encryption key: {} (algorithm: {:?})", key_id, algorithm);
        
        Ok(key_id)
    }
    
    /// Get the default algorithm
    pub fn default_algorithm(&self) -> EncryptionAlgorithm {
        self.default_algorithm
    }
    
    /// Set the default algorithm
    pub fn set_default_algorithm(&mut self, algorithm: EncryptionAlgorithm) {
        self.default_algorithm = algorithm;
        info!("🔐 Default encryption algorithm set to: {:?}", algorithm);
    }
}

// Nonce sequence for AES-GCM
struct AesGcmNonceSequence {
    nonce: [u8; 12],
}

impl AesGcmNonceSequence {
    fn new(nonce: [u8; 12]) -> Self {
        Self { nonce }
    }
}

impl NonceSequence for AesGcmNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        Ok(Nonce::assume_unique_for_key(self.nonce))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key size")]
    InvalidKeySize,
    
    #[error("Invalid nonce")]
    InvalidNonce,
    
    #[error("RNG error")]
    RngError,
    
    #[error("Key creation failed")]
    KeyCreationFailed,
    
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed - possible tampering")]
    DecryptionFailed,
    
    #[error("Serialization error")]
    SerializationError,
    
    #[error("Deserialization error")]
    DeserializationError,
    
    #[error("Base64 error")]
    Base64Error,
    
    #[error("UTF-8 error")]
    Utf8Error,
}
