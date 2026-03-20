// src/q1/ward_view.rs
// OLYMPUS v16 - Ward View Mejorado
// Grid de pacientes en tiempo real, alertas visuales, severidad

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientCard {
    pub patient_id: String,
    pub bed_number: String,
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub principal_diagnosis: String,
    pub severity: CardSeverity,
    pub sofa_score: Option<f64>,
    pub saps_score: Option<f64>,
    pub news2_score: Option<f64>,
    pub mechanical_ventilation: bool,
    pub admission_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub alert_status: AlertStatus,
    pub is_critical: bool,
    pub protocols_active: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl CardSeverity {
    pub fn color(&self) -> &str {
        match self {
            CardSeverity::Low => "#22c55e",
            CardSeverity::Medium => "#f59e0b",
            CardSeverity::High => "#f97316",
            CardSeverity::Critical => "#ef4444",
        }
    }

    pub fn from_sofa(sofa: f64) -> Self {
        if sofa >= 12.0 {
            CardSeverity::Critical
        } else if sofa >= 8.0 {
            CardSeverity::High
        } else if sofa >= 4.0 {
            CardSeverity::Medium
        } else {
            CardSeverity::Low
        }
    }

    pub fn from_news2(news2: u8) -> Self {
        if news2 >= 7 {
            CardSeverity::Critical
        } else if news2 >= 5 {
            CardSeverity::High
        } else if news2 >= 3 {
            CardSeverity::Medium
        } else {
            CardSeverity::Low
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    None,
    Watch,
    Warning,
    Critical,
}

impl AlertStatus {
    pub fn should_blink(&self) -> bool {
        matches!(self, AlertStatus::Critical)
    }

    pub fn should_play_sound(&self) -> bool {
        matches!(self, AlertStatus::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardMetrics {
    pub total_beds: usize,
    pub occupied_beds: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub ventilation_count: usize,
    pub average_sofa: f64,
    pub alerts_count: usize,
    pub last_refresh: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardLayout {
    pub beds: Vec<BedPosition>,
    pub grid_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedPosition {
    pub bed_id: String,
    pub row: usize,
    pub column: usize,
    pub patient: Option<PatientCard>,
}

pub struct WardViewManager {
    patients: HashMap<String, PatientCard>,
    beds: HashMap<String, BedPosition>,
    alert_history: Vec<AlertRecord>,
    last_sofa_check: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub timestamp: DateTime<Utc>,
    pub patient_id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    SofaIncrease,
    News2Critical,
    NewAdmission,
    VentilationStarted,
    VentilationStopped,
    Discharge,
    Death,
}

impl WardViewManager {
    pub fn new() -> Self {
        Self {
            patients: HashMap::new(),
            beds: HashMap::new(),
            alert_history: Vec::new(),
            last_sofa_check: HashMap::new(),
        }
    }

    pub fn add_patient(&mut self, patient: PatientCard) {
        let previous_sofa = self.last_sofa_check.get(&patient.patient_id).copied();
        self.patients
            .insert(patient.patient_id.clone(), patient.clone());
        self.last_sofa_check.insert(
            patient.patient_id.clone(),
            patient.sofa_score.unwrap_or(0.0),
        );

        if let Some(prev) = previous_sofa {
            let current = patient.sofa_score.unwrap_or(0.0);
            if current - prev >= 2.0 {
                self.record_alert(AlertRecord {
                    timestamp: Utc::now(),
                    patient_id: patient.patient_id,
                    alert_type: AlertType::SofaIncrease,
                    message: format!("SOFA increased by {} points", (current - prev) as i32),
                    acknowledged: false,
                });
            }
        }
    }

    pub fn remove_patient(&mut self, patient_id: &str) {
        self.patients.remove(patient_id);
        self.last_sofa_check.remove(patient_id);
    }

    pub fn get_patient(&self, patient_id: &str) -> Option<&PatientCard> {
        self.patients.get(patient_id)
    }

    pub fn get_all_patients(&self) -> Vec<&PatientCard> {
        self.patients.values().collect()
    }

    pub fn get_critical_patients(&self) -> Vec<&PatientCard> {
        self.patients.values().filter(|p| p.is_critical).collect()
    }

    pub fn get_metrics(&self) -> WardMetrics {
        let patients: Vec<_> = self.patients.values().collect();
        let critical = patients
            .iter()
            .filter(|p| p.severity == CardSeverity::Critical)
            .count();
        let high = patients
            .iter()
            .filter(|p| p.severity == CardSeverity::High)
            .count();
        let medium = patients
            .iter()
            .filter(|p| p.severity == CardSeverity::Medium)
            .count();
        let low = patients
            .iter()
            .filter(|p| p.severity == CardSeverity::Low)
            .count();
        let ventilation = patients.iter().filter(|p| p.mechanical_ventilation).count();
        let alerts = patients
            .iter()
            .filter(|p| p.alert_status != AlertStatus::None)
            .count();

        let total_sofa: f64 = patients.iter().filter_map(|p| p.sofa_score).sum();
        let avg_sofa = if !patients.is_empty() {
            total_sofa / patients.len() as f64
        } else {
            0.0
        };

        WardMetrics {
            total_beds: 20,
            occupied_beds: patients.len(),
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
            ventilation_count: ventilation,
            average_sofa: avg_sofa,
            alerts_count: alerts,
            last_refresh: Utc::now(),
        }
    }

    pub fn get_unacknowledged_alerts(&self) -> Vec<&AlertRecord> {
        self.alert_history
            .iter()
            .filter(|a| !a.acknowledged)
            .collect()
    }

    pub fn acknowledge_alert(&mut self, patient_id: &str) {
        if let Some(alert) = self
            .alert_history
            .iter_mut()
            .find(|a| a.patient_id == patient_id && !a.acknowledged)
        {
            alert.acknowledged = true;
        }
    }

    fn record_alert(&mut self, alert: AlertRecord) {
        if self.alert_history.len() >= 1000 {
            self.alert_history.remove(0);
        }
        self.alert_history.push(alert);
    }

    pub fn check_news2_alerts(&mut self) {
        for patient in self.patients.values_mut() {
            if let Some(news2) = patient.news2_score {
                if news2 > 7 && patient.alert_status != AlertStatus::Critical {
                    patient.alert_status = AlertStatus::Critical;
                    self.record_alert(AlertRecord {
                        timestamp: Utc::now(),
                        patient_id: patient.patient_id.clone(),
                        alert_type: AlertType::News2Critical,
                        message: format!("NEWS2 critical: {}", news2),
                        acknowledged: false,
                    });
                }
            }
        }
    }

    pub fn update_patient(&mut self, patient_id: &str, updates: PatientUpdate) {
        if let Some(patient) = self.patients.get_mut(patient_id) {
            if let Some(sofa_score) = updates.sofa_score {
                let prev = patient.sofa_score;
                patient.sofa_score = Some(sofa_score);

                if let Some(p) = prev {
                    if sofa_score - p >= 2.0 {
                        patient.alert_status = AlertStatus::Warning;
                        self.record_alert(AlertRecord {
                            timestamp: Utc::now(),
                            patient_id: patient_id.to_string(),
                            alert_type: AlertType::SofaIncrease,
                            message: format!(
                                "SOFA increased by {} points",
                                (sofa_score - p) as i32
                            ),
                            acknowledged: false,
                        });
                    }
                }
            }

            if let Some(news2_score) = updates.news2_score {
                patient.news2_score = Some(news2_score);
                if news2_score > 7 {
                    patient.alert_status = AlertStatus::Critical;
                }
            }

            patient.last_update = Utc::now();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientUpdate {
    pub sofa_score: Option<f64>,
    pub news2_score: Option<f64>,
    pub saps_score: Option<f64>,
    pub alert_status: Option<AlertStatus>,
}

impl Default for WardViewManager {
    fn default() -> Self {
        Self::new()
    }
}
