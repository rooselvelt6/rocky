// src/main.rs
// src/main.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn,
    routing::{get, post, put},
    Json, Router,
};
// use std::sync::Arc;
use surrealdb::engine::any::Any;
use surrealdb::{RecordId, Surreal};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use uci::uci::scale::apache::{ApacheIIRequest, ApacheIIResponse};
use uci::uci::scale::glasgow::{Glasgow, GlasgowRequest, GlasgowResponse};
use uci::uci::scale::saps::{SAPSIIRequest, SAPSIIResponse};
use uci::uci::scale::sofa::{SOFARequest, SOFAResponse};

// Import our new modules
mod olympus;
// mod models; // Moved to lib.rs

use uci::models::apache::ApacheAssessment;
use uci::models::config::SystemConfig;
use uci::models::glasgow::GlasgowAssessment;
use uci::models::history::PatientHistoryResponse;
use uci::models::news2::{ConsciousnessLevel, News2Assessment, News2RiskLevel};
use uci::models::patient::Patient;
use uci::models::saps::SapsAssessment;
use uci::models::sofa::SofaAssessment;
use uci::models::user::User;

#[cfg(feature = "ssr")]
use uci::services::validation;

#[cfg(feature = "ssr")]
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
    poseidon: std::sync::Arc<crate::olympus::poseidon::Poseidon>,
    hades: std::sync::Arc<crate::olympus::hades::Hades>,
    artemis: std::sync::Arc<crate::olympus::artemis::Artemis>,
    apollo: std::sync::Arc<crate::olympus::apollo::Apollo>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // --- FASE 1: EL DESPERTAR DEL TRONO (OLIMPO v10) ---
    println!("🏛️  Iniciando Jerarquía Soberana v10: La Luz Abyssal...");
    let iris = crate::olympus::iris::Iris::new();
    let iris_tx = iris.sender();
    
    let mut zeus = crate::olympus::zeus::Zeus::new(iris_tx.clone());
    tokio::spawn(async move {
        use crate::olympus::GodActor;
        if let Err(e) = zeus.start().await {
            eprintln!("🚨 Zeus falló al iniciar: {}", e);
        }
    });

    let mut erinyes = crate::olympus::erinyes::Erinyes::new(iris_tx.subscribe());
    tokio::spawn(async move {
        erinyes.run().await;
    });

    // Deidades Estratégicas
    let mut hera = crate::olympus::hera::Hera::new();
    let mut athena = crate::olympus::athena::Athena::new();
    let chronos = crate::olympus::chronos::Chronos::new(iris_tx.clone());
    let mut hestia = crate::olympus::hestia::Hestia::new();

    tokio::spawn(async move {
        use crate::olympus::GodActor;
        let _ = hera.start().await;
        let _ = athena.start().await;
        let _ = hestia.start().await;
    });

    tokio::spawn(async move {
        chronos.heartbeat_loop().await;
    });

    // Panteón Operativo (v10)
    let ops_gods = vec![
        Box::new(crate::olympus::poseidon::Poseidon::new()) as Box<dyn crate::olympus::GodActor>,
        Box::new(crate::olympus::hades::Hades::new()),
        Box::new(crate::olympus::hephaestus::Hephaestus::new()),
        Box::new(crate::olympus::artemis::Artemis::new()),
        Box::new(crate::olympus::hermes::Hermes::new()),
        Box::new(crate::olympus::apollo::Apollo::new()),
        Box::new(crate::olympus::demeter::Demeter::new()),
        Box::new(crate::olympus::ares::Ares::new()),
        Box::new(crate::olympus::dionysus::Dionysus::new()),
        Box::new(crate::olympus::aphrodite::Aphrodite::new()),
    ];

    for mut god in ops_gods {
        tokio::spawn(async move {
            let _ = god.start().await;
        });
    }

    // Verificamos que la carpeta dist exista (generada por trunk build)
    if !std::path::Path::new("dist").exists() {
        eprintln!("ERROR: No se encuentra la carpeta 'dist/'");
        eprintln!("   Debes ejecutar 'trunk build' primero para compilar el frontend.");
        std::process::exit(1);
    }

    // Connect to SurrealDB via Poseidon
    println!("DEBUG: Connecting to DB via Poseidon...");
    let db = match crate::olympus::poseidon::Poseidon::connect_db().await {
        Ok(db) => {
            tracing::info!("✅ Database connection established via Poseidon");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to SurrealDB: {}", e);
            std::process::exit(1);
        }
    };

    let poseidon = std::sync::Arc::new(crate::olympus::poseidon::Poseidon::new());
    let hades = std::sync::Arc::new(crate::olympus::hades::Hades::new());
    let artemis = std::sync::Arc::new(crate::olympus::artemis::Artemis::new());
    let apollo = std::sync::Arc::new(crate::olympus::apollo::Apollo::new());
    
    let state = AppState { db, poseidon, hades, artemis, apollo };

    use tower_http::compression::CompressionLayer;
    use tower_http::cors::CorsLayer;

    // ⚠️ Rate Limiting Configuration
    // TODO: tower_governor 0.4.3 tiene incompatibilidad con Axum 0.8
    // Necesita actualizar a tower_governor 0.5+ cuando esté disponible
    // Ver: https://github.com/benwis/tower-governor/issues
    /*
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .finish()
            .expect("Failed to create rate limiter configuration"),
    );
    */

    let app = Router::new()
        // .layer(GovernorLayer { config: governor_conf })
        .layer(CompressionLayer::new()) // Auto-compress responses (Gzip/Brotli/Deflate)
        // API Endpoints
        .route("/api/glasgow", post(calculate_glasgow))
        .route("/api/apache", post(calculate_apache))
        .route("/api/sofa", post(calculate_sofa))
        .route("/api/saps", post(calculate_saps))
        .route("/api/news2", post(calculate_news2))
        .route("/api/patients", post(create_patient).get(get_patients))
        .route(
            "/api/patients/{id}",
            get(get_patient).put(update_patient).delete(delete_patient),
        )
        .route("/api/patients/{id}/history", get(get_patient_history))
        .route(
            "/api/patients/{id}/can-assess/{scale_type}",
            get(check_assessment_eligibility),
        )
        .route("/api/export/patients", get(export_patients_csv))
        .route("/api/login", post(login_handler))
        .route("/api/health", get(health_check))
        .route("/ws/poseidon", get(poseidon_ws_handler))
        // Admin Routes
        .route("/api/admin/config", get(get_config).put(update_config))
        .route("/api/admin/users", get(get_users).post(create_user))
        .route(
            "/api/admin/users/{id}",
            put(update_user).delete(delete_user),
        )
        // Assessment deletion routes
        .route(
            "/api/assessments/glasgow/{id}",
            axum::routing::delete(delete_glasgow_assessment),
        )
        .route(
            "/api/assessments/apache/{id}",
            axum::routing::delete(delete_apache_assessment),
        )
        .route(
            "/api/assessments/sofa/{id}",
            axum::routing::delete(delete_sofa_assessment),
        )
        .route(
            "/api/assessments/saps/{id}",
            axum::routing::delete(delete_saps_assessment),
        )
        .route(
            "/api/assessments/news2/{id}",
            axum::routing::delete(delete_news2_assessment),
        )
        .layer(from_fn(crate::olympus::artemis::Artemis::auth_middleware))
        // Servir archivos estáticos desde dist
        .fallback_service(
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        )
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:3000".parse().unwrap(),
                    "http://127.0.0.1:3000".parse().unwrap(),
                    // Para producción, agregar el dominio real:
                    // "https://uci.hospital.com".parse().unwrap(),
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::CONTENT_TYPE,
                ]),
        )
        .with_state(state.clone()); // Add app state

    // --- INITIALIZE SYSTEM CONFIG ---
    let init_db = state.db.clone();
    tokio::spawn(async move {
        let configs: Vec<SystemConfig> = init_db.select("system_config").await.unwrap_or_default();
        if configs.is_empty() {
            let id = RecordId::from(("system_config", "settings"));
            let _: Option<SystemConfig> = init_db
                .update(id)
                .content(SystemConfig::default())
                .await
                .unwrap_or_default();
            println!("DEBUG: Initialized default system configuration.");
        }
    });

    println!("¡Servidor Axum arrancando...");
    println!("http://localhost:3000 → Aplicación UCI (Leptos + Axum)");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("No se pudo bindear el puerto 3000 (¿ya está en uso?)");

    println!("¡LISTO! Servidor corriendo en http://localhost:3000");

    // Graceful Shutdown Signal
    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        println!("🛑 Recibida señal de apagado. Cerrando servidor ordenadamente...");
    };

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();

    println!("👋 Servidor detenido correctamente.");
}

