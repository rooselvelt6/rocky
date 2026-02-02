/// Artemis v12 - Authentication Optimizado
/// Sistema simplificado sin dependencias complejas

use axum::http::StatusCode;
use axum::response::Response;
use axum::extract::Request;
use axum::middleware::Next;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use crate::models::user::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct ArtemisV12 {
    jwt_secret: String,
}

impl ArtemisV12 {
    pub fn new() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "olympus_v12_default_secret_change_in_production".to_string()),
        }
    }

    pub fn generate_token(&self, user_id: &str, role: UserRole) -> Result<String, StatusCode> {
        let claims = jwt_claims::Claims {
            sub: user_id.to_string(),
            role: format!("{:?}", role),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn validate_token(&self, token: &str) -> Result<jwt_claims::Claims, StatusCode> {
        decode::<jwt_claims::Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| StatusCode::UNAUTHORIZED)
    }

    pub async fn auth_middleware_v12(
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let auth_header = request.headers().get("authorization");
        
        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    
                    let artemis = ArtemisV12::new();
                    match artemis.validate_token(token) {
                        Ok(claims) => {
                            let role = match claims.role.as_str() {
                                "Admin" => UserRole::Admin,
                                "Doctor" => UserRole::Doctor,
                                "Nurse" => UserRole::Nurse,
                                _ => UserRole::Nurse,
                            };
                            
                            let user_id = claims.sub.clone();
                            let user = AuthenticatedUser {
                                id: user_id.clone(),
                                username: user_id,
                                role,
                            };
                            
                            let mut request = request;
                            request.extensions_mut().insert(user);
                            
                            Ok(next.run(request).await)
                        }
                        Err(_) => Err(StatusCode::UNAUTHORIZED),
                    }
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthenticatedUser, StatusCode> {
        // Base de datos en memoria para desarrollo
        match (username, password) {
            ("admin", "admin") => Ok(AuthenticatedUser {
                id: "user:admin".to_string(),
                username: username.to_string(),
                role: UserRole::Admin,
            }),
            ("doctor", "doctor") => Ok(AuthenticatedUser {
                id: "user:doctor".to_string(),
                username: username.to_string(),
                role: UserRole::Doctor,
            }),
            ("nurse", "nurse") => Ok(AuthenticatedUser {
                id: "user:nurse".to_string(),
                username: username.to_string(),
                role: UserRole::Nurse,
            }),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: AuthenticatedUser,
    pub expires_in: i64,
    pub token_type: String,
}

impl LoginResponse {
    pub fn new(token: String, user: AuthenticatedUser) -> Self {
        Self {
            token,
            user,
            expires_in: 24 * 60 * 60, // 24 hours
            token_type: "Bearer".to_string(),
        }
    }
}

pub mod jwt_claims {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub role: String,
        pub exp: usize,
        pub iat: usize,
    }
}