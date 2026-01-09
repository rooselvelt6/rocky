// src/main.rs
// src/main.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn,
    routing::{get, post},
    Json, Router,
};
// use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
// use surrealdb::engine::any::Any;
// use surrealdb::sql::thing; // Deprecated/Incompatible with select in 2.x
use surrealdb::{RecordId, Surreal};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use uci::uci::scale::apache::{ApacheIIRequest, ApacheIIResponse};
use uci::uci::scale::glasgow::{Glasgow, GlasgowRequest, GlasgowResponse};
use uci::uci::scale::saps::{SAPSIIRequest, SAPSIIResponse};
use uci::uci::scale::sofa::{SOFARequest, SOFAResponse};

// Import our new modules
mod audit;
mod auth;
mod db;
mod error;
// mod models; // Moved to lib.rs

use uci::models::apache::ApacheAssessment;
use uci::models::glasgow::GlasgowAssessment;
use uci::models::history::PatientHistoryResponse;
use uci::models::patient::Patient;
use uci::models::saps::SapsAssessment;
use uci::models::sofa::SofaAssessment;

#[cfg(feature = "ssr")]
use uci::services::validation;

// Rate limiting desactivado temporalmente por incompatibilidad con Axum 0.8
// #[cfg(feature = "ssr")]
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Verificamos que la carpeta dist exista (generada por trunk build)
    if !std::path::Path::new("dist").exists() {
        eprintln!("ERROR: No se encuentra la carpeta 'dist/'");
        eprintln!("   Debes ejecutar 'trunk build' primero para compilar el frontend.");
        std::process::exit(1);
    }

    println!("DEBUG: Starting server initialization...");

    // Connect to SurrealDB
    println!("DEBUG: Connecting to DB...");
    let db = match db::connect().await {
        Ok(db) => {
            tracing::info!("✅ Database connection established");
            println!("DEBUG: Login success!");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to SurrealDB: {}", e);
            println!("DEBUG: Login failed: {}", e);
            eprintln!("   Make sure SurrealDB is running: .\\surreal.exe start --user root --pass root file:uci.db");
            std::process::exit(1);
        }
    };

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
        .layer(from_fn(crate::auth::auth_middleware))
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
        .with_state(db); // Add database to app state

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

