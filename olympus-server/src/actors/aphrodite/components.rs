// src/actors/aphrodite/components.rs
// OLYMPUS v15 - Aphrodite: Componentes UI/UX

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Componente UI base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub id: String,
    pub name: String,
    pub component_type: ComponentType,
    pub props: serde_json::Value,
    pub children: Vec<UIComponent>,
    pub style: ComponentStyle,
    pub events: Vec<UIEvent>,
}

/// Tipos de componentes UI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    // Layout
    Container,
    Grid,
    Flex,
    Stack,

    // Content
    Text,
    Image,
    Icon,
    Button,
    Input,
    Select,
    Checkbox,
    Radio,

    // Navigation
    Navbar,
    Sidebar,
    Menu,
    Breadcrumb,
    Tab,

    // Data Display
    Table,
    List,
    Card,
    Badge,
    Avatar,
    Progress,

    // Feedback
    Alert,
    Modal,
    Tooltip,
    Loading,
    Error,

    // Charts
    LineChart,
    BarChart,
    PieChart,
    RadarChart,
    Sparkline,

    // Forms
    Form,
    Field,
    Label,
    TextArea,
    DatePicker,
    TimePicker,

    // Specialized
    Dashboard,
    MetricCard,
    StatusIndicator,
    CommandButton,
    ControlPanel,
}

/// Estilos de componente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStyle {
    pub theme: ThemeVariant,
    pub size: ComponentSize,
    pub color_scheme: ColorScheme,
    pub spacing: SpacingConfig,
    pub typography: TypographyConfig,
    pub animations: AnimationConfig,
    pub responsive: ResponsiveConfig,
}

/// Variantes de tema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    Light,
    Dark,
    Auto,
    Custom(String),
}

/// Tamaños de componentes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentSize {
    XS,
    S,
    M,
    L,
    XL,
    XXL,
    Custom(String),
}

/// Esquemas de color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub background: String,
    pub surface: String,
    pub text: String,
    pub text_secondary: String,
}

/// Configuración de espaciado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingConfig {
    pub xs: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xxl: String,
}

/// Configuración tipográfica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub font_family: String,
    pub font_size: String,
    pub font_weight: String,
    pub line_height: String,
    pub letter_spacing: Option<String>,
}

/// Configuración de animaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub duration: String,
    pub easing: String,
    pub delay: Option<String>,
    pub enabled: bool,
}

/// Configuración responsive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveConfig {
    pub breakpoints: HashMap<String, String>,
    pub mobile_layout: Option<String>,
    pub tablet_layout: Option<String>,
    pub desktop_layout: Option<String>,
}

/// Eventos de UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEvent {
    pub id: String,
    pub event_type: EventType,
    pub target: String,
    pub payload: serde_json::Value,
    pub handler: Option<String>,
}

/// Tipos de eventos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Click,
    DoubleClick,
    Hover,
    Focus,
    Blur,
    Change,
    Submit,
    KeyPress,
    KeyDown,
    KeyUp,
    Load,
    Unload,
    Resize,
    Scroll,
    Drag,
    Drop,
    Swipe,
    Pinch,
}

/// Dashboard del Olimpo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OlympusDashboard {
    pub id: String,
    pub title: String,
    pub layout: DashboardLayout,
    pub widgets: Vec<DashboardWidget>,
    pub theme: ColorScheme,
    pub refresh_interval: u64,
}

/// Layout del dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: u32,
    pub gap: String,
    pub padding: String,
    pub auto_resize: bool,
}

/// Widget del dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub data_source: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub config: serde_json::Value,
    pub refresh_rate: Option<u64>,
}

/// Tipos de widgets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetType {
    // Metrics
    MetricCard,
    Gauge,
    Progress,

    // Charts
    LineChart,
    BarChart,
    PieChart,
    ScatterPlot,
    HeatMap,

    // System
    ActorStatus,
    SystemHealth,
    ActivityLog,
    PerformanceMetrics,

    // Controls
    ControlPanel,
    CommandButton,
    ToggleSwitch,
    Slider,

    // Information
    InfoCard,
    AlertPanel,
    NewsFeed,
    NotificationList,
}

