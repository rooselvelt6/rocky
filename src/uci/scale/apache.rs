//! APACHE II Score Implementation
//!
//! Acute Physiology and Chronic Health Evaluation II
//! A widely used severity of disease classification system in ICU patients
//!
//! APACHE II score ranges from 0 to 71 points
//! Higher scores indicate more severe disease and higher risk of mortality

use serde::{Deserialize, Serialize};

/// APACHE II complete assessment
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApacheII {
    // Physiological variables (0-4 points each)
    pub temperature: f32,            // Rectal temperature in °C
    pub mean_arterial_pressure: i32, // MAP in mmHg
    pub heart_rate: i32,             // Heart rate in bpm
    pub respiratory_rate: i32,       // Respiratory rate per minute
    pub oxygenation: ApacheOxygenation,
    pub arterial_ph: f32,
    pub serum_sodium: i32,      // mEq/L
    pub serum_potassium: f32,   // mEq/L
    pub serum_creatinine: f32,  // mg/dL (use double if acute renal failure)
    pub hematocrit: f32,        // %
    pub white_blood_count: f32, // x1000/mm³
    pub glasgow_coma_score: u8, // 3-15 (inverted: 15 - GCS)

    // Age points (0-6 points)
    pub age: u8,

    // Chronic health points (0-5 points)
    pub chronic_health: ChronicHealth,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ApacheOxygenation {
    /// FiO2 >= 0.5: use A-a gradient
    AAGradient(i32), // mmHg
    /// FiO2 < 0.5: use PaO2
    PaO2(i32), // mmHg
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChronicHealth {
    None,
    ElectiveSurgery,    // 2 points
    NonElectiveSurgery, // 5 points
    NonOperative,       // 5 points
}

impl ApacheII {
    /// Calculate APACHE II score
    pub fn calculate_score(&self) -> u8 {
        let mut score = 0u8;

        // Temperature score
        score += self.temperature_score();

        // Mean Arterial Pressure score
        score += self.map_score();

        // Heart Rate score
        score += self.heart_rate_score();

        // Respiratory Rate score
        score += self.respiratory_rate_score();

        // Oxygenation score
        score += self.oxygenation_score();

        // Arterial pH score
        score += self.ph_score();

        // Serum Sodium score
        score += self.sodium_score();

        // Serum Potassium score
        score += self.potassium_score();

        // Serum Creatinine score
        score += self.creatinine_score();

        // Hematocrit score
        score += self.hematocrit_score();

        // White Blood Count score
        score += self.wbc_score();

        // Glasgow Coma Score (inverted)
        score += 15 - self.glasgow_coma_score;

        // Age points
        score += self.age_score();

        // Chronic health points
        score += self.chronic_health_score();

        score
    }

    fn temperature_score(&self) -> u8 {
        match self.temperature {
            t if t >= 41.0 => 4,
            t if t >= 39.0 => 3,
            t if t >= 38.5 => 1,
            t if t >= 36.0 => 0,
            t if t >= 34.0 => 1,
            t if t >= 32.0 => 2,
            t if t >= 30.0 => 3,
            _ => 4,
        }
    }

    fn map_score(&self) -> u8 {
        match self.mean_arterial_pressure {
            map if map >= 160 => 4,
            map if map >= 130 => 3,
            map if map >= 110 => 2,
            map if map >= 70 => 0,
            map if map >= 50 => 2,
            _ => 4,
        }
    }

    fn heart_rate_score(&self) -> u8 {
        match self.heart_rate {
            hr if hr >= 180 => 4,
            hr if hr >= 140 => 3,
            hr if hr >= 110 => 2,
            hr if hr >= 70 => 0,
            hr if hr >= 55 => 2,
            hr if hr >= 40 => 3,
            _ => 4,
        }
    }

    fn respiratory_rate_score(&self) -> u8 {
        match self.respiratory_rate {
            rr if rr >= 50 => 4,
            rr if rr >= 35 => 3,
            rr if rr >= 25 => 1,
            rr if rr >= 12 => 0,
            rr if rr >= 10 => 1,
            rr if rr >= 6 => 2,
            _ => 4,
        }
    }

    fn oxygenation_score(&self) -> u8 {
        match &self.oxygenation {
            ApacheOxygenation::AAGradient(aa) => match aa {
                aa if *aa >= 500 => 4,
                aa if *aa >= 350 => 3,
                aa if *aa >= 200 => 2,
                _ => 0,
            },
            ApacheOxygenation::PaO2(po2) => match po2 {
                po2 if *po2 >= 70 => 0,
                po2 if *po2 >= 61 => 1,
                po2 if *po2 >= 55 => 3,
                _ => 4,
            },
        }
    }

    fn ph_score(&self) -> u8 {
        match self.arterial_ph {
            ph if ph >= 7.70 => 4,
            ph if ph >= 7.60 => 3,
            ph if ph >= 7.50 => 1,
            ph if ph >= 7.33 => 0,
            ph if ph >= 7.25 => 2,
            ph if ph >= 7.15 => 3,
            _ => 4,
        }
    }

    fn sodium_score(&self) -> u8 {
        match self.serum_sodium {
            na if na >= 180 => 4,
            na if na >= 160 => 3,
            na if na >= 155 => 2,
            na if na >= 150 => 1,
            na if na >= 130 => 0,
            na if na >= 120 => 2,
            na if na >= 111 => 3,
            _ => 4,
        }
    }

    fn potassium_score(&self) -> u8 {
        match self.serum_potassium {
            k if k >= 7.0 => 4,
            k if k >= 6.0 => 3,
            k if k >= 5.5 => 1,
            k if k >= 3.5 => 0,
            k if k >= 3.0 => 1,
            k if k >= 2.5 => 2,
            _ => 4,
        }
    }

    fn creatinine_score(&self) -> u8 {
        match self.serum_creatinine {
            cr if cr >= 3.5 => 4,
            cr if cr >= 2.0 => 3,
            cr if cr >= 1.5 => 2,
            cr if cr >= 0.6 => 0,
            _ => 2,
        }
    }

    fn hematocrit_score(&self) -> u8 {
        match self.hematocrit {
            hct if hct >= 60.0 => 4,
            hct if hct >= 50.0 => 2,
            hct if hct >= 46.0 => 1,
            hct if hct >= 30.0 => 0,
            hct if hct >= 20.0 => 2,
            _ => 4,
        }
    }

    fn wbc_score(&self) -> u8 {
        match self.white_blood_count {
            wbc if wbc >= 40.0 => 4,
            wbc if wbc >= 20.0 => 2,
            wbc if wbc >= 15.0 => 1,
            wbc if wbc >= 3.0 => 0,
            wbc if wbc >= 1.0 => 2,
            _ => 4,
        }
    }

    fn age_score(&self) -> u8 {
        match self.age {
            age if age >= 75 => 6,
            age if age >= 65 => 5,
            age if age >= 55 => 3,
            age if age >= 45 => 2,
            _ => 0,
        }
    }

    fn chronic_health_score(&self) -> u8 {
        match self.chronic_health {
            ChronicHealth::None => 0,
            ChronicHealth::ElectiveSurgery => 2,
            ChronicHealth::NonElectiveSurgery => 5,
            ChronicHealth::NonOperative => 5,
        }
    }

    /// Calculate predicted mortality based on APACHE II score
    pub fn predicted_mortality(&self) -> f32 {
        let score = self.calculate_score() as f32;
        // Simplified mortality estimation (approximate formula)
        // Real calculation is more complex and requires additional factors
        match score as u8 {
            s if s < 5 => 4.0,
            s if s < 10 => 8.0,
            s if s < 15 => 15.0,
            s if s < 20 => 25.0,
            s if s < 25 => 40.0,
            s if s < 30 => 55.0,
            s if s < 35 => 73.0,
            _ => 85.0,
        }
    }

    /// Get severity classification
    pub fn severity(&self) -> (String, String) {
        let score = self.calculate_score();
        match score {
            s if s < 10 => (
                "Bajo riesgo".to_string(),
                "Mortalidad predicha < 10%. Monitoreo estándar en UCI.".to_string(),
            ),
            s if s < 15 => (
                "Riesgo moderado".to_string(),
                "Mortalidad predicha 10-25%. Requiere vigilancia estrecha.".to_string(),
            ),
            s if s < 25 => (
                "Alto riesgo".to_string(),
                "Mortalidad predicha 25-55%. Requiere intervención intensiva.".to_string(),
            ),
            s if s < 35 => (
                "Riesgo muy alto".to_string(),
                "Mortalidad predicha 55-85%. Cuidados críticos máximos.".to_string(),
            ),
            _ => (
                "Riesgo extremo".to_string(),
                "Mortalidad predicha > 85%. Pronóstico muy grave.".to_string(),
            ),
        }
    }
}

/// Request payload for APACHE II calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApacheIIRequest {
    pub temperature: f32,
    pub mean_arterial_pressure: i32,
    pub heart_rate: i32,
    pub respiratory_rate: i32,
    pub oxygenation_type: String, // "aa_gradient" or "pao2"
    pub oxygenation_value: i32,
    pub arterial_ph: f32,
    pub serum_sodium: i32,
    pub serum_potassium: f32,
    pub serum_creatinine: f32,
    pub hematocrit: f32,
    pub white_blood_count: f32,
    pub glasgow_coma_score: u8,
    pub age: u8,
    pub chronic_health: String, // "none", "elective", "non_elective", "non_operative"
    pub patient_id: Option<String>,
}

/// Response payload for APACHE II calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApacheIIResponse {
    pub score: u8,
    pub predicted_mortality: f32,
    pub severity: String,
    pub recommendation: String,
}