// --- HADES SHIELDING SYSTEM ---
use base64::{prelude::BASE64_STANDARD, Engine};

fn shield_patient(mut patient_data: Patient) -> Patient {
    let hades = crate::olympus::hades::Hades::new();
    
    // 1. Cifrar campos sensibles
    if let Ok(enc_first) = hades.encrypt(&patient_data.first_name) {
        patient_data.first_name = BASE64_STANDARD.encode(enc_first);
    }
    if let Ok(enc_last) = hades.encrypt(&patient_data.last_name) {
        patient_data.last_name = BASE64_STANDARD.encode(enc_last);
    }
    if let Ok(enc_diag) = hades.encrypt(&patient_data.principal_diagnosis) {
        patient_data.principal_diagnosis = BASE64_STANDARD.encode(enc_diag);
    }
    if let Ok(enc_dob) = hades.encrypt(&patient_data.date_of_birth) {
        patient_data.date_of_birth = BASE64_STANDARD.encode(enc_dob);
    }

    // 2. Calcular Hilo Rojo (Integridad)
    let integrity_string = format!(
        "{}:{}:{}:{}",
        patient_data.first_name, patient_data.last_name, patient_data.date_of_birth, patient_data.uci_admission_date
    );
    patient_data.integrity_hash = crate::olympus::hades::Hades::compute_hash(&integrity_string);
    
    patient_data
}

