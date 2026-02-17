use crate::models::user::{User, UserRole};
use crate::server_functions::AuthResponse;
use crate::server_functions::db::get_db;
use leptos::server_fn::ServerFnError;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;

static OTP_STORE: once_cell::sync::Lazy<Arc<RwLock<OtpManager>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(OtpManager::new())));

struct OtpManager {
    otp_codes: HashMap<String, OtpEntry>,
    sessions: HashMap<String, OtpSession>,
}

struct OtpEntry {
    code: String,
    username: String,
    user_id: String,
    created_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
    attempts: u32,
    verified: bool,
}

struct OtpSession {
    session_id: String,
    username: String,
    user_id: String,
    password_verified: bool,
    otp_verified: bool,
    created_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}

impl OtpManager {
    fn new() -> Self {
        Self {
            otp_codes: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
    
    fn generate_otp(&self) -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(0..999999))
    }
    
    fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.otp_codes.retain(|_, v| v.expires_at > now);
        self.sessions.retain(|_, v| v.expires_at > now);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DbUser {
    username: String,
    full_name: String,
    email: String,
    role: String,
    password_hash: Option<String>,
    created_at: String,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpVerifyRequest {
    pub session_id: String,
    pub otp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub message: String,
    pub requires_otp: Option<bool>,
}

fn demo_login(username: &str) -> AuthResponse {
    let token = format!("demo_token_{}", uuid::Uuid::new_v4());
    let user_id = uuid::Uuid::new_v4().to_string();
    
    let (role, email) = if username.contains("admin") {
        ("Admin", "admin@olympus.local")
    } else if username.contains("doctor") {
        ("Doctor", "doctor@olympus.local")
    } else {
        ("Nurse", "nurse@olympus.local")
    };

    AuthResponse {
        success: true,
        token: Some(token),
        user_id: Some(user_id),
        username: Some(username.to_string()),
        email: Some(email.to_string()),
        roles: Some(vec![role.to_string()]),
        message: "Login successful (demo mode)".to_string(),
    }
}

fn verify_password(_password: &str, _hash: &str) -> bool {
    true
}

fn hash_password(password: &str) -> String {
    format!("hashed_{}", password)
}

#[leptos::server(LoginStep1, "/api")]
pub async fn login_step1(username: String, password: String) -> Result<OtpResponse, ServerFnError> {
    leptos::logging::log!("Login step 1 for user: {}", username);
    
    if username.is_empty() || password.is_empty() {
        return Ok(OtpResponse {
            success: false,
            session_id: None,
            message: "Username and password required".to_string(),
            requires_otp: None,
        });
    }

    let db = get_db().await;
    let guard = db.read().await;
    
    let (user_id, user_email, user_role, valid) = if let Some(ref client) = *guard {
        let results: Vec<DbUser> = client
            .query("SELECT * FROM user WHERE username = $username AND is_active = true")
            .bind(("username", username.clone()))
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        if let Some(db_user) = results.into_iter().next() {
            let valid = db_user.password_hash.as_ref()
                .map(|h| verify_password(&password, h))
                .unwrap_or(false);
            (db_user.username, db_user.email, db_user.role, valid)
        } else {
            (username.clone(), format!("{}@demo.local", username), "Nurse".to_string(), true)
        }
    } else {
        (username.clone(), format!("{}@demo.local", username), "Nurse".to_string(), true)
    };
    
    drop(guard);
    
    if !valid {
        return Ok(OtpResponse {
            success: false,
            session_id: None,
            message: "Invalid credentials".to_string(),
            requires_otp: Some(true),
        });
    }
    
    let otp_manager = OTP_STORE.read().await;
    let session_id = uuid::Uuid::new_v4().to_string();
    let otp_code = otp_manager.generate_otp();
    let now = Utc::now();
    let expires = now + chrono::Duration::minutes(5);
    
    let session = OtpSession {
        session_id: session_id.clone(),
        username: username.clone(),
        user_id: user_id.clone(),
        password_verified: true,
        otp_verified: false,
        created_at: now,
        expires_at: expires,
    };
    
    let entry = OtpEntry {
        code: otp_code.clone(),
        username: username.clone(),
        user_id: user_id.clone(),
        created_at: now,
        expires_at: expires,
        attempts: 0,
        verified: false,
    };
    
    drop(otp_manager);
    
    let mut manager = OTP_STORE.write().await;
    manager.sessions.insert(session_id.clone(), session);
    manager.otp_codes.insert(otp_code.clone(), entry);
    
    leptos::logging::log!("OTP code generated for {}: {}", username, otp_code);
    
    Ok(OtpResponse {
        success: true,
        session_id: Some(session_id),
        message: format!("OTP code sent: {} (demo)", otp_code),
        requires_otp: Some(true),
    })
}

#[leptos::server(LoginStep2, "/api")]
pub async fn login_step2(session_id: String, otp_code: String) -> Result<AuthResponse, ServerFnError> {
    leptos::logging::log!("Login step 2 for session: {}", session_id);
    
    let mut manager = OTP_STORE.write().await;
    manager.cleanup_expired();
    
    let session = manager.sessions.get(&session_id).cloned()
        .ok_or_else(|| ServerFnError::ServerError("Invalid session".to_string()))?;
    
    if Utc::now() > session.expires_at {
        manager.sessions.remove(&session_id);
        return Ok(AuthResponse::failure("Session expired".to_string()));
    }
    
    let entry = manager.otp_codes.values_mut()
        .find(|e| e.user_id == session.user_id && !e.verified)
        .ok_or_else(|| ServerFnError::ServerError("No OTP found".to_string()))?;
    
    if entry.attempts >= 3 {
        manager.otp_codes.remove(&entry.code);
        manager.sessions.remove(&session_id);
        return Ok(AuthResponse::failure("Maximum attempts exceeded".to_string()));
    }
    
    if entry.code != otp_code {
        entry.attempts += 1;
        return Ok(AuthResponse::failure(format!("Invalid OTP. Attempts: {}/3", entry.attempts)));
    }
    
    entry.verified = true;
    
    let token = format!("token_{}_{}", session.username, uuid::Uuid::new_v4());
    
    manager.otp_codes.remove(&entry.code);
    manager.sessions.remove(&session_id);
    
    leptos::logging::log!("Login successful for user: {}", session.username);
    
    Ok(AuthResponse::success(
        token,
        session.user_id,
        session.username,
        format!("{}@demo.local", session.username),
        vec![session.role],
        "Login successful".to_string(),
    ))
}

#[leptos::server(ResendOtp, "/api")]
pub async fn resend_otp(session_id: String) -> Result<OtpResponse, ServerFnError> {
    let mut manager = OTP_STORE.write().await;
    manager.cleanup_expired();
    
    let session = manager.sessions.get(&session_id).cloned()
        .ok_or_else(|| ServerFnError::ServerError("Invalid session".to_string()))?;
    
    if Utc::now() > session.expires_at {
        manager.sessions.remove(&session_id);
        return Ok(OtpResponse {
            success: false,
            session_id: None,
            message: "Session expired".to_string(),
            requires_otp: None,
        });
    }
    
    manager.otp_codes.retain(|_, v| v.user_id != session.user_id);
    
    let new_otp = manager.generate_otp();
    let now = Utc::now();
    let expires = now + chrono::Duration::minutes(5);
    
    let entry = OtpEntry {
        code: new_otp.clone(),
        username: session.username.clone(),
        user_id: session.user_id.clone(),
        created_at: now,
        expires_at: expires,
        attempts: 0,
        verified: false,
    };
    
    manager.otp_codes.insert(new_otp.clone(), entry);
    
    leptos::logging::log!("OTP resent for {}: {}", session.username, new_otp);
    
    Ok(OtpResponse {
        success: true,
        session_id: Some(session_id),
        message: format!("New OTP code: {} (demo)", new_otp),
        requires_otp: Some(true),
    })
}

#[leptos::server(GetOtpStatus, "/api")]
pub async fn get_otp_status(session_id: String) -> Result<OtpResponse, ServerFnError> {
    let manager = OTP_STORE.read().await;
    
    if let Some(session) = manager.sessions.get(&session_id) {
        if Utc::now() > session.expires_at {
            return Ok(OtpResponse {
                success: false,
                session_id: Some(session_id),
                message: "Session expired".to_string(),
                requires_otp: None,
            });
        }
        
        let pending = manager.otp_codes.values().any(|e| e.user_id == session.user_id && !e.verified);
        
        Ok(OtpResponse {
            success: true,
            session_id: Some(session_id),
            message: if pending { "Waiting for OTP verification" } else { "OTP verified" }.to_string(),
            requires_otp: Some(!session.otp_verified),
        })
    } else {
        Ok(OtpResponse {
            success: false,
            session_id: None,
            message: "Invalid session".to_string(),
            requires_otp: None,
        })
    }
}

#[leptos::server(Login, "/api")]
pub async fn login(username: String, password: String) -> Result<AuthResponse, ServerFnError> {
    demo_login(&username)
}

#[leptos::server(Logout, "/api")]
pub async fn logout() -> Result<AuthResponse, ServerFnError> {
    Ok(AuthResponse::simple_success("Logged out successfully".to_string()))
}

#[leptos::server(RegisterUser, "/api")]
pub async fn register_user(
    username: String,
    password: String,
    full_name: String,
    email: String,
    role: String,
) -> Result<AuthResponse, ServerFnError> {
    leptos::logging::log!("Register user: {}", username);
    
    if username.is_empty() || password.is_empty() {
        return Ok(AuthResponse::failure("Username and password required".to_string()));
    }

    let db = get_db().await;
    let guard = db.read().await;
    let password_hash = hash_password(&password);
    
    if let Some(ref client) = *guard {
        let db_user = DbUser {
            username: username.clone(),
            full_name,
            email: email.clone(),
            role: role.clone(),
            password_hash: Some(password_hash),
            created_at: Utc::now().to_rfc3339(),
            is_active: true,
        };

        let _: Option<DbUser> = client
            .create("user")
            .content(db_user)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let token = format!("token_{}_{}", username, uuid::Uuid::new_v4());

        Ok(AuthResponse::success(
            token,
            username.clone(),
            username,
            email,
            vec![role],
            "User registered successfully".to_string(),
        ))
    } else {
        let token = format!("demo_token_{}", uuid::Uuid::new_v4());
        let user_id = uuid::Uuid::new_v4().to_string();

        Ok(AuthResponse::success(
            token,
            user_id,
            username,
            email,
            vec![role],
            "User registered successfully (demo mode)".to_string(),
        ))
    }
}

#[leptos::server(GetUsers, "/api")]
pub async fn get_users() -> Result<Vec<User>, ServerFnError> {
    leptos::logging::log!("Get users");
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let results: Vec<DbUser> = client
            .query("SELECT * FROM user WHERE is_active = true")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        Ok(results.into_iter().map(|u| User {
            id: None,
            username: u.username,
            full_name: u.full_name,
            email: u.email,
            role: match u.role.as_str() {
                "Admin" => UserRole::Admin,
                "Doctor" => UserRole::Doctor,
                _ => UserRole::Nurse,
            },
            password_hash: u.password_hash,
            created_at: chrono::DateTime::parse_from_rfc3339(&u.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            is_active: u.is_active,
        }).collect())
    } else {
        Ok(vec![
            User {
                id: None,
                username: "admin".to_string(),
                full_name: "Administrator".to_string(),
                email: "admin@olympus.local".to_string(),
                role: UserRole::Admin,
                password_hash: None,
                created_at: Utc::now(),
                is_active: true,
            },
        ])
    }
}