/// Posición del widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub z_index: Option<u32>,
}

/// Tamaño del widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

/// Componentes especializados del Olimpo
impl UIComponent {
    /// Crea una tarjeta de métricas de un dios
    pub fn god_metric_card(god_name: &str, metrics: &serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: format!("metric_card_{}", god_name),
            component_type: ComponentType::MetricCard,
            props: serde_json::json!({
                "title": god_name,
                "metrics": metrics,
                "status": "healthy"
            }),
            children: vec![],
            style: ComponentStyle {
                theme: ThemeVariant::Auto,
                size: ComponentSize::M,
                color_scheme: ColorScheme::default(),
                spacing: SpacingConfig::default(),
                typography: TypographyConfig::default(),
                animations: AnimationConfig::default(),
                responsive: ResponsiveConfig::default(),
            },
            events: vec![],
        }
    }

    /// Crea un gráfico de líneas para tiempo real
    pub fn realtime_chart(data_source: &str, title: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: format!("chart_{}", data_source),
            component_type: ComponentType::LineChart,
            props: serde_json::json!({
                "title": title,
                "data_source": data_source,
                "realtime": true,
                "time_range": "1h"
            }),
            children: vec![],
            style: ComponentStyle::default(),
            events: vec![],
        }
    }

    /// Crea un panel de control del sistema
    pub fn control_panel() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "system_control_panel".to_string(),
            component_type: ComponentType::ControlPanel,
            props: serde_json::json!({
                "controls": [
                    {"type": "button", "action": "start_system", "label": "Iniciar Sistema"},
                    {"type": "button", "action": "stop_system", "label": "Detener Sistema"},
                    {"type": "button", "action": "restart_system", "label": "Reiniciar Sistema"},
                    {"type": "toggle", "action": "maintenance_mode", "label": "Modo Mantenimiento"}
                ]
            }),
            children: vec![],
            style: ComponentStyle::default(),
            events: vec![],
        }
    }

    /// Crea una tabla de estado de actores
    pub fn actor_status_table() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "actor_status_table".to_string(),
            component_type: ComponentType::Table,
            props: serde_json::json!({
                "columns": [
                    {"key": "name", "label": "Actor", "sortable": true},
                    {"key": "status", "label": "Estado", "filterable": true},
                    {"key": "uptime", "label": "Tiempo Activo", "sortable": true},
                    {"key": "cpu", "label": "CPU%", "sortable": true},
                    {"key": "memory", "label": "Memoria%", "sortable": true},
                    {"key": "messages", "label": "Mensajes", "sortable": true},
                    {"key": "errors", "label": "Errores", "sortable": true}
                ],
                "data_source": "actor_status",
                "pagination": true,
                "searchable": true,
                "exportable": true
            }),
            children: vec![],
            style: ComponentStyle::default(),
            events: vec![],
        }
    }
}

/// Implementaciones por defecto
impl Default for ComponentStyle {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::Auto,
            size: ComponentSize::M,
            color_scheme: ColorScheme::default(),
            spacing: SpacingConfig::default(),
            typography: TypographyConfig::default(),
            animations: AnimationConfig::default(),
            responsive: ResponsiveConfig::default(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: "#3b82f6".to_string(),
            secondary: "#8b5cf6".to_string(),
            accent: "#ec4899".to_string(),
            success: "#22c55e".to_string(),
            warning: "#f59e0b".to_string(),
            error: "#ef4444".to_string(),
            background: "#ffffff".to_string(),
            surface: "#f3f4f6".to_string(),
            text: "#111827".to_string(),
            text_secondary: "#6b7280".to_string(),
        }
    }
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            xs: "0.25rem".to_string(),
            sm: "0.5rem".to_string(),
            md: "1rem".to_string(),
            lg: "1.5rem".to_string(),
            xl: "2rem".to_string(),
            xxl: "3rem".to_string(),
        }
    }
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_size: "16px".to_string(),
            font_weight: "400".to_string(),
            line_height: "1.5".to_string(),
            letter_spacing: None,
        }
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: "200ms".to_string(),
            easing: "ease-in-out".to_string(),
            delay: None,
            enabled: true,
        }
    }
}

