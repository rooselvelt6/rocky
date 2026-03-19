use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{Path, State},
};
use tower_http::{services::ServeDir, cors::{CorsLayer, Any}};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

// Importar sistema de actores
mod actors;
mod genesis;

use crate::actors::{GodName, ActorMessage, MessagePayload};
use crate::genesis::OlympusGenesis;

use ractor::ActorRef;

// Estado del servidor
#[derive(Clone)]
pub struct AppState {
    pub patients: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    pub god_actors: Arc<RwLock<HashMap<GodName, ActorRef<ActorMessage>>>>,
    pub start_time: std::time::Instant,
}

// Modelos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub identity_card: String,
    pub principal_diagnosis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpRequest {
    pub session_id: String,
    pub otp_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub username: Option<String>,
    pub message: String,
}

#[tokio::main]
async fn main() {
    // Inicializar tracing
    tracing_subscriber::fmt::init();
    
    println!("🏔️  OLYMPUS SYSTEM v16 - RACTOR ENGINE  🏔️");
    println!("⚡  20 Gods United - High Availability Fabric");
    println!("🚀  Sincronizando tejido de actores...");

    // IGNICION: Iniciar los 20 dioses
    let god_actors = match OlympusGenesis::ignite().await {
        Ok(actors) => {
            println!("✅ {} Dioses activos en Ractor", actors.len());
            Arc::new(RwLock::new(actors))
        }
        Err(e) => {
            eprintln!("❌ Error en Ignición: {}", e);
            std::process::exit(1);
        }
    };

    // Estado compartido
    let state = AppState {
        patients: Arc::new(RwLock::new(HashMap::new())),
        god_actors,
        start_time: std::time::Instant::now(),
    };

    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Crear router
    let app = Router::new()
        // Autenticación (usa Hades)
        .route("/api/login_step1", post(login_step1))
        .route("/api/login_step2", post(login_step2))
        .route("/api/logout", post(logout))
        // Pacientes (usa Poseidon)
        .route("/api/patients", get(get_patients).post(create_patient))
        .route("/api/patients/:id", get(get_patient).delete(delete_patient))
        // Escalas (usa Athena)
        .route("/api/scales/glasgow", post(calculate_glasgow))
        .route("/api/scales/sofa", post(calculate_sofa))
        .route("/api/scales/news2", post(calculate_news2))
        // Monitoreo (usa Zeus y Erinyes)
        .route("/api/status", get(api_status))
        .route("/api/olympus/gods", get(api_gods))
        .route("/api/olympus/trinity", get(api_trinity))
        .route("/api/admin/stats", get(api_stats))
        // UI/Temas (usa Aphrodite - Diosa de la Belleza)
        .route("/api/aphrodite/theme", get(get_current_theme).post(switch_theme))
        .route("/api/aphrodite/themes", get(get_all_themes))
        .route("/api/aphrodite/css", get(get_css_variables))
        .route("/api/aphrodite/components", get(get_components).post(update_component))
        // Archivos estáticos
        .fallback_service(ServeDir::new("dist"))
        .layer(cors)
        .with_state(state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("🚀 Servidor Axum + Actores corriendo en http://{}", addr);
    println!("📁 Sirviendo archivos estáticos desde dist/");
    println!("⚡ Zeus supervisando {} dioses", 20);

    axum::serve(listener, app).await.unwrap();
}

// === AUTENTICACIÓN (Hades) ===

async fn login_step1(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Json<AuthResponse> {
    let actors = state.god_actors.read().await;
    if let Some(hades) = actors.get(&GodName::Hades) {
        let result = ractor::call!(hades, |reply| ActorMessage::new(
            GodName::Zeus,
            GodName::Hades,
            MessagePayload::Command {
                action: "authenticate".to_string(),
                data: serde_json::json!({
                    "username": req.username,
                    "password": req.password,
                }),
                reply: Some(reply),
            }
        ));

        if let Ok(MessagePayload::Response { success, data, .. }) = result {
            return Json(AuthResponse {
                success,
                token: None,
                username: data["username"].as_str().map(|s| s.to_string()),
                message: data["message"].as_str().unwrap_or("Error").to_string(),
            });
        }
    }

    Json(AuthResponse {
        success: false,
        token: None,
        username: None,
        message: "Hades no disponible".to_string(),
    })
}

async fn login_step2(
    State(state): State<AppState>,
    Json(req): Json<OtpRequest>,
) -> Json<AuthResponse> {
    let actors = state.god_actors.read().await;
    if let Some(hades) = actors.get(&GodName::Hades) {
        let result = ractor::call!(hades, |reply| ActorMessage::new(
            GodName::Zeus,
            GodName::Hades,
            MessagePayload::Command {
                action: "verify_otp".to_string(),
                data: serde_json::json!({
                    "otp_code": req.otp_code,
                    "username": "admin",
                }),
                reply: Some(reply),
            }
        ));

        if let Ok(MessagePayload::Response { success, data, .. }) = result {
            return Json(AuthResponse {
                success,
                token: data["token"].as_str().map(|s| s.to_string()),
                username: data["username"].as_str().map(|s| s.to_string()),
                message: if success { "¡Acceso concedido!".to_string() } else { data["message"].as_str().unwrap_or("Código inválido").to_string() },
            });
        }
    }

    Json(AuthResponse {
        success: false,
        token: None,
        username: None,
        message: "Fallo en verificación OTP".to_string(),
    })
}

async fn logout() -> Json<AuthResponse> {
    Json(AuthResponse {
        success: true,
        token: None,
        username: None,
        message: "Sesión cerrada - Hades protege tu salida".to_string(),
    })
}

// === PACIENTES (Poseidon) ===

async fn get_patients(State(state): State<AppState>) -> Json<serde_json::Value> {
    let actors = state.god_actors.read().await;
    if let Some(poseidon) = actors.get(&GodName::Poseidon) {
        // Usar ractor::call para peticiones request-response (RPC)
        let result = ractor::call!(poseidon, |reply| ActorMessage::new(
            GodName::Zeus,
            GodName::Poseidon,
            MessagePayload::Query {
                query_type: "get_patients".to_string(),
                params: serde_json::json!({}),
                reply: Some(reply),
            }
        ));

        match result {
            Ok(MessagePayload::Response { success, data, .. }) if success => {
                return Json(serde_json::json!({ "patients": data }));
            }
            Ok(MessagePayload::Response { error, .. }) => {
                return Json(serde_json::json!({ "error": error.unwrap_or_else(|| "Error desconocido".to_string()) }));
            }
            _ => {
                return Json(serde_json::json!({ "error": "Fallo en la comunicación con el actor" }));
            }
        }
    }

    Json(serde_json::json!({ "error": "Actor Poseidon no disponible" }))
}

async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let patients = state.patients.read().await;
    match patients.get(&id) {
        Some(p) => Json(json!({ "patient": p })),
        None => Json(json!({ "error": "Paciente no encontrado" })),
    }
}

async fn create_patient(
    State(state): State<AppState>,
    Json(patient): Json<Patient>,
) -> Json<serde_json::Value> {
    let id = uuid::Uuid::new_v4().to_string();
    
    // Enviar a Poseidon
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Poseidon,
        MessagePayload::Command {
            action: "create_patient".to_string(),
            data: json!({
                "id": &id,
                "first_name": &patient.first_name,
                "last_name": &patient.last_name,
                "identity_card": &patient.identity_card,
                "principal_diagnosis": &patient.principal_diagnosis,
            }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(poseidon) = actors.get(&GodName::Poseidon) {
        let _ = poseidon.send_message(msg);
    }

    // Guardar en memoria
    let patient_json = json!({
        "id": id,
        "first_name": patient.first_name,
        "last_name": patient.last_name,
        "identity_card": patient.identity_card,
        "principal_diagnosis": patient.principal_diagnosis,
    });
    
    state.patients.write().await.insert(id.clone(), patient_json.clone());
    
    Json(json!({ 
        "success": true, 
        "id": id,
        "message": "Paciente creado exitosamente",
        "patient": patient_json
    }))
}

async fn delete_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    // Enviar a Poseidon
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Poseidon,
        MessagePayload::Command {
            action: "delete_patient".to_string(),
            data: json!({ "id": &id }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(target) = actors.get(&GodName::Poseidon) {
        let _ = target.send_message(msg);
    }

    state.patients.write().await.remove(&id);
    
    Json(json!({ 
        "success": true, 
        "message": "Paciente eliminado exitosamente" 
    }))
}

// === ESCALAS (Athena) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlasgowRequest {
    pub patient_id: String,
    pub eye: i32,
    pub verbal: i32,
    pub motor: i32,
}

async fn calculate_glasgow(
    State(state): State<AppState>,
    Json(req): Json<GlasgowRequest>,
) -> Json<serde_json::Value> {
    let actors = state.god_actors.read().await;
    if let Some(athena) = actors.get(&GodName::Athena) {
        let result = ractor::call!(athena, |reply| ActorMessage::new(
            GodName::Zeus,
            GodName::Athena,
            MessagePayload::Command {
                action: "calculate_glasgow".to_string(),
                data: serde_json::json!({
                    "eye": req.eye,
                    "verbal": req.verbal,
                    "motor": req.motor,
                }),
                reply: Some(reply),
            }
        ));

        if let Ok(MessagePayload::Response { success, data, .. }) = result {
            return Json(serde_json::json!({
                "success": success,
                "scale": "Glasgow",
                "patient_id": req.patient_id,
                "total": data["total"],
                "interpretation": data["interpretation"],
                "calculated_by": "Athena"
            }));
        }
    }

    Json(serde_json::json!({ "error": "Athena no disponible" }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofaRequest {
    pub patient_id: String,
    pub respiratory: i32,
    pub coagulation: i32,
    pub liver: i32,
    pub cardiovascular: i32,
    pub cns: i32,
    pub renal: i32,
}

async fn calculate_sofa(
    State(state): State<AppState>,
    Json(req): Json<SofaRequest>,
) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Athena,
        MessagePayload::Command {
            action: "calculate_sofa".to_string(),
            data: json!({
                "respiratory": req.respiratory,
                "coagulation": req.coagulation,
                "liver": req.liver,
                "cardiovascular": req.cardiovascular,
                "cns": req.cns,
                "renal": req.renal,
            }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(athena) = actors.get(&GodName::Athena) {
        let _ = athena.send_message(msg);
    }

    let total = req.respiratory + req.coagulation + req.liver + req.cardiovascular + req.cns + req.renal;
    let mortality = match total {
        0..=6 => "< 10%",
        7..=9 => "15-20%",
        10..=12 => "40-50%",
        13..=24 => "> 80%",
        _ => "Error",
    };

    Json(json!({
        "success": true,
        "scale": "SOFA",
        "patient_id": req.patient_id,
        "total": total,
        "predicted_mortality": mortality,
        "calculated_by": "Athena"
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct News2Request {
    pub patient_id: String,
    pub respiration_rate: i32,
    pub oxygen_saturation: i32,
    pub temperature: f32,
    pub heart_rate: i32,
    pub systolic_bp: i32,
}

async fn calculate_news2(
    State(state): State<AppState>,
    Json(req): Json<News2Request>,
) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Athena,
        MessagePayload::Command {
            action: "calculate_news2".to_string(),
            data: json!({
                "respiration_rate": req.respiration_rate,
                "oxygen_saturation": req.oxygen_saturation,
                "temperature": req.temperature,
                "heart_rate": req.heart_rate,
                "systolic_bp": req.systolic_bp,
            }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(athena) = actors.get(&GodName::Athena) {
        let _ = athena.send_message(msg);
    }

    // Calcular NEWS2 simplificado
    let resp_score = match req.respiration_rate {
        0..=8 => 3, 9..=11 => 1, 12..=20 => 0, 21..=24 => 2, _ => 3,
    };
    let spo2_score = match req.oxygen_saturation {
        0..=91 => 3, 92..=93 => 2, 94..=95 => 1, _ => 0,
    };
    let temp_score = match req.temperature {
        t if t < 35.0 => 3, t if t <= 36.0 => 1, t if t <= 38.0 => 0, t if t <= 39.0 => 1, _ => 2,
    };
    let hr_score = match req.heart_rate {
        0..=40 => 3, 41..=50 => 1, 51..=90 => 0, 91..=110 => 1, 111..=130 => 2, _ => 3,
    };
    let bp_score = match req.systolic_bp {
        0..=90 => 3, 91..=100 => 2, 101..=110 => 1, 111..=219 => 0, _ => 3,
    };

    let total = resp_score + spo2_score + temp_score + hr_score + bp_score;
    let risk = match total {
        0..=4 => "Bajo riesgo",
        5..=6 => "Riesgo moderado",
        _ => "Alto riesgo - respuesta de emergencia",
    };

    Json(json!({
        "success": true,
        "scale": "NEWS2",
        "patient_id": req.patient_id,
        "total": total,
        "risk_level": risk,
        "calculated_by": "Athena"
    }))
}

// === MONITOREO (Zeus + Erinyes) ===

async fn api_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let actors = state.god_actors.read().await;
    
    Json(json!({
        "status": "active",
        "version": "v16.0.0",
        "mode": "Olympus Ractor Fabric",
        "active_gods": actors.len(),
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "message": "Sistema operativo v16 acelerado por Ractor",
        "trinity": ["Zeus", "Hades", "Poseidon"],
    }))
}

async fn api_gods(State(state): State<AppState>) -> Json<serde_json::Value> {
    let actors = state.god_actors.read().await;
    
    let gods: Vec<serde_json::Value> = actors.keys().map(|god| {
        json!({
            "name": god.as_str(),
            "domain": god.domain(),
            "active": true,
            "status": "Active (Ractor)",
            "uptime_seconds": state.start_time.elapsed().as_secs(),
        })
    }).collect();

    Json(json!({
        "gods": gods,
        "total": gods.len(),
        "all_active": true,
        "fabric_status": "Healthy",
    }))
}

async fn api_trinity(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Consultar estado de la Trinidad a Zeus
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Zeus,
        MessagePayload::Query {
            query_type: "supervision_status".to_string(),
            params: json!({}),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(zeus) = actors.get(&GodName::Zeus) {
        let _ = zeus.send_message(msg);
    }

    Json(json!({
        "trinity": {
            "zeus": { "name": "Zeus", "domain": "Governance", "healthy": true, "status": "Supervising" },
            "hades": { "name": "Hades", "domain": "Security", "healthy": true, "status": "Protecting" },
            "poseidon": { "name": "Poseidon", "domain": "DataFlow", "healthy": true, "status": "Connecting" },
        },
        "all_healthy": true,
        "supervised_actors": 19,
    }))
}

async fn api_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let patients = state.patients.read().await;
    let actors = state.god_actors.read().await;
    
    Json(json!({
        "total_patients": patients.len(),
        "active_patients": patients.len(),
        "olympus_gods": actors.len(),
        "fabric_status": "Synchronized",
        "system_uptime": format!("{}s", state.start_time.elapsed().as_secs()),
        "v16_ready": true,
    }))
}

// === UI/TEMAS (Aphrodite - Diosa de la Belleza) ===

async fn get_current_theme(State(state): State<AppState>) -> Json<serde_json::Value> {
    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let result = ractor::call!(aphrodite, |reply| ActorMessage::new(
            GodName::Zeus,
            GodName::Aphrodite,
            MessagePayload::Query {
                query_type: "get_current_theme".to_string(),
                params: serde_json::json!({}),
                reply: Some(reply),
            }
        ));

        if let Ok(MessagePayload::Response { success, data, .. }) = result {
            return Json(serde_json::json!({
                "theme": data,
                "controlled_by": "Aphrodite",
                "success": success
            }));
        }
    }

    // Fallback si Aphrodite falla
    Json(serde_json::json!({
        "theme": {
            "name": "Olympus Dark (Fallback)",
            "primary_color": "#6366f1",
            "background": "#0f172a"
        },
        "error": "Aphrodite no disponible"
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchThemeRequest {
    pub theme_name: String,
}

async fn switch_theme(
    State(state): State<AppState>,
    Json(req): Json<SwitchThemeRequest>,
) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Command {
            action: "switch_theme".to_string(),
            data: json!({
                "theme_name": req.theme_name,
            }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let _ = aphrodite.send_message(msg);
    }

    Json(json!({
        "success": true,
        "message": format!("🎨 Aphrodite cambió el tema a: {}", req.theme_name),
        "theme": req.theme_name,
    }))
}

async fn get_all_themes(State(state): State<AppState>) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Query {
            query_type: "get_all_themes".to_string(),
            params: json!({}),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let _ = aphrodite.send_message(msg);
    }

    Json(json!({
        "themes": [
            "Olympus Dark",
            "Olympus Light", 
            "Golden Olympus",
            "Cosmic"
        ],
        "current": "Olympus Dark",
        "designed_by": "Aphrodite"
    }))
}

async fn get_css_variables(State(state): State<AppState>) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Query {
            query_type: "get_css_variables".to_string(),
            params: json!({}),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let _ = aphrodite.send_message(msg);
    }

    Json(json!({
        "css": r#":root {
  --color-primary: #6366f1;
  --color-secondary: #8b5cf6;
  --color-background: #0f172a;
  --color-surface: #1e293b;
  --color-text-primary: #f8fafc;
  --color-text-secondary: #94a3b8;
  --color-accent: #f59e0b;
  --border-radius: 0.75rem;
}"#,
        "styled_by": "Aphrodite"
    }))
}

async fn get_components(State(state): State<AppState>) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Query {
            query_type: "get_component_styles".to_string(),
            params: json!({}),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let _ = aphrodite.send_message(msg);
    }

    Json(json!({
        "components": [
            {
                "id": "button",
                "name": "Botón",
                "type": "button",
                "styles": {
                    "padding": "0.75rem 1.5rem",
                    "borderRadius": "0.5rem",
                    "fontWeight": "600"
                }
            },
            {
                "id": "card",
                "name": "Tarjeta",
                "type": "card",
                "styles": {
                    "padding": "1.5rem",
                    "borderRadius": "0.75rem",
                    "borderWidth": "1px"
                }
            }
        ],
        "managed_by": "Aphrodite"
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComponentRequest {
    pub component_id: String,
    pub style_key: String,
    pub style_value: String,
}

async fn update_component(
    State(state): State<AppState>,
    Json(req): Json<UpdateComponentRequest>,
) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Command {
            action: "update_component_style".to_string(),
            data: json!({
                "component_id": req.component_id,
                "style_key": req.style_key,
                "style_value": req.style_value,
            }),
            reply: None,
        }
    );

    let actors = state.god_actors.read().await;
    if let Some(aphrodite) = actors.get(&GodName::Aphrodite) {
        let _ = aphrodite.send_message(msg);
    }

    Json(json!({
        "success": true,
        "message": format!("🎨 Aphrodite actualizó {}.{} = {}", 
            req.component_id, req.style_key, req.style_value),
    }))
}
