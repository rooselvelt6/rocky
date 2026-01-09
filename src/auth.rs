// src/auth.rs - Módulo de autenticación JWT

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// SECRET KEY FOR DEVELOPMENT ONLY - TODO: Move to env var
const SECRET_KEY: &[u8] = b"super_secret_key_for_dev_only";

/// Claims del token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,    // Subject (user ID)
    pub exp: usize,     // Expiration time
    pub role: UserRole, // Rol del usuario
}

/// Roles de usuario en el sistema
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    Admin,    // Acceso completo
    Doctor,   // Puede crear/editar pacientes y evaluaciones
    Nurse,    // Puede crear evaluaciones, solo lectura de pacientes
    ReadOnly, // Solo consulta
}

/// Estructura de usuario autenticado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub role: UserRole,
}

#[derive(Debug)]
pub enum TokenError {
    CreationError,
    ValidationError,
}

/// Generates a new JWT token
pub fn generate_token(user_id: &str, role: UserRole) -> Result<String, TokenError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        role,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
    .map_err(|_| TokenError::CreationError)
}

/// Validates a JWT token
pub fn validate_token(token: &str) -> Result<Claims, TokenError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| TokenError::ValidationError)
}

/// Middleware de autenticación (Soft Enforcement)
/// Valida el token si existe, pero NO bloquea si falta (por ahora)
pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for Authorization header
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header_value) if header_value.starts_with("Bearer ") => {
            let token = &header_value[7..];
            match validate_token(token) {
                Ok(claims) => {
                    // Valid token: Insert user into request extensions
                    request.extensions_mut().insert(AuthenticatedUser {
                        id: claims.sub,
                        role: claims.role,
                    });
                }
                Err(_) => {
                    // Invalid token: Wait, for "Soft" enforcement, do we block?
                    // Safe approach: Log it, but maybe treat as guest for now to prevent breakage
                    // or return 401 if a bad token is explicitly provided?
                    // Let's block ONLY if a BAD token is provided, but allow NO token.
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
        _ => {
            // No token provided.
            // In "Soft Enforcement" mode, we allow the request to proceed as anonymous/guest.
            // Or we insert a default "Dev/Guest" user if allow_anonymous is true.

            // For now, allow pass-through.
            // The handler will check extensions().get::<AuthenticatedUser>() if it needs it.
        }
    }

    Ok(next.run(request).await)
}