impl Default for ResponsiveConfig {
    fn default() -> Self {
        let mut breakpoints = HashMap::new();
        breakpoints.insert("sm".to_string(), "640px".to_string());
        breakpoints.insert("md".to_string(), "768px".to_string());
        breakpoints.insert("lg".to_string(), "1024px".to_string());
        breakpoints.insert("xl".to_string(), "1280px".to_string());

        Self {
            breakpoints,
            mobile_layout: None,
            tablet_layout: None,
            desktop_layout: None,
        }
    }
}

/// Dashboard del Olimpo por defecto
impl OlympusDashboard {
    /// Crea el dashboard por defecto del Olimpo
    pub fn default_olympus() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: "Panel del Olimpo".to_string(),
            layout: DashboardLayout {
                columns: 4,
                gap: "1rem".to_string(),
                padding: "2rem".to_string(),
                auto_resize: true,
            },
            widgets: vec![
                DashboardWidget {
                    id: Uuid::new_v4().to_string(),
                    title: "Estado General del Sistema".to_string(),
                    widget_type: WidgetType::SystemHealth,
                    data_source: "system_health".to_string(),
                    position: WidgetPosition {
                        x: 0,
                        y: 0,
                        z_index: None,
                    },
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    config: serde_json::json!({"refresh_interval": 5000}),
                    refresh_rate: Some(5000),
                },
                DashboardWidget {
                    id: Uuid::new_v4().to_string(),
                    title: "Actividad de Actores".to_string(),
                    widget_type: WidgetType::LineChart,
                    data_source: "actor_activity".to_string(),
                    position: WidgetPosition {
                        x: 2,
                        y: 0,
                        z_index: None,
                    },
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    config: serde_json::json!({"time_range": "1h"}),
                    refresh_rate: Some(10000),
                },
                DashboardWidget {
                    id: Uuid::new_v4().to_string(),
                    title: "Panel de Control".to_string(),
                    widget_type: WidgetType::ControlPanel,
                    data_source: "control_panel".to_string(),
                    position: WidgetPosition {
                        x: 0,
                        y: 1,
                        z_index: None,
                    },
                    size: WidgetSize {
                        width: 1,
                        height: 1,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    config: serde_json::json!({}),
                    refresh_rate: None,
                },
                DashboardWidget {
                    id: Uuid::new_v4().to_string(),
                    title: "Registro de Actividad".to_string(),
                    widget_type: WidgetType::ActivityLog,
                    data_source: "activity_log".to_string(),
                    position: WidgetPosition {
                        x: 1,
                        y: 1,
                        z_index: None,
                    },
                    size: WidgetSize {
                        width: 2,
                        height: 1,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    config: serde_json::json!({"max_entries": 100}),
                    refresh_rate: Some(2000),
                },
                DashboardWidget {
                    id: Uuid::new_v4().to_string(),
                    title: "Métricas de Rendimiento".to_string(),
                    widget_type: WidgetType::PerformanceMetrics,
                    data_source: "performance_metrics".to_string(),
                    position: WidgetPosition {
                        x: 3,
                        y: 1,
                        z_index: None,
                    },
                    size: WidgetSize {
                        width: 1,
                        height: 1,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    config: serde_json::json!({"cpu": true, "memory": true, "network": true}),
                    refresh_rate: Some(5000),
                },
            ],
            theme: ColorScheme::default(),
            refresh_interval: 5000,
        }
    }
}