fn unshield_patient(mut patient_data: Patient) -> Patient {
    let hades = crate::olympus::hades::Hades::new();
    
    // 1. Verificar integridad
    let check_string = format!(
        "{}:{}:{}:{}",
        patient_data.first_name, patient_data.last_name, patient_data.date_of_birth, patient_data.uci_admission_date
    );
    let current_hash = crate::olympus::hades::Hades::compute_hash(&check_string);
    if current_hash != patient_data.integrity_hash {
        tracing::error!("🚨 ARTEMIS ADVERTENCIA: Violación del Hilo Rojo en paciente {:?}.", patient_data.id);
    }

    // 2. Descifrar campos
    if let Ok(bytes) = BASE64_STANDARD.decode(&patient_data.first_name) {
        if let Ok(dec) = hades.decrypt(&bytes) { patient_data.first_name = dec; }
    }
    if let Ok(bytes) = BASE64_STANDARD.decode(&patient_data.last_name) {
        if let Ok(dec) = hades.decrypt(&bytes) { patient_data.last_name = dec; }
    }
    if let Ok(bytes) = BASE64_STANDARD.decode(&patient_data.principal_diagnosis) {
        if let Ok(dec) = hades.decrypt(&bytes) { patient_data.principal_diagnosis = dec; }
    }
    if let Ok(bytes) = BASE64_STANDARD.decode(&patient_data.date_of_birth) {
        if let Ok(dec) = hades.decrypt(&bytes) { patient_data.date_of_birth = dec; }
    }

    patient_data
}

/// Endpoint para verificar el estado emocional... digo, de salud del sistema
async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match state.db.health().await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "up",
                "database": "connected",
                "version": env!("CARGO_PKG_VERSION")
            })),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "down",
                "database": "disconnected",
                "error": e.to_string()
            })),
        ),
    }
}

/// Helper function to check 24-hour restriction for any assessment type
async fn check_24h_restriction<T: serde::de::DeserializeOwned>(
    db: &Surreal<Any>,
    patient_id: &str,
    table_name: &str,
) -> Result<(), String> {
    let sql = format!(
        "SELECT * FROM {} WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC LIMIT 1",
        table_name
    );

    let mut params = std::collections::BTreeMap::new();
    params.insert("id".to_string(), patient_id.to_string());

    let mut resp = db
        .query(&sql)
        .bind(params)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let last_assessments: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();

    if let Some(last) = last_assessments.first() {
        if let Some(assessed_at) = last.get("assessed_at").and_then(|v| v.as_str()) {
            validation::validate_24_hour_interval(Some(assessed_at))?;
        }
    }

    Ok(())
}

