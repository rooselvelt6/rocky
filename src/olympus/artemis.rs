use crate::olympus::{GodActor, GodCommand};
use async_trait::async_trait;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub use uci::models::user::UserRole;

const SECRET_KEY: &[u8] = b"v10_sovereign_secret_key_uci";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub role: UserRole,
}

pub struct Artemis;

impl Artemis {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_token(&self, user_id: &str, role: UserRole) -> Result<String, String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("timestamp")
            .timestamp() as usize;

        let claims = Claims { sub: user_id.to_string(), exp: expiration, role };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))
            .map_err(|e| format!("Artemis: Error al forjar token: {}", e))
    }

    pub fn validate_token(token: &str) -> Result<Claims, String> {
        decode::<Claims>(token, &DecodingKey::from_secret(SECRET_KEY), &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| format!("Artemis: Token inválido: {}", e))
    }
    
    pub async fn auth_middleware(
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());

        if let Some(header_value) = auth_header {
            if header_value.starts_with("Bearer ") {
                let token = &header_value[7..];
                if let Ok(claims) = Self::validate_token(token) {
                    request.extensions_mut().insert(AuthenticatedUser {
                        id: claims.sub,
                        role: claims.role,
                    });
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
        Ok(next.run(request).await)
    }
}

#[async_trait]
impl GodActor for Artemis {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🏹 Artemis: Defensor de la soberanía y gestor de acceso activo.");
        Ok(())
    }

    async fn handle_command(&mut self, _cmd: GodCommand) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
