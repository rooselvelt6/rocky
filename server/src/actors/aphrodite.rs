// server/src/actors/aphrodite.rs
// Aphrodite: Diosa de la Belleza, UI/UX y Temas
// Gestiona la apariencia del sistema de forma dinámica

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;
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
    available_themes: Vec<Theme>,
    components: HashMap<String, Component>,
    messages_count: u64,
    theme_changes: u64,
}

impl Aphrodite {
    pub fn new() -> Self {
        let mut themes = Vec::new();
        
        // Tema por defecto (Olympus Dark)
        themes.push(Theme::default());
        
        // Tema Claro
        themes.push(Theme {
            name: "Olympus Light".to_string(),
            primary_color: "#4f46e5".to_string(),
            secondary_color: "#7c3aed".to_string(),
            background: "#f8fafc".to_string(),
            surface: "#ffffff".to_string(),
            text_primary: "#0f172a".to_string(),
            text_secondary: "#64748b".to_string(),
            accent: "#f59e0b".to_string(),
            success: "#10b981".to_string(),
            warning: "#f59e0b".to_string(),
            error: "#ef4444".to_string(),
            border_radius: "0.75rem".to_string(),
            font_family: "Inter, system-ui, sans-serif".to_string(),
        });
        
        // Tema Dorado (Oro del Olimpo)
        themes.push(Theme {
            name: "Golden Olympus".to_string(),
            primary_color: "#fbbf24".to_string(),
            secondary_color: "#f59e0b".to_string(),
            background: "#1c1917".to_string(),
            surface: "#292524".to_string(),
            text_primary: "#fafaf9".to_string(),
            text_secondary: "#a8a29e".to_string(),
            accent: "#fcd34d".to_string(),
            success: "#34d399".to_string(),
            warning: "#fbbf24".to_string(),
            error: "#f87171".to_string(),
            border_radius: "1rem".to_string(),
            font_family: "Georgia, serif".to_string(),
        });
        
        // Tema Cósmico
        themes.push(Theme {
            name: "Cosmic".to_string(),
            primary_color: "#06b6d4".to_string(),
            secondary_color: "#8b5cf6".to_string(),
            background: "#020617".to_string(),
            surface: "#0f172a".to_string(),
            text_primary: "#e2e8f0".to_string(),
            text_secondary: "#64748b".to_string(),
            accent: "#22d3ee".to_string(),
            success: "#34d399".to_string(),
            warning: "#fbbf24".to_string(),
            error: "#f472b6".to_string(),
            border_radius: "0.5rem".to_string(),
            font_family: "SF Mono, monospace".to_string(),
        });
        
        let mut components = HashMap::new();
        
        // Componentes base
        components.insert("button".to_string(), Component {
            id: "button".to_string(),
            name: "Botón".to_string(),
            component_type: "button".to_string(),
            styles: [
                ("padding".to_string(), "0.75rem 1.5rem".to_string()),
                ("borderRadius".to_string(), "0.5rem".to_string()),
                ("fontWeight".to_string(), "600".to_string()),
                ("transition".to_string(), "all 0.2s".to_string()),
            ].into_iter().collect(),
            active: true,
        });
        
        components.insert("card".to_string(), Component {
            id: "card".to_string(),
            name: "Tarjeta".to_string(),
            component_type: "card".to_string(),
            styles: [
                ("padding".to_string(), "1.5rem".to_string()),
                ("borderRadius".to_string(), "0.75rem".to_string()),
                ("borderWidth".to_string(), "1px".to_string()),
                ("boxShadow".to_string(), "0 4px 6px -1px rgba(0,0,0,0.1)".to_string()),
            ].into_iter().collect(),
            active: true,
        });
        
        components.insert("nav".to_string(), Component {
            id: "nav".to_string(),
            name: "Navegación".to_string(),
            component_type: "nav".to_string(),
            styles: [
                ("padding".to_string(), "1rem".to_string()),
                ("position".to_string(), "sticky".to_string()),
                ("top".to_string(), "0".to_string()),
                ("zIndex".to_string(), "50".to_string()),
            ].into_iter().collect(),
            active: true,
        });
        
        Self {
            current_theme: themes[0].clone(),
            available_themes: themes,
            components,
            messages_count: 0,
            theme_changes: 0,
        }
    }
    
    fn switch_theme(&mut self, theme_name: &str) -> Result<Theme, String> {
        if let Some(theme) = self.available_themes.iter().find(|t| t.name == theme_name) {
            self.current_theme = theme.clone();
            self.theme_changes += 1;
            tracing::info!("🎨 Aphrodite: Cambiado a tema '{}'", theme_name);
            Ok(theme.clone())
        } else {
            Err(format!("Tema '{}' no encontrado", theme_name))
        }
    }
    
    fn get_current_theme(&self) -> Theme {
        self.current_theme.clone()
    }
    
    fn get_all_themes(&self) -> Vec<String> {
        self.available_themes.iter().map(|t| t.name.clone()).collect()
    }
    