/// Handler para calcular la escala de Glasgow y guardar en DB
async fn calculate_glasgow(
    State(state): State<AppState>,
    Json(payload): Json<GlasgowRequest>,
) -> Json<GlasgowResponse> {
    // Intentamos crear la escala con los valores recibidos
    match Glasgow::from_u8(payload.eye, payload.verbal, payload.motor) {
        Ok(glasgow) => {
            let (diagnosis, recommendation) = glasgow.result();
            let response = GlasgowResponse {
                score: glasgow.score(),
                diagnosis: diagnosis.clone(),
                recommendation: recommendation.clone(),
            };

            // Parse patient_id (Option<String> -> Option<RecordId>)
            let patient_id_thing = payload
                .patient_id
                .as_ref()
                .and_then(|id| id.parse::<RecordId>().ok());

            // If patient_id provided, check 24-hour restriction
            if let Some(p_id) = payload.patient_id.as_ref() {
                if let Err(msg) =
                    check_24h_restriction::<GlasgowAssessment>(&state.db, p_id, "glasgow_assessments")
                        .await
                {
                    return Json(GlasgowResponse {
                        score: 0,
                        diagnosis: "Restriction".to_string(),
                        recommendation: msg,
                    });
                }
            }

            // Save to database
            let mut assessment = GlasgowAssessment::new(
                payload.eye,
                payload.verbal,
                payload.motor,
                glasgow.score(),
                diagnosis,
                recommendation,
            );
            assessment.patient_id = patient_id_thing;

            match state.db.create("glasgow_assessments").content(assessment).await {
                Ok(saved) => {
                    // SurrealDB 2.x returns Option<T> for single create
                    let saved: Option<GlasgowAssessment> = saved;
                    if let Some(saved_assessment) = saved {
                        tracing::info!("✅ Saved assessment with ID: {:?}", saved_assessment.id);
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to save assessment: {}", e);
                }
            }

            Json(response)
        }
        Err(e) => Json(GlasgowResponse {
            score: 0,
            diagnosis: "Error".to_string(),
            recommendation: e,
        }),
    }
}

/// Handler para calcular APACHE II score y guardar en DB
async fn calculate_apache(
    State(state): State<AppState>,
    Json(payload): Json<ApacheIIRequest>,
) -> Json<ApacheIIResponse> {
    match payload.to_apache() {
        Ok(apache) => {
            // Validate physiological ranges
            if let Err(e) = uci::services::validation::validate_vitals(
                Some(payload.temperature as f64),
                Some(payload.mean_arterial_pressure as f64),
                Some(payload.heart_rate as f64),
                Some(payload.respiratory_rate as f64),
            ) {
                return Json(ApacheIIResponse {
                    score: 0,
                    predicted_mortality: 0.0,
                    severity: "Error".to_string(),
                    recommendation: e,
                });
            }

            let score = apache.calculate_score();
            let mortality = apache.predicted_mortality();
            let (severity, base_recommendation) = apache.severity();

            // Smart Clinical Analysis
            let smart_analysis = uci::services::clinical::analyze_mortality(mortality as f64);
            let recommendation = format!(
                "{}\n\n[AI INSIGHT]: {}",
                base_recommendation, smart_analysis
            );

            let response = ApacheIIResponse {
                score,
                predicted_mortality: mortality,
                severity: severity.clone(),
                recommendation: recommendation.clone(),
            };

            // Parse patient_id
            let patient_id_thing = payload
                .patient_id
                .as_ref()
                .and_then(|id| id.parse::<RecordId>().ok());

            // Check 24-hour restriction
            if let Some(p_id) = payload.patient_id.as_ref() {
                if let Err(msg) =
                    check_24h_restriction::<ApacheAssessment>(&state.db, p_id, "apache_assessments").await
                {
                    return Json(ApacheIIResponse {
                        score: 0,
                        predicted_mortality: 0.0,
                        severity: "Restriction".to_string(),
                        recommendation: msg,
                    });
                }
            }

            // Save to database
            let mut assessment = ApacheAssessment::new(
                payload.temperature,
                payload.mean_arterial_pressure,
                payload.heart_rate,
                payload.respiratory_rate,
                payload.oxygenation_type,
                payload.oxygenation_value,
                payload.arterial_ph,
                payload.serum_sodium,
                payload.serum_potassium,
                payload.serum_creatinine,
                payload.hematocrit,
                payload.white_blood_count,
                payload.glasgow_coma_score,
                payload.age,
                payload.chronic_health,
                score,
                mortality,
                severity,
                recommendation,
            );
            assessment.patient_id = patient_id_thing;

            match state.db.create("apache_assessments").content(assessment).await {
                Ok(saved) => {
                    // SurrealDB 2.x returns Option<T>
                    let saved: Option<ApacheAssessment> = saved;
                    if let Some(saved_assessment) = saved {
                        tracing::info!(
                            "✅ Saved APACHE II assessment with ID: {:?}",
                            saved_assessment.id
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to save APACHE II assessment: {}", e);
                }
            }

            Json(response)
        }
        Err(e) => Json(ApacheIIResponse {
            score: 0,
            predicted_mortality: 0.0,
            severity: "Error".to_string(),
            recommendation: e,
        }),
    }
}

/// Handler para calcular SOFA score y guardar en DB
async fn calculate_sofa(
    State(state): State<AppState>,
    Json(payload): Json<SOFARequest>,
) -> Json<SOFAResponse> {
    match payload.to_sofa() {
        Ok(sofa) => {
            let score = sofa.calculate_score();
            let (severity, recommendation) = sofa.interpretation();

            let response = SOFAResponse {
                score,
                severity: severity.clone(),
                recommendation: recommendation.clone(),
            };

            // Parse patient_id
            let patient_id_thing = payload
                .patient_id
                .as_ref()
                .and_then(|id| id.parse::<RecordId>().ok());

            // Check 24-hour restriction
            if let Some(p_id) = payload.patient_id.as_ref() {
                if let Err(msg) =
                    check_24h_restriction::<SofaAssessment>(&state.db, p_id, "sofa_assessments").await
                {
                    return Json(SOFAResponse {
                        score: 0,
                        severity: "Restriction".to_string(),
                        recommendation: msg,
                    });
                }
            }

            // Save to database
            let mut assessment = SofaAssessment::new(
                payload.pao2_fio2,
                payload.platelets,
                payload.bilirubin,
                payload.cardiovascular,
                payload.glasgow,
                payload.renal,
                score,
                severity,
                recommendation,
            );
            assessment.patient_id = patient_id_thing;

            match state.db.create("sofa_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Option<SofaAssessment> = saved;
                    if let Some(saved_assessment) = saved {
                        tracing::info!(
                            "✅ Saved SOFA assessment with ID: {:?}",
                            saved_assessment.id
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to save SOFA assessment: {}", e);
                }
            }

            Json(response)
        }
        Err(e) => Json(SOFAResponse {
            score: 0,
            severity: "Error".to_string(),
            recommendation: e,
        }),
    }
}

/// Handler para calcular SAPS II score y guardar en DB
async fn calculate_saps(
    State(state): State<AppState>,
    Json(payload): Json<SAPSIIRequest>,
) -> Json<SAPSIIResponse> {
    match payload.to_saps() {
        Ok(saps) => {
            // Validate physiological ranges
            if let Err(e) = uci::services::validation::validate_vitals(
                Some(payload.temperature as f64),
                Some(payload.systolic_bp as f64), // SAPS uses systolic, validating as mean for now or just checking general range
                Some(payload.heart_rate as f64),
                None, // SAPS doesn't use RR in the same way or optional
            ) {
                return Json(SAPSIIResponse {
                    score: 0,
                    predicted_mortality: 0.0,
                    severity: "Error".to_string(),
                    recommendation: e,
                });
            }

            let score = saps.calculate_score();
            let mortality = saps.predicted_mortality();
            let (severity, base_recommendation) = saps.interpretation();

            // Smart Clinical Analysis
            let smart_analysis = uci::services::clinical::analyze_mortality(mortality as f64);
            let recommendation = format!(
                "{}\n\n[AI INSIGHT]: {}",
                base_recommendation, smart_analysis
            );

            let response = SAPSIIResponse {
                score,
                predicted_mortality: mortality,
                severity: severity.clone(),
                recommendation: recommendation.clone(),
            };

            // Parse patient_id
            let patient_id_thing = payload
                .patient_id
                .as_ref()
                .and_then(|id| id.parse::<RecordId>().ok());

            // Check 24-hour restriction
            if let Some(p_id) = payload.patient_id.as_ref() {
                if let Err(msg) =
                    check_24h_restriction::<SapsAssessment>(&state.db, p_id, "saps_assessments").await
                {
                    return Json(SAPSIIResponse {
                        score: 0,
                        predicted_mortality: 0.0,
                        severity: "Restriction".to_string(),
                        recommendation: msg,
                    });
                }
            }

            // Save to database
            let mut assessment = SapsAssessment::new(
                payload.age,
                payload.heart_rate,
                payload.systolic_bp,
                payload.temperature,
                payload.pao2_fio2,
                payload.urinary_output,
                payload.serum_urea,
                payload.white_blood_count,
                payload.serum_potassium,
                payload.serum_sodium,
                payload.serum_bicarbonate,
                payload.bilirubin,
                payload.glasgow,
                payload.chronic_disease,
                payload.admission_type,
                score,
                mortality,
                severity,
                recommendation,
            );
            assessment.patient_id = patient_id_thing;

            match state.db.create("saps_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Option<SapsAssessment> = saved;
                    if let Some(saved_assessment) = saved {
                        tracing::info!(
                            "✅ Saved SAPS II assessment with ID: {:?}",
                            saved_assessment.id
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to save SAPS II assessment: {}", e);
                }
            }

            Json(response)
        }
        Err(e) => Json(SAPSIIResponse {
            score: 0,
            predicted_mortality: 0.0,
            severity: "Error".to_string(),
            recommendation: e,
        }),
    }
}

#[derive(serde::Deserialize)]
pub struct News2Request {
    pub respiration_rate: u8,
    pub spo2_scale: u8,
    pub spo2: u8,
    pub air_or_oxygen: bool,
    pub systolic_bp: u16,
    pub heart_rate: u16,
    pub consciousness: ConsciousnessLevel,
    pub temperature: f32,
    pub patient_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct News2Response {
    pub score: u8,
    pub risk_level: News2RiskLevel,
    pub recommendation: String,
}

/// Handler para calcular NEWS2 y guardar en DB
async fn calculate_news2(
    State(state): State<AppState>,
    Json(payload): Json<News2Request>,
) -> Json<News2Response> {
    let mut assessment_data = News2Assessment {
        id: None,
        patient_id: payload.patient_id.clone().unwrap_or_default(),
        assessed_at: chrono::Utc::now().to_rfc3339(),
        respiration_rate: payload.respiration_rate,
        spo2_scale: payload.spo2_scale,
        spo2: payload.spo2,
        air_or_oxygen: payload.air_or_oxygen,
        systolic_bp: payload.systolic_bp,
        heart_rate: payload.heart_rate,
        consciousness: payload.consciousness.clone(),
        temperature: payload.temperature,
        score: 0,
        risk_level: News2RiskLevel::Low,
    };

    assessment_data.calculate_score();

    let score = assessment_data.score;
    let risk_level = assessment_data.risk_level.clone();

    let recommendation = match risk_level {
        News2RiskLevel::Low => "Continue routine clinical monitoring.".to_string(),
        News2RiskLevel::LowMedium => {
            "Increased frequency of monitoring and clinical review.".to_string()
        }
        News2RiskLevel::Medium => {
            "Urgent clinical review by a clinician with core skills in assessment of acute illness."
                .to_string()
        }
        News2RiskLevel::High => {
            "Emergency assessment by a team with critical care skills!".to_string()
        }
    };

    // Save to database
    if let Some(_p_id_str) = payload.patient_id.as_ref() {
        match db
            .create("news2_assessments")
            .content(assessment_data)
            .await
        {
            Ok(saved) => {
                let saved: Option<News2Assessment> = saved;
                if let Some(s) = saved {
                    tracing::info!("✅ Saved NEWS2 assessment: {:?}", s.id);
                }
            }
            Err(e) => tracing::error!("❌ Failed to save NEWS2: {}", e),
        }
    }

    Json(News2Response {
        score,
        risk_level,
        recommendation,
    })
}

async fn delete_news2_assessment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match db
        .delete::<Option<News2Assessment>>(("news2_assessments", id))
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Handler para crear un nuevo paciente
async fn create_patient(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Json(payload): Json<Patient>,
) -> Json<Option<Patient>> {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());
    // Ensure we are creating a new record with the payload data
    // We might want to overwrite the ID or other fields if they are passed,
    // but for now let's trust the payload or better yet, reconstruct it to ensure safety.
    // Ideally we'd have a CreatePatientRequest DTO, but reusing Patient for now.

    let mut patient = Patient::new(
        validation::sanitize_input(&payload.first_name),
        validation::sanitize_input(&payload.last_name),
        payload.date_of_birth,
        payload.gender,
        payload.hospital_admission_date,
        payload.uci_admission_date,
        payload.skin_color,
        validation::sanitize_input(&payload.principal_diagnosis),
        payload.mechanical_ventilation,
        payload.uci_history,
        payload.transfer_from_other_center,
        payload.admission_type,
        payload.invasive_processes,
    );

    // BLINDAJE HADES
    patient = shield_patient(patient);

    match state.db.create("patients").content(patient).await {
        Ok(response) => {
            // saved is Option<Patient>
            let saved: Option<Patient> = response;
            if let Some(p) = &saved {
                if let Some(id) = &p.id {
                    let id_str = format!("{}", id);
                    state.apollo.log_action(
                        &state.db,
                        "CREATE",
                        "patients",
                        &id_str,
                        Some("Created new patient".to_string()),
                        user_id,
                    )
                    .await;
                    
                    // POSEIDON WAVE-SYNC
                    state.poseidon.broadcast(crate::olympus::poseidon::PoseidonEvent::PatientCreated(id_str));
                }
            }
            // DESBLOQUEO PARA EL CLIENTE
            Json(saved.map(unshield_patient))
        }
        Err(e) => {
            tracing::error!("❌ Failed to create patient: {}", e);
            Json(None)
        }
    }
}

/// Handler para obtener todos los pacientes
async fn get_patients(
    State(state): State<AppState>,
) -> Result<Json<Vec<Patient>>, crate::error::AppError> {
    let patients: Vec<Patient> = db
        .select("patients")
        .await
        .map_err(crate::error::AppError::from)?;
    
    // DESBLOQUEO MASIVO PARA EL SISTEMA
    let unshielded_patients = patients.into_iter().map(unshield_patient).collect();
    
    Ok(Json(unshielded_patients))
}

/// Handler to get a single patient by ID
async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<Patient>> {
    let id_thing = id.parse::<RecordId>().ok();
    if let Some(thing) = id_thing {
        match state.db.select(thing).await {
            Ok(patient) => Json(patient.map(unshield_patient)),
            Err(e) => {
                tracing::error!("❌ Failed to fetch patient {}: {}", id, e);
                Json(None)
            }
        }
    } else {
        Json(None)
    }
}

/// Handler to update a patient
async fn update_patient(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
    Json(payload): Json<Patient>,
) -> Json<Option<Patient>> {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());
    let id_thing = id.parse::<RecordId>().ok();
    if let Some(thing) = id_thing {
        // Sanitize payload before update
        let mut sanitized_payload = payload;
        sanitized_payload.first_name = validation::sanitize_input(&sanitized_payload.first_name);
        sanitized_payload.last_name = validation::sanitize_input(&sanitized_payload.last_name);
        sanitized_payload.principal_diagnosis =
            validation::sanitize_input(&sanitized_payload.principal_diagnosis);
        // uci_history is bool, no sanitization needed
        // sanitized_payload.uci_history = validation::sanitize_input(&sanitized_payload.uci_history);

        // BLINDAJE HADES PARA ACTUALIZACIÓN
        let patient_to_save = shield_patient(sanitized_payload);

        // We use .update to replace or merge. .content replaces.
        match state.db.update(thing).content(patient_to_save).await {
            Ok(response) => {
                let saved: Option<Patient> = response;
                state.apollo.log_action(
                    &state.db,
                    "UPDATE",
                    "patients",
                    &id,
                    Some("Updated patient details".to_string()),
                    user_id,
                )
                .await;
                
                // POSEIDON WAVE-SYNC
                state.poseidon.broadcast(crate::olympus::poseidon::PoseidonEvent::PatientUpdated(id.clone()));
                
                Json(saved.map(unshield_patient))
            }
            Err(e) => {
                tracing::error!("❌ Failed to update patient {}: {}", id, e);
                Json(None)
            }
        }
    } else {
        Json(None)
    }
}

/// Handler to delete a patient
async fn delete_patient(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());
    let id_thing = id.parse::<RecordId>().ok();
    if let Some(thing) = id_thing {
        match state.db.delete::<Option<Patient>>(thing).await {
            Ok(_) => {
                state.apollo.log_action(
                    &state.db,
                    "DELETE",
                    "patients",
                    &id,
                    Some("Deleted patient record".to_string()),
                    user_id,
                )
                .await;

                // POSEIDON WAVE-SYNC
                state.poseidon.broadcast(crate::olympus::poseidon::PoseidonEvent::PatientDeleted(id.clone()));

                StatusCode::NO_CONTENT
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete patient {}: {}", id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

/// Check if a patient can perform a specific assessment (24-hour restriction)
async fn check_assessment_eligibility(
    State(state): State<AppState>,
    Path((patient_id, scale_type)): Path<(String, String)>,
) -> Json<validation::ValidationResult> {
    // Validate patient_id format
    if patient_id.parse::<RecordId>().is_err() {
        return Json(validation::ValidationResult {
            can_assess: false,
            hours_since_last: None,
            hours_remaining: None,
            message: Some("Invalid patient ID format".to_string()),
        });
    }

    // Determine table name based on scale type
    let table_name = match scale_type.to_lowercase().as_str() {
        "glasgow" => "glasgow_assessments",
        "apache" => "apache_assessments",
        "sofa" => "sofa_assessments",
        "saps" => "saps_assessments",
        _ => {
            return Json(validation::ValidationResult {
                can_assess: false,
                hours_since_last: None,
                hours_remaining: None,
                message: Some("Invalid scale type".to_string()),
            });
        }
    };

    // Query last assessment of this type for this patient
    let sql = format!(
        "SELECT * FROM {} WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC LIMIT 1",
        table_name
    );

    let mut params = std::collections::BTreeMap::new();
    params.insert("id".to_string(), patient_id.to_string());

    let mut resp = match state.db.query(&sql).bind(params).await {
        Ok(r) => r,
        Err(_) => {
            return Json(validation::ValidationResult {
                can_assess: true, // If query fails, allow assessment
                hours_since_last: None,
                hours_remaining: None,
                message: None,
            });
        }
    };

    // Try to extract assessed_at timestamp
    let result: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();

    if let Some(assessment) = result.first() {
        if let Some(assessed_at) = assessment.get("assessed_at").and_then(|v| v.as_str()) {
            return Json(validation::check_assessment_eligibility(Some(assessed_at)));
        }
    }

    // No previous assessment found - can assess
    Json(validation::ValidationResult {
        can_assess: true,
        hours_since_last: None,
        hours_remaining: None,
        message: None,
    })
}

// Used from uci::models::history::PatientHistoryResponse

/// Handler to get patient history (all assessments)
async fn get_patient_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<PatientHistoryResponse> {
    // Validate ID format first
    if id.parse::<RecordId>().is_err() {
        return Json(PatientHistoryResponse {
            glasgow: vec![],
            apache: vec![],
            sofa: vec![],
            saps: vec![],
        });
    }

    // Run parallel queries? Or sequential for simplicity.
    // We filter by patient_id = $id
    // Note: patient_id is stored as a Thing in the DB.
    // SurrealQL query: SELECT * FROM table WHERE patient_id = type::thing($id)

    let sql_glasgow = "SELECT * FROM glasgow_assessments WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC";
    let sql_apache = "SELECT * FROM apache_assessments WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC";
    let sql_sofa = "SELECT * FROM sofa_assessments WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC";
    let sql_saps = "SELECT * FROM saps_assessments WHERE patient_id = type::thing($id) ORDER BY assessed_at DESC";

    // Helper to fetch
    async fn fetch_records<T: serde::de::DeserializeOwned>(
        db: &Surreal<Client>,
        sql: &str,
        id: &str,
    ) -> Vec<T> {
        let mut params = std::collections::BTreeMap::new();
        params.insert("id".to_string(), id.to_string());

        let mut response = match state.db.query(sql).bind(params).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Query failed: {}", e);
                return vec![];
            }
        };
        response.take(0).unwrap_or_default()
    }

    let (glasgow, apache, sofa, saps) = tokio::join!(
        fetch_records::<GlasgowAssessment>(&state.db, sql_glasgow, &id),
        fetch_records::<ApacheAssessment>(&state.db, sql_apache, &id),
        fetch_records::<SofaAssessment>(&state.db, sql_sofa, &id),
        fetch_records::<SapsAssessment>(&state.db, sql_saps, &id)
    );

    Json(PatientHistoryResponse {
        glasgow,
        apache,
        sofa,
        saps,
    })
}

/// Handler to export patients to CSV
async fn export_patients_csv(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    match state.db.select("patients").await {
        Ok(patients) => {
            // patients is Vec<Patient>
            match uci::services::export::patients_to_csv(patients) {
                Ok(csv_data) => (
                    [
                        (axum::http::header::CONTENT_TYPE, "text/csv"),
                        (
                            axum::http::header::CONTENT_DISPOSITION,
                            "attachment; filename=\"patients.csv\"",
                        ),
                    ],
                    csv_data,
                ),
                Err(_) => (
                    [
                        (axum::http::header::CONTENT_TYPE, "text/plain"),
                        (axum::http::header::CONTENT_DISPOSITION, "inline"),
                    ],
                    "Error generating CSV".to_string(),
                ),
            }
        }
        Err(_) => (
            [
                (axum::http::header::CONTENT_TYPE, "text/plain"),
                (axum::http::header::CONTENT_DISPOSITION, "inline"),
            ],
            "Error fetching patients".to_string(),
        ),
    }
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: crate::olympus::artemis::AuthenticatedUser,
}

/// Handler para el inicio de sesión de usuario
async fn login_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Autenticación mock para desarrollo (admin/admin)
    // En producción, esto debería validar contra la base de datos con contraseñas hasheadas.
    if payload.username == "admin" && payload.password == "admin" {
        let user_id = "user:admin";
        let role = uci::models::user::UserRole::Admin;

        match state.artemis.generate_token(user_id, role.clone()) {
            Ok(token) => Ok(Json(LoginResponse {
                token,
                user: crate::olympus::artemis::AuthenticatedUser {
                    id: user_id.to_string(),
                    role,
                },
            })),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// --- ADMIN HANDLERS ---

/// Helper to ensure the user is an Admin
fn ensure_admin(parts: &axum::http::request::Parts) -> Result<(), StatusCode> {
    let auth_user = parts
        .extensions
        .get::<crate::olympus::artemis::AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_user.role != uci::models::user::UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

async fn get_config(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
) -> Result<Json<SystemConfig>, StatusCode> {
    ensure_admin(&parts)?;
    let configs: Vec<SystemConfig> = db
        .select("system_config")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(configs.into_iter().next().unwrap_or_default()))
}

async fn update_config(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Json(payload): Json<SystemConfig>,
) -> Result<Json<SystemConfig>, StatusCode> {
    ensure_admin(&parts)?;
    let mut config = payload;
    config.updated_at = chrono::Utc::now();

    // We update the record with ID "system_config:settings" or similar to ensure one entry
    let id = RecordId::from(("system_config", "settings"));
    let updated: Option<SystemConfig> = db
        .update(id)
        .content(config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(updated.unwrap_or_default()))
}

async fn get_users(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
) -> Result<Json<Vec<User>>, StatusCode> {
    ensure_admin(&parts)?;
    let users = db
        .select("users")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(users))
}

async fn create_user(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Json(payload): Json<User>,
) -> Result<Json<Option<User>>, StatusCode> {
    ensure_admin(&parts)?;
    let mut user = payload;
    user.created_at = chrono::Utc::now();
    // In a real app we would hash the password here if provided in the DTO
    let created: Option<User> = db
        .create("users")
        .content(user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created))
}

async fn update_user(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
    Json(payload): Json<User>,
) -> Result<Json<Option<User>>, StatusCode> {
    ensure_admin(&parts)?;
    let id_thing = id
        .parse::<RecordId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let updated: Option<User> = db
        .update(id_thing)
        .content(payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(updated))
}

async fn delete_user(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    ensure_admin(&parts)?;
    let id_thing = id
        .parse::<RecordId>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    state.db.delete::<Option<User>>(id_thing)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- ASSESSMENT DELETION HANDLERS ---

/// Handler to delete a Glasgow assessment
async fn delete_glasgow_assessment(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());

    let id_thing = match id.parse::<RecordId>() {
        Ok(thing) => thing,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match state.db.delete::<Option<GlasgowAssessment>>(id_thing).await {
        Ok(_) => {
            state.apollo.log_action(
                &state.db,
                "DELETE",
                "glasgow_assessments",
                &id,
                Some("Deleted Glasgow assessment".to_string()),
                user_id,
            )
            .await;
            StatusCode::NO_CONTENT
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete Glasgow assessment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Handler to delete an Apache assessment
async fn delete_apache_assessment(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());

    let id_thing = match id.parse::<RecordId>() {
        Ok(thing) => thing,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match state.db.delete::<Option<ApacheAssessment>>(id_thing).await {
        Ok(_) => {
            state.apollo.log_action(
                &state.db,
                "DELETE",
                "apache_assessments",
                &id,
                Some("Deleted Apache assessment".to_string()),
                user_id,
            )
            .await;
            StatusCode::NO_CONTENT
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete Apache assessment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Handler to delete a SOFA assessment
async fn delete_sofa_assessment(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());

    let id_thing = match id.parse::<RecordId>() {
        Ok(thing) => thing,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match state.db.delete::<Option<SofaAssessment>>(id_thing).await {
        Ok(_) => {
            state.apollo.log_action(
                &state.db,
                "DELETE",
                "sofa_assessments",
                &id,
                Some("Deleted SOFA assessment".to_string()),
                user_id,
            )
            .await;
            StatusCode::NO_CONTENT
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete SOFA assessment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Handler to delete a SAPS assessment
async fn delete_saps_assessment(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::olympus::artemis::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());

    let id_thing = match id.parse::<RecordId>() {
        Ok(thing) => thing,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match state.db.delete::<Option<SapsAssessment>>(id_thing).await {
        Ok(_) => {
            state.apollo.log_action(
                &state.db,
                "DELETE",
                "saps_assessments",
                &id,
                Some("Deleted SAPS assessment".to_string()),
                user_id,
            )
            .await;
            StatusCode::NO_CONTENT
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete SAPS assessment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// --- POSEIDON WEBSOCKET HANDLER ---
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use futures_util::{sink::SinkExt, stream::StreamExt};

async fn poseidon_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut _receiver) = socket.split();
    let mut rx = state.poseidon.tx.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap_or_default();
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });
}

