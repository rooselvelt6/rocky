// src/actors/aphrodite/animations.rs
// OLYMPUS v15 - Aphrodite: Sistema de Animaciones

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Sistema de animaciones para UI/UX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSystem {
    pub id: String,
    pub animations: HashMap<String, Animation>,
    pub transitions: HashMap<String, Transition>,
    pub keyframes: HashMap<String, KeyframeSet>,
    pub presets: AnimationPresets,
    pub performance: AnimationPerformance,
}

/// Animación individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub id: String,
    pub name: String,
    pub duration: f64,
    pub delay: f64,
    pub easing: EasingFunction,
    pub iterations: AnimationIteration,
    pub direction: AnimationDirection,
    pub fill_mode: AnimationFillMode,
    pub play_state: AnimationPlayState,
    pub keyframes: Vec<Keyframe>,
    pub targets: Vec<String>,
}

/// Keyframe individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub offset: f64,
    pub properties: HashMap<String, AnimationValue>,
    pub easing: Option<EasingFunction>,
}

/// Valor de animación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationValue {
    Number(f64),
    String(String),
    Color(String),
    Transform(Transform),
    Array(Vec<AnimationValue>),
    Object(HashMap<String, AnimationValue>),
}

/// Transformación CSS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub translate_x: Option<f64>,
    pub translate_y: Option<f64>,
    pub translate_z: Option<f64>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub scale_z: Option<f64>,
    pub rotate_x: Option<f64>,
    pub rotate_y: Option<f64>,
    pub rotate_z: Option<f64>,
    pub skew_x: Option<f64>,
    pub skew_y: Option<f64>,
}

/// Funciones de easing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    EaseOutFlash,
    CubicBezier(f64, f64, f64, f64),
    Steps(u32, StepPosition),
}

/// Posición de steps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}

/// Iteraciones de animación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationIteration {
    Infinite,
    Number(u32),
}

/// Dirección de animación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

/// Modo de relleno de animación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

/// Estado de reproducción de animación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationPlayState {
    Running,
    Paused,
    Idle,
}

/// Transición entre estados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub property: String,
    pub duration: f64,
    pub delay: f64,
    pub easing: EasingFunction,
    pub from_value: AnimationValue,
    pub to_value: AnimationValue,
}

/// Conjunto de keyframes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeSet {
    pub id: String,
    pub name: String,
    pub keyframes: Vec<Keyframe>,
}

/// Presets de animaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPresets {
    pub entrance: HashMap<String, Animation>,
    pub exit: HashMap<String, Animation>,
    pub attention: HashMap<String, Animation>,
    pub loading: HashMap<String, Animation>,
    pub feedback: HashMap<String, Animation>,
    pub special: HashMap<String, Animation>,
}

/// Configuración de rendimiento de animaciones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPerformance {
    pub max_concurrent_animations: u32,
    pub frame_rate_target: u32,
    pub gpu_acceleration: bool,
    pub reduce_motion: bool,
    pub battery_optimization: bool,
    pub fallback_animations: bool,
}

