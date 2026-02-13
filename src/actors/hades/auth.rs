// src/actors/hades/auth.rs
// OLYMPUS v15 - Hades Authentication Service
// Autenticación real con Argon2id, JWT, y RBAC

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::SaltString, password_hash::rand_core::OsRng};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use tracing::{info, warn};

use crate::actors::hades::audit::{AuditLogger, AuditResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHash {
    pub hash: String,
    pub salt: String,
    pub algorithm: String,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: PasswordHash,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub login_attempts: u32,
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    SuperAdmin,    // Full access to everything
    Admin,         // Administrative access
    Doctor,        // Medical staff
    Nurse,         // Nursing staff
    Researcher,    // Research access
    Auditor,       // Read-only audit access
    Patient,       // Patient portal access
    Guest,         // Limited public access
}

impl Role {
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::SuperAdmin => vec![
                Permission::All,
            ],
            Role::Admin => vec![
                Permission::UserManage,
                Permission::SystemConfigure,
                Permission::AuditRead,
                Permission::DataRead,
                Permission::DataWrite,
            ],
            Role::Doctor => vec![
                Permission::PatientRead,
                Permission::PatientWrite,
                Permission::MedicalRecordsRead,
                Permission::MedicalRecordsWrite,
                Permission::PrescriptionsWrite,
                Permission::DataRead,
            ],
            Role::Nurse => vec![
                Permission::PatientRead,
                Permission::VitalsWrite,
                Permission::MedicalRecordsRead,
                Permission::DataRead,
            ],
            Role::Researcher => vec![
                Permission::AnalyticsRead,
                Permission::DataRead,
                Permission::ReportsRead,
            ],
            Role::Auditor => vec![
                Permission::AuditRead,
                Permission::ReportsRead,
                Permission::DataRead,
            ],
            Role::Patient => vec![
                Permission::OwnRecordsRead,
                Permission::OwnDataWrite,
            ],
            Role::Guest => vec![
                Permission::PublicRead,
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Permission {
    All,
    UserManage,
    SystemConfigure,
    AuditRead,
    DataRead,
    DataWrite,
    PatientRead,
    PatientWrite,
    MedicalRecordsRead,
    MedicalRecordsWrite,
    VitalsWrite,
    PrescriptionsWrite,
    AnalyticsRead,
    ReportsRead,
    OwnRecordsRead,
    OwnDataWrite,
    PublicRead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,           // Subject (user ID)
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub iat: i64,              // Issued at
    pub exp: i64,              // Expiration
    pub jti: String,           // JWT ID
    pub aud: String,           // Audience
    pub iss: String,           // Issuer
}

#[derive(Debug, Clone)]
pub struct AuthenticationService {
    users: Arc<RwLock<HashMap<String, User>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    audit_logger: Arc<RwLock<AuditLogger>>,
    jwt_secret: Arc<RwLock<String>>,
    token_duration_hours: u64,
    max_login_attempts: u32,
    lockout_duration_minutes: u64,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuthenticationService {
    pub fn new(audit_logger: Arc<RwLock<AuditLogger>>) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                let secret = uuid::Uuid::new_v4().to_string();
                warn!("JWT_SECRET not set, using random secret. Set JWT_SECRET for production!");
                secret
            });
        
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            audit_logger,
            jwt_secret: Arc::new(RwLock::new(jwt_secret)),
            token_duration_hours: 24,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
        }
    }
    
    /// Hash a password using Argon2id
    pub async fn hash_password(&self, password: &str) -> Result<PasswordHash, AuthenticationError> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthenticationError::HashingError(e.to_string()))?;
        
        Ok(PasswordHash {
            hash: password_hash.to_string(),
            salt: salt.to_string(),
            algorithm: "Argon2id".to_string(),
            version: password_hash.version.unwrap_or(0x13),
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Verify a password against a hash
    pub async fn verify_password(&self, password: &str, hash: &PasswordHash) -> Result<bool, AuthenticationError> {
        let parsed_hash = argon2::PasswordHash::new(&hash.hash)
            .map_err(|e| AuthenticationError::InvalidHash(e.to_string()))?;
        
        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthenticationError::VerificationError(e.to_string())),
        }
    }
    
    /// Create a new user
    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
        roles: Vec<Role>,
    ) -> Result<User, AuthenticationError> {
        let users = self.users.read().await;
        
        // Check if username or email already exists
        if users.values().any(|u| u.username == username) {
            return Err(AuthenticationError::UsernameExists);
        }
        if users.values().any(|u| u.email == email) {
            return Err(AuthenticationError::EmailExists);
        }
        drop(users);
        
        // Hash password
        let password_hash = self.hash_password(&password).await?;
        
        // Collect all permissions from roles
        let mut all_permissions = Vec::new();
        for role in &roles {
            all_permissions.extend(role.permissions());
        }
        // Deduplicate
        all_permissions.sort();
        all_permissions.dedup();
        
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            roles,
            permissions: all_permissions,
            created_at: chrono::Utc::now(),
            last_login: None,
            login_attempts: 0,
            locked_until: None,
            mfa_enabled: false,
            mfa_secret: None,
        };
        
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());
        drop(users);
        
        // Audit log
        self.audit_logger.write().await.log(
            "USER_CREATED",
            &user.id,
            &user.username,
            AuditResult::Success,
            serde_json::json!({
                "email": user.email,
                "roles": user.roles.iter().map(|r| format!("{:?}", r)).collect::<Vec<_>>(),
            }),
        ).await;
        
        info!("👤 User created: {} ({})", user.username, user.id);
        
        Ok(user)
    }
    
    /// Authenticate user and create session
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(User, String), AuthenticationError> {
        let users = self.users.read().await;
        
        // Find user by username or email
        let user = users.values()
            .find(|u| u.username == username || u.email == username)
            .cloned()
            .ok_or(AuthenticationError::InvalidCredentials)?;
        drop(users);
        
        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if chrono::Utc::now() < locked_until {
                warn!("🔒 Account locked for user: {}", username);
                
                self.audit_logger.write().await.log(
                    "LOGIN_ATTEMPT_LOCKED",
                    &user.id,
                    username,
                    AuditResult::Failure,
                    serde_json::json!({
                        "locked_until": locked_until,
                        "ip_address": ip_address,
                    }),
                ).await;
                
                return Err(AuthenticationError::AccountLocked);
            }
        }
        
        // Verify password
        let password_valid = self.verify_password(password, &user.password_hash).await?;
        
        if !password_valid {
            // Increment login attempts
            let mut users = self.users.write().await;
            if let Some(u) = users.get_mut(&user.id) {
                u.login_attempts += 1;
                
                // Lock account if max attempts reached
                if u.login_attempts >= self.max_login_attempts {
                    u.locked_until = Some(chrono::Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes as i64));
                    warn!("🔒 Account locked after {} failed attempts: {}", u.login_attempts, username);
                }
            }
            drop(users);
            
            // Audit log
            self.audit_logger.write().await.log(
                "LOGIN_FAILED",
                &user.id,
                username,
                AuditResult::Failure,
                serde_json::json!({
                    "reason": "Invalid password",
                    "attempt": user.login_attempts + 1,
                    "ip_address": ip_address,
                }),
            ).await;
            
            return Err(AuthenticationError::InvalidCredentials);
        }
        
        // Reset login attempts on successful login
        let mut users = self.users.write().await;
        if let Some(u) = users.get_mut(&user.id) {
            u.login_attempts = 0;
            u.locked_until = None;
            u.last_login = Some(chrono::Utc::now());
        }
        let user = users.get(&user.id).cloned().unwrap();
        drop(users);
        
        // Generate JWT token
        let token = self.generate_jwt(&user).await?;
        
        // Create session
        let session = Session {
            token: token.clone(),
            user_id: user.id.clone(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            ip_address,
            user_agent,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(token.clone(), session);
        drop(sessions);
        
        // Audit log
        self.audit_logger.write().await.log(
            "LOGIN_SUCCESS",
            &user.id,
            &user.username,
            AuditResult::Success,
            serde_json::json!({}),
        ).await;
        
        info!("✅ User authenticated: {} ({})", user.username, user.id);
        
        Ok((user, token))
    }
    
    /// Generate JWT token
    async fn generate_jwt(&self, user: &User) -> Result<String, AuthenticationError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(self.token_duration_hours as i64);
        
        let claims = JwtClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            roles: user.roles.iter().map(|r| format!("{:?}", r)).collect(),
            permissions: user.permissions.iter().map(|p| format!("{:?}", p)).collect(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            aud: "olympus".to_string(),
            iss: "hades".to_string(),
        };
        
        let secret = self.jwt_secret.read().await;
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ).map_err(|e| AuthenticationError::TokenCreationError(e.to_string()))?;
        
        Ok(token)
    }
    
    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<JwtClaims, AuthenticationError> {
        let secret = self.jwt_secret.read().await;
        
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthenticationError::TokenExpired,
            _ => AuthenticationError::InvalidToken,
        })?;
        
        Ok(token_data.claims)
    }
    
    /// Check if user has permission
    pub async fn has_permission(&self, user_id: &str, permission: Permission) -> bool {
        let users = self.users.read().await;
        
        if let Some(user) = users.get(user_id) {
            user.permissions.contains(&permission) || user.permissions.contains(&Permission::All)
        } else {
            false
        }
    }
    
    /// Check if user has role
    pub async fn has_role(&self, user_id: &str, role: Role) -> bool {
        let users = self.users.read().await;
        
        if let Some(user) = users.get(user_id) {
            user.roles.contains(&role)
        } else {
            false
        }
    }
    
    /// Logout user
    pub async fn logout(&self, token: &str) -> Result<(), AuthenticationError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.remove(token) {
            drop(sessions);
            
            self.audit_logger.write().await.log(
                "LOGOUT",
                &session.user_id,
                "",
                AuditResult::Success,
                serde_json::json!({}),
            ).await;
            
            info!("👋 User logged out: {}", session.user_id);
            Ok(())
        } else {
            Err(AuthenticationError::InvalidToken)
        }
    }
    
    /// Cleanup expired sessions
    pub async fn cleanup_sessions(&self, max_age: Duration) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| now.duration_since(session.created_at) > max_age)
            .map(|(token, _)| token.clone())
            .collect();
        
        for token in &expired {
            sessions.remove(token);
        }
        
        expired.len()
    }
    
    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }
    
    /// Get active session count
    pub async fn active_sessions(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
    
    /// Change user password
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AuthenticationError> {
        let users = self.users.read().await;
        let user = users.get(user_id).cloned()
            .ok_or(AuthenticationError::UserNotFound)?;
        drop(users);
        
        // Verify old password
        if !self.verify_password(old_password, &user.password_hash).await? {
            self.audit_logger.write().await.log(
                "PASSWORD_CHANGE_FAILED",
                user_id,
                &user.username,
                AuditResult::Failure,
                serde_json::json!({"reason": "Invalid old password"}),
            ).await;
            
            return Err(AuthenticationError::InvalidCredentials);
        }
        
        // Hash new password
        let new_hash = self.hash_password(new_password).await?;
        
        // Update user
        let mut users = self.users.write().await;
        if let Some(u) = users.get_mut(user_id) {
            u.password_hash = new_hash;
        }
        drop(users);
        
        // Audit log
        self.audit_logger.write().await.log(
            "PASSWORD_CHANGED",
            user_id,
            &user.username,
            AuditResult::Success,
            serde_json::json!({}),
        ).await;
        
        // Invalidate all sessions except current
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.user_id != user_id);
        drop(sessions);
        
        info!("🔑 Password changed for user: {}", user.username);
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Username already exists")]
    UsernameExists,
    
    #[error("Email already exists")]
    EmailExists,
    
    #[error("Account locked")]
    AccountLocked,
    
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    #[error("Token creation error: {0}")]
    TokenCreationError(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid token")]
    InvalidToken,
}