    fn update_component_style(&mut self, component_id: &str, style_key: &str, style_value: &str) -> Result<(), String> {
        if let Some(component) = self.components.get_mut(component_id) {
            component.styles.insert(style_key.to_string(), style_value.to_string());
            tracing::info!("🎨 Aphrodite: Actualizado {}.{} = {}", component_id, style_key, style_value);
            Ok(())
        } else {
            Err(format!("Componente '{}' no encontrado", component_id))
        }
    }
    
    fn get_component_styles(&self, component_id: &str) -> Option<HashMap<String, String>> {
        self.components.get(component_id).map(|c| c.styles.clone())
    }
    
    fn generate_css_variables(&self) -> String {
        format!(
            r#":root {{
  --color-primary: {};
  --color-secondary: {};
  --color-background: {};
  --color-surface: {};
  --color-text-primary: {};
  --color-text-secondary: {};
  --color-accent: {};
  --color-success: {};
  --color-warning: {};
  --color-error: {};
  --border-radius: {};
  --font-family: {};
}}"#,
            self.current_theme.primary_color,
            self.current_theme.secondary_color,
            self.current_theme.background,
            self.current_theme.surface,
            self.current_theme.text_primary,
            self.current_theme.text_secondary,
            self.current_theme.accent,
            self.current_theme.success,
            self.current_theme.warning,
            self.current_theme.error,
            self.current_theme.border_radius,
            self.current_theme.font_family,
        )
    }
}

#[async_trait]
impl OlympianActor for Aphrodite {
    fn name(&self) -> GodName {
        GodName::Aphrodite
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;
        tracing::debug!("🎨 Aphrodite recibió mensaje de {:?}", msg.from);

        match &msg.payload {
            MessagePayload::Query { query_type, .. } => {
                let response_data = match query_type.as_str() {
                    "get_current_theme" => {
                        serde_json::json!({
                            "theme": self.get_current_theme(),
                            "css_variables": self.generate_css_variables(),
                        })
                    }
                    
                    "get_all_themes" => {
                        serde_json::json!({
                            "themes": self.get_all_themes(),
                            "current": self.current_theme.name,
                        })
                    }
                    
                    "get_component_styles" => {
                        // Extraer component_id de params
                        serde_json::json!({
                            "components": self.components.values().collect::<Vec<_>>(),
                        })
                    }
                    
                    "get_css_variables" => {
                        serde_json::json!({
                            "css": self.generate_css_variables(),
                        })
                    }
                    
                    _ => return None,
                };

                Some(ActorMessage::new(
                    GodName::Aphrodite,
                    msg.from,
                    MessagePayload::Response {
                        success: true,
                        data: response_data,
                        error: None,
                    }
                ))
            }

            MessagePayload::Command { action, data } => {
                let result = match action.as_str() {
                    "switch_theme" => {
                        if let Some(theme_name) = data.get("theme_name").and_then(|v| v.as_str()) {
                            match self.switch_theme(theme_name) {
                                Ok(theme) => serde_json::json!({
                                    "success": true,
                                    "theme": theme,
                                    "message": format!("Tema cambiado a {}", theme_name),
                                }),
                                Err(e) => serde_json::json!({
                                    "success": false,
                                    "error": e,
                                }),
                            }
                        } else {
                            serde_json::json!({
                                "success": false,
                                "error": "theme_name requerido",
                            })
                        }
                    }
                    
                    "update_component_style" => {
                        let component_id = data.get("component_id").and_then(|v| v.as_str());
                        let style_key = data.get("style_key").and_then(|v| v.as_str());
                        let style_value = data.get("style_value").and_then(|v| v.as_str());
                        
                        if let (Some(cid), Some(key), Some(val)) = (component_id, style_key, style_value) {
                            match self.update_component_style(cid, key, val) {
                                Ok(_) => serde_json::json!({
                                    "success": true,
                                    "message": "Estilo actualizado",
                                }),
                                Err(e) => serde_json::json!({
                                    "success": false,
                                    "error": e,
                                }),
                            }
                        } else {
                            serde_json::json!({
                                "success": false,
                                "error": "Parámetros incompletos",
                            })
                        }
                    }
                    
                    "create_custom_theme" => {
                        // Crear tema personalizado desde datos
                        serde_json::json!({
                            "success": true,
                            "message": "Tema personalizado creado (demo)",
                        })
                    }
                    
                    _ => return None,
                };

                Some(ActorMessage::new(
                    GodName::Aphrodite,
                    msg.from,
                    MessagePayload::Response {
                        success: true,
                        data: result,
                        error: None,
                    }
                ))
            }

            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Aphrodite,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Designing - {} theme changes", self.theme_changes),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("🎨 Aphrodite: Inicializando sistema de belleza...");
        tracing::info!("🎨 Aphrodite: {} temas disponibles", self.available_themes.len());
        tracing::info!("🎨 Aphrodite: {} componentes registrados", self.components.len());
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🎨 Aphrodite: {} cambios de tema realizados", self.theme_changes);
        Ok(())
    }
}
