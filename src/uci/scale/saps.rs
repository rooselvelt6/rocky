//! SAPS II Score Implementation
//!
//! Simplified Acute Physiology Score II
//! Designed to measure the severity of disease for patients admitted to ICU

use serde::{Deserialize, Serialize};

/// SAPS II complete assessment
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SAPSII {
    pub age: u8,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub temperature: f32,       // °C
    pub pao2_fio2: Option<i32>, // If ventilated or CPAP
    pub urinary_output: f32,    // L/day
    pub serum_urea: f32,        // mmol/L or mg/dl (needs conversion)
    pub white_blood_count: f32, // x10³/mm³
    pub serum_potassium: f32,   // mmol/L
    pub serum_sodium: i32,      // mmol/L
    pub serum_bicarbonate: f32, // mmol/L
    pub bilirubin: f32,         // mg/dl
    pub glasgow_coma_score: u8, // 3-15
    pub chronic_disease: ChronicDisease,
    pub admission_type: AdmissionType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChronicDisease {
    None,
    MetastaticCancer,
    Hematologic,
    AIDS,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AdmissionType {
    ScheduledSurgical,
    Medical,
    UnscheduledSurgical,
}

impl SAPSII {
    /// Calculate SAPS II score (0-163)
    pub fn calculate_score(&self) -> u8 {
        let mut score = 0u8;

        score += self.age_score();
        score += self.heart_rate_score();
        score += self.systolic_bp_score();
        score += self.temperature_score();
        score += self.pao2_fio2_score();
        score += self.urinary_output_score();
        score += self.urea_score();
        score += self.wbc_score();
        score += self.potassium_score();
        score += self.sodium_score();
        score += self.bicarbonate_score();
        score += self.bilirubin_score();
        score += self.glasgow_score();
        score += self.chronic_disease_score();
        score += self.admission_type_score();

        score
    }

    fn age_score(&self) -> u8 {
        match self.age {
            a if a < 40 => 0,
            a if a < 60 => 7,
            a if a < 70 => 12,
            a if a < 75 => 15,
            a if a < 80 => 16,
            _ => 18,
        }
    }

    fn heart_rate_score(&self) -> u8 {
        match self.heart_rate {
            hr if hr < 40 => 11,
            hr if hr < 70 => 2,
            hr if hr < 120 => 0,
            hr if hr < 160 => 4,
            _ => 7,
        }
    }

    fn systolic_bp_score(&self) -> u8 {
        match self.systolic_bp {
            sbp if sbp < 70 => 13,
            sbp if sbp < 100 => 5,
            sbp if sbp < 200 => 0,
            _ => 2,
        }
    }

    fn temperature_score(&self) -> u8 {
        match self.temperature {
            t if t < 39.0 => 0,
            _ => 3,
        }
    }

    fn pao2_fio2_score(&self) -> u8 {
        match self.pao2_fio2 {
            Some(ratio) => match ratio {
                r if r < 100 => 11,
                r if r < 200 => 9,
                _ => 6,
            },
            None => 0,
        }
    }

    fn urinary_output_score(&self) -> u8 {
        match self.urinary_output {
            uo if uo < 0.5 => 11,
            uo if uo < 1.0 => 4,
            _ => 0,
        }
    }

    fn urea_score(&self) -> u8 {
        // Assuming mg/dl
        match self.serum_urea {
            u if u < 28.0 => 0,
            u if u < 84.0 => 6,
            _ => 10,
        }
    }

    fn wbc_score(&self) -> u8 {
        match self.white_blood_count {
            wbc if wbc < 1.0 => 12,
            wbc if wbc < 20.0 => 0,
            _ => 3,
        }
    }

    fn potassium_score(&self) -> u8 {
        match self.serum_potassium {
            k if k < 3.0 => 3,
            k if k < 5.0 => 0,
            _ => 3,
        }
    }

    fn sodium_score(&self) -> u8 {
        match self.serum_sodium {
            na if na < 125 => 5,
            na if na < 145 => 0,
            _ => 1,
        }
    }

    fn bicarbonate_score(&self) -> u8 {
        match self.serum_bicarbonate {
            hco3 if hco3 < 15.0 => 6,
            hco3 if hco3 < 20.0 => 3,
            _ => 0,
        }
    }

    fn bilirubin_score(&self) -> u8 {
        match self.bilirubin {
            bil if bil < 4.0 => 0,
            bil if bil < 6.0 => 4,
            _ => 9,
        }
    }

    fn glasgow_score(&self) -> u8 {
        match self.glasgow_coma_score {
            14..=15 => 0,
            11..=13 => 5,
            9..=10 => 7,
            6..=8 => 13,
            3..=5 => 26,
            _ => 26,
        }
    }

    fn chronic_disease_score(&self) -> u8 {
        match self.chronic_disease {
            ChronicDisease::None => 0,
            ChronicDisease::MetastaticCancer => 9,
            ChronicDisease::Hematologic => 10,
            ChronicDisease::AIDS => 17,
        }
    }

    fn admission_type_score(&self) -> u8 {
        match self.admission_type {
            AdmissionType::ScheduledSurgical => 0,
            AdmissionType::Medical => 6,
            AdmissionType::UnscheduledSurgical => 8,
        }
    }

    /// Calculate predicted mortality (%)
    pub fn predicted_mortality(&self) -> f32 {
        let score = self.calculate_score() as f32;
        // Logistic regression formula: P = e^x / (1 + e^x)
        // where x = -7.7631 + 0.0737*SAPS + 0.9971*ln(SAPS + 1)
        let x = -7.7631 + (0.0737 * score) + (0.9971 * (score + 1.0).ln());
        let mortality = x.exp() / (1.0 + x.exp()) * 100.0;
        mortality.min(99.9)
    }

    /// Get interpretation
    pub fn interpretation(&self) -> (String, String) {
        let score = self.calculate_score();
        let mortality = self.predicted_mortality();

        match score {
            0..=29 => (
                "Bajo riesgo".to_string(),
                format!(
                    "Mortalidad predicha: {:.1}%. Pronóstico favorable.",
                    mortality
                ),
            ),
            30..=49 => (
                "Riesgo moderado".to_string(),
                format!(
                    "Mortalidad predicha: {:.1}%. Requiere monitoreo continuo.",
                    mortality
                ),
            ),
            50..=69 => (
                "Alto riesgo".to_string(),
                format!(
                    "Mortalidad predicha: {:.1}%. Requiere cuidados intensivos.",
                    mortality
                ),
            ),
            _ => (
                "Riesgo muy alto".to_string(),
                format!("Mortalidad predicha: {:.1}%. Pronóstico grave.", mortality),
            ),
        }
    }
}

/// Request payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SAPSIIRequest {
    pub age: u8,
    pub heart_rate: i32,
    pub systolic_bp: i32,
    pub temperature: f32,
    pub pao2_fio2: Option<i32>,
    pub urinary_output: f32,
    pub serum_urea: f32,
    pub white_blood_count: f32,
    pub serum_potassium: f32,
    pub serum_sodium: i32,
    pub serum_bicarbonate: f32,
    pub bilirubin: f32,
    pub glasgow: u8,
    pub chronic_disease: String, // "none", "cancer", "hematologic", "aids"
    pub admission_type: String,  // "scheduled", "medical", "unscheduled"
    pub patient_id: Option<String>,
}

/// Response payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SAPSIIResponse {
    pub score: u8,
    pub predicted_mortality: f32,
    pub severity: String,
    pub recommendation: String,
}

