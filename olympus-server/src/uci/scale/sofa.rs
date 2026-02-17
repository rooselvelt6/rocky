//! SOFA Score Implementation
//!
//! Sequential Organ Failure Assessment
//! Used to track a patient's status during ICU stay
//! Scoring system to determine the extent of organ function/failure

use serde::{Deserialize, Serialize};

/// SOFA complete assessment
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SOFA {
    pub respiration_pao2_fio2: i32, // PaO2/FiO2 ratio mmHg
    pub coagulation_platelets: i32, // Platelets ×10³/µl
    pub liver_bilirubin: f32,       // Bilirubin mg/dl
    pub cardiovascular: CardiovascularScore,
    pub cns_glasgow: u8, // Glasgow Coma Score
    pub renal: RenalScore,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CardiovascularScore {
    MAP70OrMore,   // 0 points
    MAPLessThan70, // 1 point
    DopamineLTE5,  // 2 points: <=5 µg/kg/min or dobutamine (any dose)
    DopamineGT5,   // 3 points: >5 µg/kg/min or epi/norepi <= 0.1
    DopamineGT15,  // 4 points: >15 µg/kg/min or epi/norepi > 0.1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RenalScore {
    CreatinineLT1_2,    // 0 points: <1.2 mg/dl
    Creatinine1_2to1_9, // 1 point: 1.2-1.9
    Creatinine2_0to3_4, // 2 points: 2.0-3.4
    Creatinine3_5to4_9, // 3 points: 3.5-4.9
    CreatinineGT5,      // 4 points: ≥5.0
}

impl SOFA {
    /// Calculate total SOFA score (0-24)
    pub fn calculate_score(&self) -> u8 {
        let mut score = 0u8;

        // Respiration
        score += self.respiration_score();

        // Coagulation
        score += self.coagulation_score();

        // Liver
        score += self.liver_score();

        // Cardiovascular
        score += self.cardiovascular_score();

        // CNS (Glasgow)
        score += self.cns_score();

        // Renal
        score += self.renal_score();

        score
    }

    fn respiration_score(&self) -> u8 {
        match self.respiration_pao2_fio2 {
            r if r >= 400 => 0,
            r if r >= 300 => 1,
            r if r >= 200 => 2,
            r if r >= 100 => 3,
            _ => 4,
        }
    }

    fn coagulation_score(&self) -> u8 {
        match self.coagulation_platelets {
            p if p >= 150 => 0,
            p if p >= 100 => 1,
            p if p >= 50 => 2,
            p if p >= 20 => 3,
            _ => 4,
        }
    }

    fn liver_score(&self) -> u8 {
        match self.liver_bilirubin {
            b if b < 1.2 => 0,
            b if b < 2.0 => 1,
            b if b < 6.0 => 2,
            b if b < 12.0 => 3,
            _ => 4,
        }
    }

    fn cardiovascular_score(&self) -> u8 {
        match &self.cardiovascular {
            CardiovascularScore::MAP70OrMore => 0,
            CardiovascularScore::MAPLessThan70 => 1,
            CardiovascularScore::DopamineLTE5 => 2,
            CardiovascularScore::DopamineGT5 => 3,
            CardiovascularScore::DopamineGT15 => 4,
        }
    }

    fn cns_score(&self) -> u8 {
        match self.cns_glasgow {
            15 => 0,
            13..=14 => 1,
            10..=12 => 2,
            6..=9 => 3,
            3..=5 => 4,
            _ => 4,
        }
    }

    fn renal_score(&self) -> u8 {
        match &self.renal {
            RenalScore::CreatinineLT1_2 => 0,
            RenalScore::Creatinine1_2to1_9 => 1,
            RenalScore::Creatinine2_0to3_4 => 2,
            RenalScore::Creatinine3_5to4_9 => 3,
            RenalScore::CreatinineGT5 => 4,
        }
    }

    /// Get interpretation
    pub fn interpretation(&self) -> (String, String) {
        let score = self.calculate_score();
        match score {
            0..=6 => (
                "Falla orgánica leve".to_string(),
                "Mortalidad < 10%. Monitoreo continuo.".to_string(),
            ),
            7..=9 => (
                "Falla orgánica moderada".to_string(),
                "Mortalidad 15-20%. Vigilancia estrecha.".to_string(),
            ),
            10..=12 => (
                "Falla orgánica severa".to_string(),
                "Mortalidad 40-50%. Cuidados intensivos máximos.".to_string(),
            ),
            13..=14 => (
                "Falla orgánica muy severa".to_string(),
                "Mortalidad > 50%. Considerar terapias avanzadas.".to_string(),
            ),
            _ => (
                "Falla orgánica crítica".to_string(),
                "Mortalidad > 80%. Pronóstico muy reservado.".to_string(),
            ),
        }
    }
}

/// Request payload for SOFA calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SOFARequest {
    pub pao2_fio2: i32,
    pub platelets: i32,
    pub bilirubin: f32,
    pub cardiovascular: String, // "map_70_plus", "map_lt_70", "dopa_lte5", "dopa_gt5", "dopa_gt15"
    pub glasgow: u8,
    pub renal: String, // "cr_lt_1_2", "cr_1_2_1_9", "cr_2_0_3_4", "cr_3_5_4_9", "cr_gte_5"
    pub patient_id: Option<String>,
}

/// Response payload for SOFA calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SOFAResponse {
    pub score: u8,
    pub severity: String,
    pub recommendation: String,
}

impl SOFARequest {
    pub fn to_sofa(&self) -> Result<SOFA, String> {
        let cardiovascular = match self.cardiovascular.as_str() {
            "map_70_plus" => CardiovascularScore::MAP70OrMore,
            "map_lt_70" => CardiovascularScore::MAPLessThan70,
            "dopa_lte5" => CardiovascularScore::DopamineLTE5,
            "dopa_gt5" => CardiovascularScore::DopamineGT5,
            "dopa_gt15" => CardiovascularScore::DopamineGT15,
            _ => return Err("Invalid cardiovascular state".to_string()),
        };

        let renal = match self.renal.as_str() {
            "cr_lt_1_2" => RenalScore::CreatinineLT1_2,
            "cr_1_2_1_9" => RenalScore::Creatinine1_2to1_9,
            "cr_2_0_3_4" => RenalScore::Creatinine2_0to3_4,
            "cr_3_5_4_9" => RenalScore::Creatinine3_5to4_9,
            "cr_gte_5" => RenalScore::CreatinineGT5,
            _ => return Err("Invalid renal score".to_string()),
        };

        if self.glasgow < 3 || self.glasgow > 15 {
            return Err("Glasgow must be between 3 and 15".to_string());
        }

        Ok(SOFA {
            respiration_pao2_fio2: self.pao2_fio2,
            coagulation_platelets: self.platelets,
            liver_bilirubin: self.bilirubin,
            cardiovascular,
            cns_glasgow: self.glasgow,
            renal,
        })
    }
}
