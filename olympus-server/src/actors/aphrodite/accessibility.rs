// src/actors/aphrodite/accessibility.rs
// OLYMPUS v15 - Aphrodite: Sistema de Accesibilidad

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sistema de accesibilidad completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySystem {
    pub screen_reader_enabled: bool,
    pub high_contrast_mode: bool,
    pub reduced_motion: bool,
    pub keyboard_navigation: bool,
    pub focus_indicators: bool,
    pub color_blind_mode: Option<ColorBlindType>,
    pub text_scaling: f64,
    pub aria_labels: HashMap<String, String>,
}

/// Tipos de daltonismo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorBlindType {
    Protanopia,    // Rojo
    Deuteranopia,  // Verde
    Tritanopia,    // Azul
    Achromatopsia, // Monocromático
}

/// Configuración de ARIA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AriaConfig {
    pub role: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub expanded: Option<bool>,
    pub hidden: Option<bool>,
    pub disabled: Option<bool>,
    pub required: Option<bool>,
    pub selected: Option<bool>,
    pub pressed: Option<bool>,
    pub checked: Option<bool>,
    pub level: Option<u8>,
    pub live: Option<String>,
    pub atomic: Option<bool>,
    pub relevant: Option<String>,
}

/// Configuración de navegación por teclado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub tab_index: i32,
    pub auto_focus: bool,
    pub keyboard_shortcuts: HashMap<String, String>,
    pub focus_trap: bool,
    pub escape_closes: bool,
}

/// Configuración de contraste
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastConfig {
    pub min_contrast_ratio: f64,
    pub high_contrast: bool,
    pub forced_colors: bool,
}

impl Default for AccessibilitySystem {
    fn default() -> Self {
        Self {
            screen_reader_enabled: false,
            high_contrast_mode: false,
            reduced_motion: false,
            keyboard_navigation: true,
            focus_indicators: true,
            color_blind_mode: None,
            text_scaling: 1.0,
            aria_labels: HashMap::new(),
        }
    }
}

impl AccessibilitySystem {
    /// Crea un nuevo sistema de accesibilidad
    pub fn new() -> Self {
        Self::default()
    }

    /// Activa el modo de alto contraste
    pub fn enable_high_contrast(&mut self) {
        self.high_contrast_mode = true;
    }

    /// Desactiva el modo de alto contraste
    pub fn disable_high_contrast(&mut self) {
        self.high_contrast_mode = false;
    }

    /// Activa el modo de movimiento reducido
    pub fn enable_reduced_motion(&mut self) {
        self.reduced_motion = true;
    }

    /// Desactiva el modo de movimiento reducido
    pub fn disable_reduced_motion(&mut self) {
        self.reduced_motion = false;
    }

    /// Establece el tipo de daltonismo
    pub fn set_color_blind_mode(&mut self, color_blind_type: ColorBlindType) {
        self.color_blind_mode = Some(color_blind_type);
    }

    /// Elimina el modo daltonismo
    pub fn clear_color_blind_mode(&mut self) {
        self.color_blind_mode = None;
    }

    /// Escala el texto
    pub fn set_text_scale(&mut self, scale: f64) {
        self.text_scaling = scale.clamp(0.5, 3.0);
    }

    /// Agrega una etiqueta ARIA
    pub fn add_aria_label(&mut self, id: &str, label: &str) {
        self.aria_labels.insert(id.to_string(), label.to_string());
    }

    /// Verifica si cumple con WCAG 2.1 AA
    pub fn meets_wcag_aa(&self) -> bool {
        // En una implementación real, esto verificaría contraste, navegación, etc.
        self.keyboard_navigation && self.focus_indicators
    }

    /// Verifica si cumple con WCAG 2.1 AAA
    pub fn meets_wcag_aaa(&self) -> bool {
        // Nivel AAA es más estricto
        self.meets_wcag_aa() && self.text_scaling >= 1.0
    }
}

impl Default for AriaConfig {
    fn default() -> Self {
        Self {
            role: "generic".to_string(),
            label: None,
            description: None,
            expanded: None,
            hidden: None,
            disabled: None,
            required: None,
            selected: None,
            pressed: None,
            checked: None,
            level: None,
            live: None,
            atomic: None,
            relevant: None,
        }
    }
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            tab_index: 0,
            auto_focus: false,
            keyboard_shortcuts: HashMap::new(),
            focus_trap: false,
            escape_closes: true,
        }
    }
}

impl Default for ContrastConfig {
    fn default() -> Self {
        Self {
            min_contrast_ratio: 4.5, // WCAG AA standard
            high_contrast: false,
            forced_colors: false,
        }
    }
}
