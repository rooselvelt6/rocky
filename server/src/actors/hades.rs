// server/src/actors/hades.rs
// Hades: Seguridad, Autenticación y Cifrado

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;

pub struct Hades {
    jwt_secret: String,
    active_sessions: Vec<String>,
    messages_count: u64,
}

impl Hades {
    pub fn new() -> Self {
        Self {
            jwt_secret: "olympus_secret_key_2026".to_string(),
            active_sessions: Vec::new(),
            messages_count: 0,
        }
    }

    fn validate_credentials(&self, username: &str, password: &str) -> bool {
        username == "admin" && password == "admin123"
    }

    fn generate_token(&self, username: &str) -> String {
        // Simulación simple de JWT
        format!("jwt_{}_{}_{}", username, Utc::now().timestamp(), self.jwt_secret.chars().take(8).collect::<String>())
    }

    fn validate_otp(&self, code: &str) -> bool {
        code == "123456"
    }

    fn create_session(&mut self, username: &str) -> String {
        let session = format!("session_{}_{}", username, Utc::now().timestamp());
        self.active_sessions.push(session.clone());
        session
    }
}

#[async_trait]
impl OlympianActor for Hades {
    fn name(&self) -> GodName {
        GodName::Hades
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;

        match &msg.payload {
            MessagePayload::Command { action, data } => {
                match action.as_str() {
                    "authenticate" => {
                        let username = data.get("username")?.as_str()?;
                        let password = data.get("password")?.as_str()?;
                        
                        if self.validate_credentials(username, password) {
                            let session = self.create_session(username);
                            Some(ActorMessage::new(
                                GodName::Hades,
                                msg.from,
                                MessagePayload::Response {
                                    success: true,
                                    data: serde_json::json!({
                                        "requires_otp": true,
                                        "session_id": session,
                                        "message": "Código OTP enviado: 123456"
                                    }),
                                    error: None,
                                }
                            ))
                        } else {
                            Some(ActorMessage::new(
                                GodName::Hades,
                                msg.from,
                                MessagePayload::Response {
                                    success: false,
                                    data: serde_json::json!({}),
                                    error: Some("Credenciales inválidas".to_string()),
                                }
                            ))
                        }
                    }

                    "verify_otp" => {
                        let code = data.get("otp_code")?.as_str()?;
                        let username = data.get("username")?.as_str()?;
                        
                        if self.validate_otp(code) {
                            let token = self.generate_token(username);
                            Some(ActorMessage::new(
                                GodName::Hades,
                                msg.from,
                                MessagePayload::Response {
                                    success: true,
                                    data: serde_json::json!({
                                        "token": token,
                                        "username": username,
                                        "message": "¡Zeus aprueba tu acceso!"
                                    }),
                                    error: None,
                                }
                            ))
                        } else {
                            Some(ActorMessage::new(
                                GodName::Hades,
                                msg.from,
                                MessagePayload::Response {
                                    success: false,
                                    data: serde_json::json!({}),
                                    error: Some("Código OTP inválido".to_string()),
                                }
                            ))
                        }
                    }

                    "logout" => {
                        Some(ActorMessage::new(
                            GodName::Hades,
                            msg.from,
                            MessagePayload::Response {
                                success: true,
                                data: serde_json::json!({
                                    "message": "Sesión cerrada - Hades protege tu salida"
                                }),
                                error: None,
                            }
                        ))
                    }

                    _ => None
                }
            }

            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Hades,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: "Protecting".to_string(),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("🔒 Hades: Inicializando seguridad...");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🔒 Hades: Cerrando sesiones activas...");
        Ok(())
    }
}
