// src/main.rs
use axum::{extract::State, routing::post, Json, Router};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use uci::uci::scale::apache::{ApacheIIRequest, ApacheIIResponse};
use uci::uci::scale::glasgow::{Glasgow, GlasgowRequest, GlasgowResponse};
use uci::uci::scale::saps::{SAPSIIRequest, SAPSIIResponse};
use uci::uci::scale::sofa::{SOFARequest, SOFAResponse};

// Import our new modules
mod db;
// mod models; // Moved to lib.rs

use uci::models::apache::ApacheAssessment;
use uci::models::glasgow::GlasgowAssessment;
use uci::models::patient::Patient;
use uci::models::saps::SapsAssessment;
use uci::models::sofa::SofaAssessment;

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

    // Connect to SurrealDB
    let db = match db::connect().await {
        Ok(db) => {
            tracing::info!("✅ Database connection established");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to SurrealDB: {}", e);
            eprintln!("   Make sure SurrealDB is running: .\\surreal.exe start --user root --pass root file:uci.db");
            std::process::exit(1);
        }
    };

use tower_http::cors::CorsLayer;

    let app = Router::new()
        // API Endpoints
        .route("/api/glasgow", post(calculate_glasgow))
        .route("/api/apache", post(calculate_apache))
        .route("/api/sofa", post(calculate_sofa))
        .route("/api/saps", post(calculate_saps))
        .route("/api/patients", post(create_patient).get(get_patients))
        // Servir archivos estáticos desde dist
        .fallback_service(
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        )
        .layer(CorsLayer::permissive()) // Enable CORS for all origins
        .with_state(db); // Add database to app state

    println!("¡Servidor Axum arrancando...");
    println!("http://localhost:3000 → Aplicación UCI (Leptos + Axum)");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("No se pudo bindear el puerto 3000 (¿ya está en uso?)");

    println!("¡LISTO! Servidor corriendo en http://localhost:3000");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
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

            // Save to database
            let assessment = GlasgowAssessment::new(
                payload.eye,
                payload.verbal,
                payload.motor,
                glasgow.score(),
                diagnosis,
                recommendation,
            );

            match db.create("glasgow_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Vec<GlasgowAssessment> = saved; // Explicit type annotation
                                                               // saved is a Vec<GlasgowAssessment> because we created on a table
                    if let Some(saved_assessment) = saved.first() {
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
            let score = apache.calculate_score();
            let mortality = apache.predicted_mortality();
            let (severity, recommendation) = apache.severity();

            let response = ApacheIIResponse {
                score,
                predicted_mortality: mortality,
                severity: severity.clone(),
                recommendation: recommendation.clone(),
            };

            // Save to database
            let assessment = ApacheAssessment::new(
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

            match db.create("apache_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Vec<ApacheAssessment> = saved;
                    if let Some(saved_assessment) = saved.first() {
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

            // Save to database
            let assessment = SofaAssessment::new(
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

            match db.create("sofa_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Vec<SofaAssessment> = saved;
                    if let Some(saved_assessment) = saved.first() {
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
            let score = saps.calculate_score();
            let mortality = saps.predicted_mortality();
            let (severity, recommendation) = saps.interpretation();

            let response = SAPSIIResponse {
                score,
                predicted_mortality: mortality,
                severity: severity.clone(),
                recommendation: recommendation.clone(),
            };

            // Save to database
            let assessment = SapsAssessment::new(
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

            match db.create("saps_assessments").content(assessment).await {
                Ok(saved) => {
                    let saved: Vec<SapsAssessment> = saved;
                    if let Some(saved_assessment) = saved.first() {
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
    Json(payload): Json<Patient>,
) -> Json<Option<Patient>> {
    // Ensure we are creating a new record with the payload data
    // We might want to overwrite the ID or other fields if they are passed,
    // but for now let's trust the payload or better yet, reconstruct it to ensure safety.
    // Ideally we'd have a CreatePatientRequest DTO, but reusing Patient for now.

    let patient = Patient::new(
        payload.first_name,
        payload.last_name,
        payload.date_of_birth,
        payload.gender,
        payload.hospital_admission_date,
        payload.uci_admission_date,
        payload.skin_color,
        payload.principal_diagnosis,
        payload.mechanical_ventilation,
        payload.uci_history,
        payload.transfer_from_other_center,
        payload.admission_type,
        payload.invasive_processes,
    );

    match db.create("patients").content(patient).await {
        Ok(saved) => {
            let saved: Vec<Patient> = saved;
            Json(saved.into_iter().next())
        }
        Err(e) => {
            tracing::error!("❌ Failed to create patient: {}", e);
            Json(None)
        }
    }
}

/// Handler para obtener todos los pacientes
async fn get_patients(State(db): State<Surreal<Client>>) -> Json<Vec<Patient>> {
    match db.select("patients").await {
        Ok(patients) => Json(patients),
        Err(e) => {
            tracing::error!("❌ Failed to fetch patients: {}", e);
            Json(vec![])
        }
    }
}
