// server/src/actors/aphrodite.rs
// Aphrodite: Diosa de la Belleza, UI/UX y Temas
// Gestiona la apariencia del sistema de forma dinámica

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use ractor::{Actor, ActorRef, ActorProcessingErr};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background: String,
    pub surface: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub border_radius: String,
    pub font_family: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Olympus Dark".to_string(),
            primary_color: "#6366f1".to_string(), // Indigo
            secondary_color: "#8b5cf6".to_string(), // Purple
            background: "#0f172a".to_string(), // Slate 900
            surface: "#1e293b".to_string(), // Slate 800
            text_primary: "#f8fafc".to_string(), // Slate 50
            text_secondary: "#94a3b8".to_string(), // Slate 400
            accent: "#f59e0b".to_string(), // Amber
            success: "#10b981".to_string(), // Emerald
            warning: "#f59e0b".to_string(), // Amber
            error: "#ef4444".to_string(), // Red
            border_radius: "0.75rem".to_string(),
            font_family: "Inter, system-ui, sans-serif".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub component_type: String, // "button", "card", "input", "nav", etc.
    pub styles: HashMap<String, String>,
    pub active: bool,
}

pub struct Aphrodite {
    current_theme: Theme,
}

impl Aphrodite {
    pub fn new() -> Self {
        Self { current_theme: Theme::default() }
    }
}

impl Actor for Aphrodite {
    type Msg = ActorMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("🎨 Aphrodite v16: Estética del Olimpo cargada (Ractor).");
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, _state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match msg.payload {
            MessagePayload::Query { query_type, reply, .. } => {
                if let Some(port) = reply {
                    if query_type == "get_current_theme" {
                        let _ = port.send(MessagePayload::Response {
                            success: true,
                            data: serde_json::to_value(&self.current_theme).unwrap_or_default(),
                            error: None,
                        });
                    }
                }
            }
            MessagePayload::Command { action, .. } => {
                tracing::debug!("🎨 Aphrodite procesando estilo: {}", action);
            }
            _ => {}
        }
        Ok(())
    }
}

impl OlympianActor for Aphrodite {
    fn name(&self) -> GodName {
        GodName::Aphrodite
    }

    async fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🎨 Aphrodite: Sesión de diseño cerrada.");
        Ok(())
    }
}
