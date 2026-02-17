pub mod auth;
pub mod patients;
pub mod scales;
pub mod admin;
pub mod db;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
    pub message: String,
}

impl AuthResponse {
    pub fn success(token: String, user_id: String, username: String, email: String, roles: Vec<String>, message: String) -> Self {
        Self {
            success: true,
            token: Some(token),
            user_id: Some(user_id),
            username: Some(username),
            email: Some(email),
            roles: Some(roles),
            message,
        }
    }
    
    pub fn simple_success(message: String) -> Self {
        Self {
            success: true,
            token: None,
            user_id: None,
            username: None,
            email: None,
            roles: None,
            message,
        }
    }
    
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            token: None,
            user_id: None,
            username: None,
            email: None,
            roles: None,
            message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}
