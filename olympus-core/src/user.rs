use crate::Thing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Doctor,
    Nurse,
    Staff,
    ReadOnly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

impl User {
    pub fn new(username: String, full_name: String, email: String, role: UserRole) -> Self {
        Self {
            id: None,
            username,
            full_name,
            email,
            role,
            password_hash: None,
            created_at: chrono::Utc::now(),
            is_active: true,
        }
    }
}
