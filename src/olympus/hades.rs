use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use zeroize::{Zeroize, ZeroizeOnDrop};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use std::env;

/// Estructura Maestra de HADES para operaciones criptográficas (v10)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Hades {
    key: [u8; 32],
}

impl Hades {
    /// Inicializa HADES derivando una llave maestra (Secret Guardian)
    pub fn new() -> Self {
        let master_secret = env::var("HADES_SECRET").unwrap_or_else(|_| "secret_de_emergencia_v10".to_string());
        let salt = SaltString::generate(&mut OsRng);
        
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();
        
        if let Ok(hash) = argon2.hash_password(master_secret.as_bytes(), &salt) {
             let hash_bytes = hash.hash().unwrap();
             let len = hash_bytes.len().min(32);
             key[..len].copy_from_slice(&hash_bytes[..len]);
        }

        Self { key }
    }

    /// Cifra un texto plano (Resistencia Post-Cuántica Simulada)
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(&self.key.into());
        let nonce = Nonce::from_slice(b"v10unique_nonc"); // 12 bytes

        cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Fallo en el cifrado HADES: {}", e))
    }

    /// Descifra datos sensibles
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, String> {
        let cipher = ChaCha20Poly1305::new(&self.key.into());
        let nonce = Nonce::from_slice(b"v10unique_nonc");

        let decrypted_bytes = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Fallo en el descifrado HADES: {}", e))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Error UTF8 en HADES: {}", e))
    }

    /// Helper para hashing de integridad (El Hilo Rojo)
    pub fn compute_hash(data: &str) -> String {
        let hash = blake3::hash(data.as_bytes());
        hash.to_hex().to_string()
    }
}

#[async_trait]
impl GodActor for Hades {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🛡️ Hades: Escudo v10 activo y protegiendo memoria.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
