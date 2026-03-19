// server/src/actors/hades.rs
// Hades: Seguridad, Autenticación y Cifrado

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use chrono::Utc;
use secrecy::{SecretString, ExposeSecret};
use zeroize::Zeroize;
use ractor::{Actor, ActorRef, ActorProcessingErr};

pub struct Hades {
    jwt_secret: SecretString,
    active_sessions: Vec<String>,
}

#[derive(Zeroize)]
struct AuthBuffer {
    username: String,
    password: String,
}

impl Hades {
    pub fn new() -> Self {
        Self {
            jwt_secret: SecretString::from("olympus_secret_key_2026".to_string()),
            active_sessions: Vec::new(),
        }
    }

    fn validate_credentials(&self, username: &str, password: &str) -> bool {
        let mut buffer = AuthBuffer {
            username: username.to_string(),
            password: password.to_string(),
        };
        let is_valid = buffer.username == "admin" && buffer.password == "admin123";
        buffer.zeroize();
        is_valid
    }

    fn generate_token(&self, username: &str) -> String {
        format!("jwt_{}_{}_{}", username, Utc::now().timestamp(), self.jwt_secret.expose_secret().chars().take(8).collect::<String>())
    }

    fn validate_otp(&self, code: &str) -> bool {
        let mut code_buffer = code.to_string();
        let is_valid = code_buffer == "123456";
        code_buffer.zeroize();
        is_valid
    }

    fn create_session(&mut self, username: &str) -> String {
        let session = format!("session_{}_{}", username, Utc::now().timestamp());
        self.active_sessions.push(session.clone());
        session
    }
}

impl Actor for Hades {
    type Msg = ActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("🔒 Hades v16: Escudo de seguridad activo y Zeroize cargado.");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, _state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match msg.payload {
            MessagePayload::Command { action, data, reply } => {
                tracing::info!("🔒 Hades procesando comando: {}", action);
                
                if let Some(port) = reply {
                    match action.as_str() {
                        "authenticate" => {
                            let username = data["username"].as_str().unwrap_or_default();
                            let password = data["password"].as_str().unwrap_or_default();
                            
                            let success = self.validate_credentials(username, password);
                            let _ = port.send(MessagePayload::Response {
                                success,
                                data: if success { 
                                    serde_json::json!({ "username": username, "message": "OTP enviado (Simulado)" }) 
                                } else { 
                                    serde_json::json!({ "message": "Credenciales inválidas" }) 
                                },
                                error: None,
                            });
                        }
                        "verify_otp" => {
                            let code = data["otp_code"].as_str().unwrap_or_default();
                            let username = data["username"].as_str().unwrap_or_default();
                            
                            let success = self.validate_otp(code);
                            let _ = port.send(MessagePayload::Response {
                                success,
                                data: if success {
                                    serde_json::json!({ "token": self.generate_token(username), "username": username })
                                } else {
                                    serde_json::json!({ "message": "Código OTP inválido" })
                                },
                                error: None,
                            });
                        }
                        _ => {
                            let _ = port.send(MessagePayload::Response {
                                success: false,
                                data: serde_json::json!({}),
                                error: Some("Acción no soportada por Hades".to_string()),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl OlympianActor for Hades {
    fn name(&self) -> GodName {
        GodName::Hades
    }

    async fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🔒 Hades: Seguridad cerrada.");
        Ok(())
    }
}
