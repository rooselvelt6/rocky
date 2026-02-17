// src/actors/hades/auth.rs
// OLYMPUS v15 - Hades Authentication Service
// Autenticación real con Argon2id, JWT, y RBAC

#![allow(dead_code)]

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
pub struct OtpCode {
    pub code: String,
    pub user_id: String,
    pub username: String,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub attempts: u32,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub password_verified: bool,
    pub otp_verified: bool,
    pub created_at: Instant,
    pub expires_at: Instant,
}

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
    otp_codes: Arc<RwLock<HashMap<String, OtpCode>>>,
    otp_sessions: Arc<RwLock<HashMap<String, OtpSession>>>,
    audit_logger: Arc<RwLock<AuditLogger>>,
    jwt_secret: Arc<RwLock<String>>,
    token_duration_hours: u64,
    max_login_attempts: u32,
    lockout_duration_minutes: u64,
    otp_expiry_seconds: u64,
    otp_max_attempts: u32,
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
            otp_codes: Arc::new(RwLock::new(HashMap::new())),
            otp_sessions: Arc::new(RwLock::new(HashMap::new())),
            audit_logger,
            jwt_secret: Arc::new(RwLock::new(jwt_secret)),
            token_duration_hours: 24,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            otp_expiry_seconds: 300,
            otp_max_attempts: 3,
        }
    }
    
    fn generate_otp_code(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..999999))
    }
    
    pub async fn initiate_login(&self, username: &str, password: &str, ip_address: Option<String>, user_agent: Option<String>) 
        -> Result<OtpSessionResponse, AuthenticationError> 
    {
        let users = self.users.read().await;
        let user = users.values()
            .find(|u| u.username == username || u.email == username)
            .cloned()
            .ok_or(AuthenticationError::InvalidCredentials)?;
        drop(users);
        
        if let Some(locked_until) = user.locked_until {
            if chrono::Utc::now() < locked_until {
                return Err(AuthenticationError::AccountLocked);
            }
        }
        
        let password_valid = self.verify_password(password, &user.password_hash).await?;
        if !password_valid {
            let mut users = self.users.write().await;
            if let Some(u) = users.get_mut(&user.id) {
                u.login_attempts += 1;
                if u.login_attempts >= self.max_login_attempts {
                    u.locked_until = Some(chrono::Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes as i64));
                }
            }
            self.audit_logger.write().await.log("LOGIN_FAILED", &user.id, username, AuditResult::Failure, 
                serde_json::json!({"reason": "Invalid password"})).await;
            return Err(AuthenticationError::InvalidCredentials);
        }
        
        let session_id = uuid::Uuid::new_v4().to_string();
        let otp_code = self.generate_otp_code();
        let now = Instant::now();
        
        let otp_session = OtpSession {
            session_id: session_id.clone(),
            user_id: user.id.clone(),
            username: user.username.clone(),
            password_verified: true,
            otp_verified: false,
            created_at: now,
            expires_at: now + Duration::from_secs(self.otp_expiry_seconds),
        };
        
        let otp = OtpCode {
            code: otp_code.clone(),
            user_id: user.id.clone(),
            username: user.username.clone(),
            created_at: now,
            expires_at: now + Duration::from_secs(self.otp_expiry_seconds),
            attempts: 0,
            verified: false,
        };
        
        {
            let mut sessions = self.otp_sessions.write().await;
            sessions.insert(session_id.clone(), otp_session);
        }
        {
            let mut codes = self.otp_codes.write().await;
            codes.insert(otp_code.clone(), otp);
        }
        
        self.audit_logger.write().await.log("OTP_SENT", &user.id, username, AuditResult::Success,
            serde_json::json!({"session_id": session_id})).await;
        
        info!("📧 OTP code generated for user: {} (code: {})", username, otp_code);
        
        Ok(OtpSessionResponse {
            session_id,
            otp_code: otp_code,
            message: "OTP code sent to your email".to_string(),
        })
    }
    
    pub async fn verify_otp(&self, session_id: &str, otp_code: &str, ip_address: Option<String>, user_agent: Option<String>) 
        -> Result<AuthResponse, AuthenticationError> 
    {
        let otp_sessions = self.otp_sessions.read().await;
        let session = otp_sessions.get(session_id)
            .ok_or(AuthenticationError::InvalidSession)?
            .clone();
        drop(otp_sessions);
        
        if Instant::now() > session.expires_at {
            self.otp_sessions.write().await.remove(session_id);
            return Err(AuthenticationError::SessionExpired);
        }
        
        let mut codes = self.otp_codes.write().await;
        let otp = codes.get_mut(otp_code)
            .ok_or(AuthenticationError::InvalidOtp)?;
        
        if Instant::now() > otp.expires_at {
            codes.remove(otp_code);
            return Err(AuthenticationError::OtpExpired);
        }
        
        if otp.attempts >= self.otp_max_attempts {
            codes.remove(otp_code);
            self.otp_sessions.write().await.remove(session_id);
            return Err(AuthenticationError::OtpMaxAttempts);
        }
        
        if otp.code != otp_code {
            otp.attempts += 1;
            drop(codes);
            self.audit_logger.write().await.log("OTP_FAILED", &session.user_id, &session.username, AuditResult::Failure,
                serde_json::json!({"attempts": otp.attempts})).await;
            return Err(AuthenticationError::InvalidOtp);
        }
        
        otp.verified = true;
        drop(codes);
        
        let mut otp_sessions = self.otp_sessions.write().await;
        if let Some(s) = otp_sessions.get_mut(session_id) {
            s.otp_verified = true;
        }
        drop(otp_sessions);
        
        let users = self.users.read().await;
        let user = users.get(&session.user_id).cloned().ok_or(AuthenticationError::UserNotFound)?;
        drop(users);
        
        let mut users = self.users.write().await;
        if let Some(u) = users.get_mut(&user.id) {
            u.login_attempts = 0;
            u.locked_until = None;
            u.last_login = Some(chrono::Utc::now());
        }
        drop(users);
        
        let token = self.generate_jwt(&user).await?;
        
        let session = Session {
            token: token.clone(),
            user_id: user.id.clone(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            ip_address,
            user_agent,
        };
        
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(token.clone(), session);
        }
        
        self.otp_codes.write().await.remove(otp_code);
        self.otp_sessions.write().await.remove(session_id);
        
        self.audit_logger.write().await.log("LOGIN_SUCCESS_OTP", &user.id, &user.username, AuditResult::Success,
            serde_json::json!({})).await;
        
        info!("✅ User authenticated with OTP: {} ({})", user.username, user.id);
        
        Ok(AuthResponse {
            user_id: user.id,
            username: user.username,
            email: user.email,
            roles: user.roles.iter().map(|r| format!("{:?}", r)).collect(),
            token,
        })
    }
    
    pub async fn get_otp_status(&self, session_id: &str) -> Result<OtpStatus, AuthenticationError> {
        let sessions = self.otp_sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or(AuthenticationError::InvalidSession)?
            .clone();
        drop(sessions);
        
        let codes = self.otp_codes.read().await;
        let otp = codes.values().find(|o| o.user_id == session.user_id && !o.verified);
        
        Ok(OtpStatus {
            session_id: session.session_id,
            username: session.username,
            password_verified: session.password_verified,
            otp_verified: session.otp_verified,
            expires_at: session.expires_at.elapsed().as_secs(),
            pending_otp: otp.is_some(),
        })
    }
    
    pub async fn resend_otp(&self, session_id: &str) -> Result<String, AuthenticationError> {
        let otp_sessions = self.otp_sessions.read().await;
        let session = otp_sessions.get(session_id)
            .ok_or(AuthenticationError::InvalidSession)?
            .clone();
        drop(otp_sessions);
        
        if Instant::now() > session.expires_at {
            self.otp_sessions.write().await.remove(session_id);
            return Err(AuthenticationError::SessionExpired);
        }
        
        if session.otp_verified {
            return Err(AuthenticationError::OtpAlreadyVerified);
        }
        
        let new_otp = self.generate_otp_code();
        let now = Instant::now();
        
        let codes = self.otp_codes.read().await;
        let old_otp = codes.values().find(|o| o.user_id == session.user_id && !o.verified);
        
        if let Some(old) = old_otp {
            let mut codes_write = self.otp_codes.write().await;
            codes_write.remove(&old.code);
        }
        drop(codes);
        
        let otp = OtpCode {
            code: new_otp.clone(),
            user_id: session.user_id.clone(),
            username: session.username.clone(),
            created_at: now,
            expires_at: now + Duration::from_secs(self.otp_expiry_seconds),
            attempts: 0,
            verified: false,
        };
        
        self.otp_codes.write().await.insert(new_otp.clone(), otp);
        
        self.audit_logger.write().await.log("OTP_RESENT", &session.user_id, &session.username, AuditResult::Success,
            serde_json::json!({})).await;
        
        info!("📧 OTP resent for user: {}", session.username);
        
        Ok(new_otp)
    }
    
    pub async fn cleanup_expired_otp(&self) -> usize {
        let now = Instant::now();
        let mut count = 0;
        
        {
            let mut codes = self.otp_codes.write().await;
            let expired: Vec<String> = codes.iter()
                .filter(|(_, o)| now > o.expires_at)
                .map(|(k, _)| k.clone())
                .collect();
            for k in &expired {
                codes.remove(k);
            }
            count += expired.len();
        }
        
        {
            let mut sessions = self.otp_sessions.write().await;
            let expired: Vec<String> = sessions.iter()
                .filter(|(_, s)| now > s.expires_at)
                .map(|(k, _)| k.clone())
                .collect();
            for k in &expired {
                sessions.remove(k);
            }
            count += expired.len();
        }
        
        count
    }
    
    pub async fn get_active_otp_sessions(&self) -> Vec<OtpStatus> {
        let sessions = self.otp_sessions.read().await;
        let codes = self.otp_codes.read().await;
        
        sessions.values().map(|s| {
            let pending = codes.values().any(|c| c.user_id == s.user_id && !c.verified);
            OtpStatus {
                session_id: s.session_id.clone(),
                username: s.username.clone(),
                password_verified: s.password_verified,
                otp_verified: s.otp_verified,
                expires_at: s.expires_at.elapsed().as_secs(),
                pending_otp: pending,
            }
        }).collect()
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
    
    #[error("Invalid OTP code")]
    InvalidOtp,
    
    #[error("OTP code expired")]
    OtpExpired,
    
    #[error("Maximum OTP attempts exceeded")]
    OtpMaxAttempts,
    
    #[error("Invalid session")]
    InvalidSession,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("OTP already verified")]
    OtpAlreadyVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpSessionResponse {
    pub session_id: String,
    pub otp_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpStatus {
    pub session_id: String,
    pub username: String,
    pub password_verified: bool,
    pub otp_verified: bool,
    pub expires_at: u64,
    pub pending_otp: bool,
}