/// Efectos de animación especializados para el Olimpo
impl AnimationSystem {
    /// Crea un nuevo sistema de animaciones
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            animations: HashMap::new(),
            transitions: HashMap::new(),
            keyframes: HashMap::new(),
            presets: AnimationPresets::default(),
            performance: AnimationPerformance::default(),
        }
    }

    /// Animación de aparición divina (para dioses)
    pub fn divine_appearance() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "divine_appearance".to_string(),
            duration: 1.5,
            delay: 0.0,
            easing: EasingFunction::EaseOutElastic,
            iterations: AnimationIteration::Number(1),
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::Both,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(0.0),
                                scale_y: Some(0.0),
                                translate_y: Some(-50.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.5,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(0.8));
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(1.1),
                                scale_y: Some(1.1),
                                translate_y: Some(10.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 1.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(1.0),
                                scale_y: Some(1.0),
                                translate_y: Some(0.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Animación de rayo (para Zeus)
    pub fn lightning_strike() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "lightning_strike".to_string(),
            duration: 0.3,
            delay: 0.0,
            easing: EasingFunction::EaseOutFlash,
            iterations: AnimationIteration::Number(1),
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::Forwards,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                        props.insert(
                            "filter".to_string(),
                            AnimationValue::String("brightness(0)".to_string()),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.1,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                        props.insert(
                            "filter".to_string(),
                            AnimationValue::String("brightness(3)".to_string()),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.3,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                        props.insert(
                            "filter".to_string(),
                            AnimationValue::String("brightness(1)".to_string()),
                        );
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Animación de flujo de datos (para Poseidón)
    pub fn data_flow() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "data_flow".to_string(),
            duration: 2.0,
            delay: 0.0,
            easing: EasingFunction::EaseInOutSine,
            iterations: AnimationIteration::Infinite,
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::None,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(-100.0),
                                ..Default::default()
                            }),
                        );
                        props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.1,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.9,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 1.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(100.0),
                                ..Default::default()
                            }),
                        );
                        props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Animación de pulsación (para monitoreo)
    pub fn heartbeat_pulse() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "heartbeat_pulse".to_string(),
            duration: 1.0,
            delay: 0.0,
            easing: EasingFunction::EaseInOutQuad,
            iterations: AnimationIteration::Infinite,
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::None,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(1.0),
                                scale_y: Some(1.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.5,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(1.2),
                                scale_y: Some(1.2),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 1.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                scale_x: Some(1.0),
                                scale_y: Some(1.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Animación de error crítico
    pub fn critical_error() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "critical_error".to_string(),
            duration: 0.5,
            delay: 0.0,
            easing: EasingFunction::EaseInOutBounce,
            iterations: AnimationIteration::Number(3),
            direction: AnimationDirection::Alternate,
            fill_mode: AnimationFillMode::Both,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(0.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.25,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(-10.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 0.75,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(10.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 1.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                translate_x: Some(0.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Animación de carga elegante
    pub fn elegant_loading() -> Animation {
        Animation {
            id: Uuid::new_v4().to_string(),
            name: "elegant_loading".to_string(),
            duration: 2.0,
            delay: 0.0,
            easing: EasingFunction::EaseInOutCubic,
            iterations: AnimationIteration::Infinite,
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::None,
            play_state: AnimationPlayState::Running,
            keyframes: vec![
                Keyframe {
                    offset: 0.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                rotate_z: Some(0.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
                Keyframe {
                    offset: 1.0,
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "transform".to_string(),
                            AnimationValue::Transform(Transform {
                                rotate_z: Some(360.0),
                                ..Default::default()
                            }),
                        );
                        props
                    },
                    easing: None,
                },
            ],
            targets: vec![],
        }
    }

    /// Agrega una animación al sistema
    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.insert(animation.id.clone(), animation);
    }

    /// Obtiene una animación por ID
    pub fn get_animation(&self, id: &str) -> Option<&Animation> {
        self.animations.get(id)
    }

    /// Elimina una animación
    pub fn remove_animation(&mut self, id: &str) -> Option<Animation> {
        self.animations.remove(id)
    }

    /// Optimiza animaciones para rendimiento
    pub fn optimize_for_performance(&mut self) {
        if self.performance.reduce_motion {
            // Reducir complejidad de animaciones
            for animation in self.animations.values_mut() {
                animation.duration = animation.duration.min(0.3);
                animation.easing = EasingFunction::Linear;
            }
        }

        if self.performance.battery_optimization {
            // Reducir frecuencia de actualización
            for animation in self.animations.values_mut() {
                if matches!(animation.iterations, AnimationIteration::Infinite) {
                    animation.iterations = AnimationIteration::Number(1);
                }
            }
        }
    }
}

/// Implementaciones por defecto
impl Default for AnimationPerformance {
    fn default() -> Self {
        Self {
            max_concurrent_animations: 50,
            frame_rate_target: 60,
            gpu_acceleration: true,
            reduce_motion: false,
            battery_optimization: false,
            fallback_animations: true,
        }
    }
}

impl Default for AnimationPresets {
    fn default() -> Self {
        let mut entrance = HashMap::new();
        let exit = HashMap::new();
        let attention = HashMap::new();
        let loading = HashMap::new();
        let feedback = HashMap::new();
        let special = HashMap::new();

        // Presets de entrada
        entrance.insert(
            "fade_in".to_string(),
            Animation {
                id: "fade_in".to_string(),
                name: "Fade In".to_string(),
                duration: 0.5,
                delay: 0.0,
                easing: EasingFunction::EaseOut,
                iterations: AnimationIteration::Number(1),
                direction: AnimationDirection::Normal,
                fill_mode: AnimationFillMode::Forwards,
                play_state: AnimationPlayState::Running,
                keyframes: vec![
                    Keyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                        easing: None,
                    },
                    Keyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                        easing: None,
                    },
                ],
                targets: vec![],
            },
        );

        Self {
            entrance,
            exit,
            attention,
            loading,
            feedback,
            special,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate_x: None,
            translate_y: None,
            translate_z: None,
            scale_x: None,
            scale_y: None,
            scale_z: None,
            rotate_x: None,
            rotate_y: None,
            rotate_z: None,
            skew_x: None,
            skew_y: None,
        }
    }
}
