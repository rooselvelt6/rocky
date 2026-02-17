use crate::models::config::SystemConfig;
use crate::server_functions::db::get_db;
use leptos::server_fn::ServerFnError;

#[leptos::server(GetSystemConfig, "/api")]
pub async fn get_system_config() -> Result<SystemConfig, ServerFnError> {
    leptos::logging::log!("Server: get_system_config called");
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let results: Vec<SystemConfig> = client
            .query("SELECT * FROM config LIMIT 1")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        Ok(results.into_iter().next().unwrap_or_default())
    } else {
        Ok(SystemConfig::default())
    }
}

#[leptos::server(UpdateSystemConfig, "/api")]
pub async fn update_system_config(config: SystemConfig) -> Result<bool, ServerFnError> {
    leptos::logging::log!("Server: update_system_config called");
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let _: Option<SystemConfig> = client
            .update(("config", "main"))
            .content(config)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        
        Ok(true)
    } else {
        Ok(true)
    }
}

#[leptos::server(GetStats, "/api")]
pub async fn get_stats() -> Result<serde_json::Value, ServerFnError> {
    leptos::logging::log!("Server: get_stats called");
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let patients: Vec<serde_json::Value> = client
            .query("SELECT count() as total FROM patient GROUP ALL")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let glasgow: Vec<serde_json::Value> = client
            .query("SELECT count() as total FROM glasgow GROUP ALL")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let sofa: Vec<serde_json::Value> = client
            .query("SELECT count() as total FROM sofa GROUP ALL")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        let total_patients = patients.first().and_then(|p| p.get("total")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let total_assessments = (glasgow.first().and_then(|g| g.get("total")).and_then(|v| v.as_i64()).unwrap_or(0) 
            + sofa.first().and_then(|s| s.get("total")).and_then(|v| v.as_i64()).unwrap_or(0)) as i32;

        Ok(serde_json::json!({
            "total_patients": total_patients,
            "active_patients": total_patients,
            "total_assessments": total_assessments,
            "critical_patients": 0,
            "stable_patients": total_patients,
            "warning_patients": 0
        }))
    } else {
        Ok(serde_json::json!({
            "total_patients": 24,
            "active_patients": 12,
            "total_assessments": 156,
            "critical_patients": 3,
            "stable_patients": 8,
            "warning_patients": 1
        }))
    }
}

#[leptos::server(GetWardData, "/api")]
pub async fn get_ward_data() -> Result<Vec<serde_json::Value>, ServerFnError> {
    leptos::logging::log!("Server: get_ward_data called");
    
    let db = get_db().await;
    let guard = db.read().await;
    
    if let Some(ref client) = *guard {
        let patients: Vec<serde_json::Value> = client
            .query("SELECT * FROM patient LIMIT 20")
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            .take(0)
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

        Ok(patients)
    } else {
        Ok(vec![
            serde_json::json!({
                "id": "1",
                "first_name": "Juan",
                "last_name": "Pérez",
                "principal_diagnosis": "Neumonía",
                "glasgow": 12,
                "apache": 18,
                "sofa": 6,
                "status": "CRITICAL"
            }),
            serde_json::json!({
                "id": "2", 
                "first_name": "María",
                "last_name": "García",
                "principal_diagnosis": "Postquirúrgico",
                "glasgow": 15,
                "apache": 8,
                "sofa": 2,
                "status": "STABLE"
            }),
        ])
    }
}
