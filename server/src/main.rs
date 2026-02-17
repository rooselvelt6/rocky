use axum::{
    routing::{get, post, delete},
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

use actors::{GodName, ActorMessage, MessagePayload};
use genesis::OlympusGenesis;

// Estado del servidor
#[derive(Clone)]
pub struct AppState {
    pub patients: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    pub god_senders: Arc<RwLock<HashMap<GodName, mpsc::Sender<ActorMessage>>>>,
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
    
    println!("🏔️  OLYMPUS SYSTEM v15 - ACTOR SYSTEM  🏔️");
    println!("⚡  20 Divine Gods - OTP Architecture");
    println!("🚀  Integrando sistema de actores...");

    // IGNICION: Iniciar los 20 dioses
    let god_senders = match OlympusGenesis::ignite().await {
        Ok(senders) => {
            println!("✅ {} Dioses iniciados correctamente", senders.len());
            Arc::new(RwLock::new(senders))
        }
        Err(e) => {
            eprintln!("❌ Error iniciando Genesis: {}", e);
            std::process::exit(1);
        }
    };

    // Estado compartido
    let state = AppState {
        patients: Arc::new(RwLock::new(HashMap::new())),
        god_senders,
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
    // Enviar mensaje a Hades para autenticar
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Hades,
        MessagePayload::Command {
            action: "authenticate".to_string(),
            data: json!({
                "username": req.username,
                "password": req.password,
            }),
        }
    );

    // En una implementación completa, esperaríamos respuesta async
    // Por ahora, simulamos la respuesta
    let senders = state.god_senders.read().await;
    if let Some(hades_tx) = senders.get(&GodName::Hades) {
        let _ = hades_tx.send(msg).await;
    }

    // Simular respuesta
    if req.username == "admin" && req.password == "admin123" {
        Json(AuthResponse {
            success: true,
            token: None,
            username: Some(req.username),
            message: "Código OTP enviado: 123456".to_string(),
        })
    } else {
        Json(AuthResponse {
            success: false,
            token: None,
            username: None,
            message: "Credenciales inválidas".to_string(),
        })
    }
}

async fn login_step2(
    State(state): State<AppState>,
    Json(req): Json<OtpRequest>,
) -> Json<AuthResponse> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Hades,
        MessagePayload::Command {
            action: "verify_otp".to_string(),
            data: json!({
                "otp_code": req.otp_code,
                "username": "admin",
            }),
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(hades_tx) = senders.get(&GodName::Hades) {
        let _ = hades_tx.send(msg).await;
    }

    if req.otp_code == "123456" {
        Json(AuthResponse {
            success: true,
            token: Some("jwt_token_olympus_2026".to_string()),
            username: Some("admin".to_string()),
            message: "¡Zeus aprueba tu acceso!".to_string(),
        })
    } else {
        Json(AuthResponse {
            success: false,
            token: None,
            username: None,
            message: "Código OTP inválido".to_string(),
        })
    }
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
    // Enviar mensaje a Poseidon
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Poseidon,
        MessagePayload::Query {
            query_type: "get_patients".to_string(),
            params: json!({}),
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(poseidon_tx) = senders.get(&GodName::Poseidon) {
        let _ = poseidon_tx.send(msg).await;
    }

    // Por ahora, leer de memoria
    let patients = state.patients.read().await;
    let list: Vec<_> = patients.values().cloned().collect();
    Json(json!({ "patients": list }))
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(poseidon_tx) = senders.get(&GodName::Poseidon) {
        let _ = poseidon_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(poseidon_tx) = senders.get(&GodName::Poseidon) {
        let _ = poseidon_tx.send(msg).await;
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
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Athena,
        MessagePayload::Command {
            action: "calculate_glasgow".to_string(),
            data: json!({
                "eye": req.eye,
                "verbal": req.verbal,
                "motor": req.motor,
            }),
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(athena_tx) = senders.get(&GodName::Athena) {
        let _ = athena_tx.send(msg).await;
    }

    // Calcular respuesta
    let total = req.eye + req.verbal + req.motor;
    let interpretation = match total {
        3..=8 => "Coma severo",
        9..=12 => "Coma moderado",
        13..=15 => "Coma leve/Normal",
        _ => "Error",
    };

    Json(json!({
        "success": true,
        "scale": "Glasgow",
        "patient_id": req.patient_id,
        "eye": req.eye,
        "verbal": req.verbal,
        "motor": req.motor,
        "total": total,
        "interpretation": interpretation,
        "calculated_by": "Athena"
    }))
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(athena_tx) = senders.get(&GodName::Athena) {
        let _ = athena_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(athena_tx) = senders.get(&GodName::Athena) {
        let _ = athena_tx.send(msg).await;
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
    let uptime = state.start_time.elapsed().as_secs();
    let senders = state.god_senders.read().await;
    
    Json(json!({
        "status": "active",
        "version": "v15.0.0",
        "mode": "Olympus Actor System",
        "active_gods": senders.len(),
        "uptime_seconds": uptime,
        "message": "Sistema operativo con 20 dioses divinos",
        "trinity": ["Zeus", "Hades", "Poseidon"],
    }))
}

async fn api_gods(State(state): State<AppState>) -> Json<serde_json::Value> {
    let senders = state.god_senders.read().await;
    
    // Construir lista de dioses con datos simulados (en producción vendrían de health checks)
    let gods: Vec<serde_json::Value> = senders.keys().map(|god| {
        json!({
            "name": god.as_str(),
            "domain": god.domain(),
            "active": true,
            "status": "Active",
            "messages_processed": 0,
            "uptime_seconds": state.start_time.elapsed().as_secs(),
        })
    }).collect();

    Json(json!({
        "gods": gods,
        "total": gods.len(),
        "all_active": true,
        "trinity_status": "Healthy",
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(zeus_tx) = senders.get(&GodName::Zeus) {
        let _ = zeus_tx.send(msg).await;
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
    let senders = state.god_senders.read().await;
    
    Json(json!({
        "total_patients": patients.len(),
        "active_patients": patients.len(),
        "olympus_gods": senders.len(),
        "gods_active": senders.len(),
        "system_uptime": format!("{}s", state.start_time.elapsed().as_secs()),
        "trinity_healthy": true,
    }))
}

// === UI/TEMAS (Aphrodite - Diosa de la Belleza) ===

async fn get_current_theme(State(state): State<AppState>) -> Json<serde_json::Value> {
    let msg = ActorMessage::new(
        GodName::Zeus,
        GodName::Aphrodite,
        MessagePayload::Query {
            query_type: "get_current_theme".to_string(),
            params: json!({}),
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
    }

    // Respuesta por defecto (en producción vendría del actor)
    Json(json!({
        "theme": {
            "name": "Olympus Dark",
            "primary_color": "#6366f1",
            "secondary_color": "#8b5cf6",
            "background": "#0f172a",
            "surface": "#1e293b",
            "text_primary": "#f8fafc",
            "text_secondary": "#94a3b8",
            "accent": "#f59e0b",
            "border_radius": "0.75rem",
        },
        "controlled_by": "Aphrodite"
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
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
        }
    );

    let senders = state.god_senders.read().await;
    if let Some(aphrodite_tx) = senders.get(&GodName::Aphrodite) {
        let _ = aphrodite_tx.send(msg).await;
    }

    Json(json!({
        "success": true,
        "message": format!("🎨 Aphrodite actualizó {}.{} = {}", 
            req.component_id, req.style_key, req.style_value),
    }))
}