impl SAPSIIRequest {
    pub fn to_saps(&self) -> Result<SAPSII, String> {
        let chronic_disease = match self.chronic_disease.as_str() {
            "none" => ChronicDisease::None,
            "cancer" => ChronicDisease::MetastaticCancer,
            "hematologic" => ChronicDisease::Hematologic,
            "aids" => ChronicDisease::AIDS,
            _ => return Err("Invalid chronic disease type".to_string()),
        };

        let admission_type = match self.admission_type.as_str() {
            "scheduled" => AdmissionType::ScheduledSurgical,
            "medical" => AdmissionType::Medical,
            "unscheduled" => AdmissionType::UnscheduledSurgical,
            _ => return Err("Invalid admission type".to_string()),
        };

        if self.glasgow < 3 || self.glasgow > 15 {
            return Err("Glasgow must be between 3 and 15".to_string());
        }

        Ok(SAPSII {
            age: self.age,
            heart_rate: self.heart_rate,
            systolic_bp: self.systolic_bp,
            temperature: self.temperature,
            pao2_fio2: self.pao2_fio2,
            urinary_output: self.urinary_output,
            serum_urea: self.serum_urea,
            white_blood_count: self.white_blood_count,
            serum_potassium: self.serum_potassium,
            serum_sodium: self.serum_sodium,
            serum_bicarbonate: self.serum_bicarbonate,
            bilirubin: self.bilirubin,
            glasgow_coma_score: self.glasgow,
            chronic_disease,
            admission_type,
        })
    }
}