impl ApacheIIRequest {
    pub fn to_apache(&self) -> Result<ApacheII, String> {
        let oxygenation = match self.oxygenation_type.as_str() {
            "aa_gradient" => ApacheOxygenation::AAGradient(self.oxygenation_value),
            "pao2" => ApacheOxygenation::PaO2(self.oxygenation_value),
            _ => return Err("Invalid oxygenation type".to_string()),
        };

        let chronic_health = match self.chronic_health.as_str() {
            "none" => ChronicHealth::None,
            "elective" => ChronicHealth::ElectiveSurgery,
            "non_elective" => ChronicHealth::NonElectiveSurgery,
            "non_operative" => ChronicHealth::NonOperative,
            _ => return Err("Invalid chronic health type".to_string()),
        };

        if self.glasgow_coma_score < 3 || self.glasgow_coma_score > 15 {
            return Err("Glasgow Coma Score must be between 3 and 15".to_string());
        }

        Ok(ApacheII {
            temperature: self.temperature,
            mean_arterial_pressure: self.mean_arterial_pressure,
            heart_rate: self.heart_rate,
            respiratory_rate: self.respiratory_rate,
            oxygenation,
            arterial_ph: self.arterial_ph,
            serum_sodium: self.serum_sodium,
            serum_potassium: self.serum_potassium,
            serum_creatinine: self.serum_creatinine,
            hematocrit: self.hematocrit,
            white_blood_count: self.white_blood_count,
            glasgow_coma_score: self.glasgow_coma_score,
            age: self.age,
            chronic_health,
        })
    }
}
