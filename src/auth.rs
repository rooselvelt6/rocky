// src/auth.rs - Módulo de autenticación JWT

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

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

impl UserRole {
    /// Verifica si el rol tiene permiso para una operación
    pub fn can_create_patient(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Doctor)
    }

    pub fn can_edit_patient(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Doctor)
    }

    pub fn can_delete_patient(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn can_create_assessment(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Doctor | UserRole::Nurse)
    }
}

/// Estructura de usuario autenticado
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub role: UserRole,
}

// TODO: Implementar generación y validación de JWT
// Requiere agregar dependencia: jsonwebtoken = "9.2"
//
// pub fn generate_token(user_id: &str, role: UserRole) -> Result<String, TokenError> {
//     let claims = Claims {
//         sub: user_id.to_string(),
//         exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
//         role,
//     };
//
//     jsonwebtoken::encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret(SECRET_KEY),
//     )
// }
//
// pub fn validate_token(token: &str) -> Result<Claims, TokenError> {
//     jsonwebtoken::decode::<Claims>(
//         token,
//         &DecodingKey::from_secret(SECRET_KEY),
//         &Validation::default(),
//     )
// }

/// Middleware de autenticación (stub para desarrollo)
/// TODO: Implementar validación real de JWT
pub async fn auth_middleware(
    _headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Por ahora, permite todas las solicitudes en modo desarrollo
    // En producción, debe validar el token JWT del header Authorization

    // let auth_header = _headers
    //     .get("Authorization")
    //     .and_then(|h| h.to_str().ok())
    //     .ok_or(StatusCode::UNAUTHORIZED)?;
    //
    // if !auth_header.starts_with("Bearer ") {
    //     return Err(StatusCode::UNAUTHORIZED);
    // }
    //
    // let token = &auth_header[7..];
    // let claims = validate_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    //
    // request.extensions_mut().insert(AuthenticatedUser {
    //     id: claims.sub,
    //     role: claims.role,
    // });

    // Modo desarrollo: usuario admin ficticio
    #[cfg(debug_assertions)]
    {
        request.extensions_mut().insert(AuthenticatedUser {
            id: "dev_user".to_string(),
            role: UserRole::Admin,
        });
    }

    Ok(next.run(request).await)
}
