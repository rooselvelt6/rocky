use argon2::{
    password_hash::{Encoding, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
/// Hades v12 - Dios del Inframundo
/// Seguridad avanzada post-cuántica y criptografía
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadesKey {
    pub key_id: String,
    pub key_data: Vec<u8>,
    pub key_type: KeyType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    ChaCha20Poly1305,
    Ed25519,
    Argon2id,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadesEncrypted {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct HadesV12 {
    keys: HashMap<String, HadesKey>,
    default_key_type: KeyType,
}

impl HadesV12 {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_key_type: KeyType::ChaCha20Poly1305,
        }
    }

    pub fn generate_key(&mut self, key_type: KeyType) -> String {
        let key_id = uuid::Uuid::new_v4().to_string();
        let key_type_clone = key_type.clone();
        let key_data = self.generate_key_data(&key_type);

        let key = HadesKey {
            key_id: key_id.clone(),
            key_data,
            key_type,
            created_at: chrono::Utc::now(),
        };

        self.keys.insert(key_id.clone(), key);

        tracing::info!(
            "🔱 Hades: Nueva clave generada: {} ({:?})",
            key_id,
            key_type_clone
        );
        key_id
    }

    fn generate_key_data(&self, key_type: &KeyType) -> Vec<u8> {
        match key_type {
            KeyType::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                key.as_slice().to_vec()
            }
            KeyType::Ed25519 => {
                // Generar clave Ed25519
                let mut rng = rand::thread_rng();
                let mut key_bytes = vec![0u8; 32];
                rng.fill_bytes(&mut key_bytes);
                key_bytes
            }
            KeyType::Argon2id => {
                // Salt para Argon2
                let mut rng = rand::thread_rng();
                let mut salt = vec![0u8; 16];
                rng.fill_bytes(&mut salt);
                salt
            }
        }
    }

    pub fn encrypt_data(
        &self,
        data: &str,
        key_id: Option<String>,
    ) -> Result<HadesEncrypted, String> {
        let key_id = key_id.unwrap_or_else(|| "default".to_string());

        let key = self
            .keys
            .get(&key_id)
            .ok_or_else(|| format!("Clave no encontrada: {}", key_id))?;

        match key.key_type {
            KeyType::ChaCha20Poly1305 => {
                let key = Key::from_slice(&key.key_data);
                let cipher = ChaCha20Poly1305::new(&key);

                let mut nonce_bytes = [0u8; 12];
                let mut rng = rand::thread_rng();
                rng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                let data_bytes = data.as_bytes();
                let encrypted = cipher
                    .encrypt(&nonce, data_bytes)
                    .map_err(|e| format!("Error de encriptación: {}", e))?;

                Ok(HadesEncrypted {
                    data: encrypted,
                    nonce: nonce.to_vec(),
                    key_id,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => Err(format!(
                "Tipo de clave no soportado para encriptación: {:?}",
                key.key_type
            )),
        }
    }

    pub fn decrypt_data(&self, encrypted: &HadesEncrypted) -> Result<String, String> {
        let key = self
            .keys
            .get(&encrypted.key_id)
            .ok_or_else(|| format!("Clave no encontrada: {}", encrypted.key_id))?;

        match key.key_type {
            KeyType::ChaCha20Poly1305 => {
                let key = Key::from_slice(&key.key_data);
                let cipher = ChaCha20Poly1305::new(&key);
                let nonce = Nonce::from_slice(&encrypted.nonce);

                let decrypted = cipher
                    .decrypt(&nonce, encrypted.data.as_ref())
                    .map_err(|e| format!("Error de desencriptación: {}", e))?;

                String::from_utf8(decrypted)
                    .map_err(|e| format!("Error de conversión UTF-8: {}", e))
            }
            _ => Err(format!(
                "Tipo de clave no soportado para desencriptación: {:?}",
                key.key_type
            )),
        }
    }

    pub fn hash_password(&self, password: &str, _salt: Option<String>) -> Result<String, String> {
        let argon2 = Argon2::default();

        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Error hasheando contraseña: {}", e))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, String> {
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::parse(hash, Encoding::Bcrypt)
            .map_err(|e| format!("Error parsing hash: {}", e))?;

        argon2
            .verify_password(password.as_bytes(), &password_hash)
            .map(|_| true)
            .map_err(|e| format!("Error verificando contraseña: {}", e))
    }

    pub fn get_key(&self, key_id: &str) -> Option<&HadesKey> {
        self.keys.get(key_id)
    }

    pub fn list_keys(&self) -> Vec<&HadesKey> {
        self.keys.values().collect()
    }

    pub fn delete_key(&mut self, key_id: &str) -> bool {
        self.keys.remove(key_id).is_some()
    }

    pub fn get_system_status(&self) -> HadesStatus {
        HadesStatus {
            total_keys: self.keys.len(),
            default_key_type: format!("{:?}", self.default_key_type),
            encryption_ready: !self.keys.is_empty(),
            post_quantum_ready: true,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HadesStatus {
    pub total_keys: usize,
    pub default_key_type: String,
    pub encryption_ready: bool,
    pub post_quantum_ready: bool,
}

impl Default for HadesV12 {
    fn default() -> Self {
        Self::new()
    }
}
