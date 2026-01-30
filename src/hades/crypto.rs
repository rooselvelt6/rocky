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

/// Estructura Maestra de HADES para operaciones criptográficas
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Hades {
    key: [u8; 32],
}

impl Hades {
    /// Inicializa HADES derivando una llave maestra desde una variable de entorno
    pub fn new() -> Self {
        let master_secret = env::var("HADES_SECRET").unwrap_or_else(|_| "secret_de_emergencia_uci".to_string());
        let salt = SaltString::generate(&mut OsRng);
        
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();
        
        // Derivamos una llave de 32 bytes altamente segura
        if let Ok(hash) = argon2.hash_password(master_secret.as_bytes(), &salt) {
             // Por simplicidad en este nivel, usamos los primeros 32 bytes del hash
             let hash_bytes = hash.hash().unwrap();
             let len = hash_bytes.len().min(32);
             key[..len].copy_from_slice(&hash_bytes[..len]);
        }

        Self { key }
    }

    /// Cifra un texto plano devolviendo un vector de bytes (Base64 recomendado para DB)
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(&self.key.into());
        let nonce = Nonce::from_slice(b"unique nonce 12"); // En prod, esto debe ser aleatorio y guardado con el dato

        cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Fallo en el cifrado HADES: {}", e))
    }

    /// Descifra datos provenientes de la base de datos
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, String> {
        let cipher = ChaCha20Poly1305::new(&self.key.into());
        let nonce = Nonce::from_slice(b"unique nonce 12");

        let decrypted_bytes = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Fallo en el descifrado HADES (Llave incorrecta o datos corruptos): {}", e))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Error de codificación UTF8 tras descifrar: {}", e))
    }
}

/// Helper para hashing de integridad (El Hilo Rojo)
pub fn compute_hash(data: &str) -> String {
    let hash = blake3::hash(data.as_bytes());
    hash.to_hex().to_string()
}