/// Helper function to check 24-hour restriction for any assessment type
async fn check_24h_restriction<T: serde::de::DeserializeOwned>(
    db: &Surreal<Client>,
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
    State(db): State<Surreal<Client>>,
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
                    check_24h_restriction::<GlasgowAssessment>(&db, p_id, "glasgow_assessments")
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

            match db.create("glasgow_assessments").content(assessment).await {
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
    State(db): State<Surreal<Client>>,
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
                    check_24h_restriction::<ApacheAssessment>(&db, p_id, "apache_assessments").await
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

            match db.create("apache_assessments").content(assessment).await {
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
    State(db): State<Surreal<Client>>,
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
                    check_24h_restriction::<SofaAssessment>(&db, p_id, "sofa_assessments").await
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

            match db.create("sofa_assessments").content(assessment).await {
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
    State(db): State<Surreal<Client>>,
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
                    check_24h_restriction::<SapsAssessment>(&db, p_id, "saps_assessments").await
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

            match db.create("saps_assessments").content(assessment).await {
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

/// Handler para crear un nuevo paciente
async fn create_patient(
    State(db): State<Surreal<Client>>,
    parts: axum::http::request::Parts,
    Json(payload): Json<Patient>,
) -> Json<Option<Patient>> {
    let auth_user = parts.extensions.get::<crate::auth::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());
    // Ensure we are creating a new record with the payload data
    // We might want to overwrite the ID or other fields if they are passed,
    // but for now let's trust the payload or better yet, reconstruct it to ensure safety.
    // Ideally we'd have a CreatePatientRequest DTO, but reusing Patient for now.

    let patient = Patient::new(
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

    match db.create("patients").content(patient).await {
        Ok(response) => {
            // saved is Option<Patient>
            let saved: Option<Patient> = response;
            if let Some(p) = &saved {
                if let Some(id) = &p.id {
                    let id_str = format!("{}", id);
                    crate::audit::log_action(
                        &db,
                        "CREATE",
                        "patients",
                        &id_str,
                        Some("Created new patient".to_string()),
                        user_id,
                    )
                    .await;
                }
            }
            Json(saved)
        }
        Err(e) => {
            tracing::error!("❌ Failed to create patient: {}", e);
            Json(None)
        }
    }
}

/// Handler para obtener todos los pacientes
async fn get_patients(
    State(db): State<Surreal<Client>>,
) -> Result<Json<Vec<Patient>>, crate::error::AppError> {
    let patients = db
        .select("patients")
        .await
        .map_err(crate::error::AppError::from)?;
    Ok(Json(patients))
}

/// Handler to get a single patient by ID
async fn get_patient(
    State(db): State<Surreal<Client>>,
    Path(id): Path<String>,
) -> Json<Option<Patient>> {
    let id_thing = id.parse::<RecordId>().ok();
    if let Some(thing) = id_thing {
        match db.select(thing).await {
            Ok(patient) => Json(patient),
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
    State(db): State<Surreal<Client>>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
    Json(payload): Json<Patient>,
) -> Json<Option<Patient>> {
    let auth_user = parts.extensions.get::<crate::auth::AuthenticatedUser>();
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

        // We use .update to replace or merge. .content replaces.
        match db.update(thing).content(sanitized_payload).await {
            Ok(response) => {
                let saved: Option<Patient> = response;
                crate::audit::log_action(
                    &db,
                    "UPDATE",
                    "patients",
                    &id,
                    Some("Updated patient details".to_string()),
                    user_id,
                )
                .await;
                Json(saved)
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
    State(db): State<Surreal<Client>>,
    parts: axum::http::request::Parts,
    Path(id): Path<String>,
) -> StatusCode {
    let auth_user = parts.extensions.get::<crate::auth::AuthenticatedUser>();
    let user_id = auth_user.map(|u| u.id.clone());
    let id_thing = id.parse::<RecordId>().ok();
    if let Some(thing) = id_thing {
        match db.delete::<Option<Patient>>(thing).await {
            Ok(_) => {
                crate::audit::log_action(
                    &db,
                    "DELETE",
                    "patients",
                    &id,
                    Some("Deleted patient record".to_string()),
                    user_id,
                )
                .await;
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
    State(db): State<Surreal<Client>>,
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

    let mut resp = match db.query(&sql).bind(params).await {
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
    State(db): State<Surreal<Client>>,
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

        let mut response = match db.query(sql).bind(params).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Query failed: {}", e);
                return vec![];
            }
        };
        response.take(0).unwrap_or_default()
    }

    let (glasgow, apache, sofa, saps) = tokio::join!(
        fetch_records::<GlasgowAssessment>(&db, sql_glasgow, &id),
        fetch_records::<ApacheAssessment>(&db, sql_apache, &id),
        fetch_records::<SofaAssessment>(&db, sql_sofa, &id),
        fetch_records::<SapsAssessment>(&db, sql_saps, &id)
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
    State(db): State<Surreal<Client>>,
) -> impl axum::response::IntoResponse {
    match db.select("patients").await {
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
    pub user: crate::auth::AuthenticatedUser,
}

/// Handler para el inicio de sesión de usuario
async fn login_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Autenticación mock para desarrollo (admin/admin)
    // En producción, esto debería validar contra la base de datos con contraseñas hasheadas.
    if payload.username == "admin" && payload.password == "admin" {
        let user_id = "user:admin";
        let role = crate::auth::UserRole::Admin;

        match crate::auth::generate_token(user_id, role.clone()) {
            Ok(token) => Ok(Json(LoginResponse {
                token,
                user: crate::auth::AuthenticatedUser {
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
