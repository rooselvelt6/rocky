use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct News2Assessment {
    pub id: Option<String>,
    pub patient_id: String,
    pub assessed_at: String,

    // Physiological Parameters
    pub respiration_rate: u8,
    pub spo2_scale: u8, // Scale 1 (normal) or Scale 2 (COPD/Hypercapnic)
    pub spo2: u8,
    pub air_or_oxygen: bool, // false for Air, true for Supplemental Oxygen
    pub systolic_bp: u16,
    pub heart_rate: u16,
    pub consciousness: ConsciousnessLevel,
    pub temperature: f32,

    pub score: u8,
    pub risk_level: News2RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsciousnessLevel {
    Alert,
    CVPU, // Confusion, Voice, Pain, Unresponsive
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum News2RiskLevel {
    Low,
    LowMedium, // 3 in a single parameter
    Medium,
    High,
}

impl News2Assessment {
    pub fn calculate_score(&mut self) {
        let mut score = 0;

        // 1. Respiration Rate
        score += match self.respiration_rate {
            ..=8 => 3,
            9..=11 => 1,
            12..=20 => 0,
            21..=24 => 2,
            25.. => 3,
        };

        // 2. SpO2
        if self.spo2_scale == 1 {
            score += match self.spo2 {
                ..=91 => 3,
                92..=93 => 2,
                94..=95 => 1,
                96.. => 0,
            };
        } else {
            // Scale 2 (COPD, Target 88-92%)
            score += match self.spo2 {
                ..=83 => 3,
                84..=85 => 2,
                86..=87 => 1,
                88..=92 => 0,
                93..=94 => 1,
                95..=96 => 2,
                97.. => 3,
            };
        }

        // 3. Supplemental Oxygen
        if self.air_or_oxygen {
            score += 2;
        }

        // 4. Systolic BP
        score += match self.systolic_bp {
            ..=90 => 3,
            91..=100 => 2,
            101..=110 => 1,
            111..=219 => 0,
            220.. => 3,
        };

        // 5. Heart Rate
        score += match self.heart_rate {
            ..=40 => 3,
            41..=50 => 1,
            51..=90 => 0,
            91..=110 => 1,
            111..=130 => 2,
            131.. => 3,
        };

        // 6. Consciousness
        if self.consciousness == ConsciousnessLevel::CVPU {
            score += 3;
        }

        // 7. Temperature
        if self.temperature <= 35.0 {
            score += 3;
        } else if self.temperature <= 36.0 {
            score += 1;
        } else if self.temperature <= 38.0 {
            score += 0;
        } else if self.temperature <= 39.0 {
            score += 1;
        } else {
            score += 2;
        }

        self.score = score;

        // Determine Risk Level
        self.risk_level = if score >= 7 {
            News2RiskLevel::High
        } else if score >= 5 {
            News2RiskLevel::Medium
        } else {
            // Check for red score (3 in a single parameter)
            let mut has_red_score = false;
            // Re-evaluate parameters for "3" points
            if self.respiration_rate <= 8 || self.respiration_rate >= 25 {
                has_red_score = true;
            }
            if self.spo2_scale == 1 && self.spo2 <= 91 {
                has_red_score = true;
            }
            if self.spo2_scale == 2 && (self.spo2 <= 83 || self.spo2 >= 97) {
                has_red_score = true;
            }
            if self.systolic_bp <= 90 || self.systolic_bp >= 220 {
                has_red_score = true;
            }
            if self.heart_rate <= 40 || self.heart_rate >= 131 {
                has_red_score = true;
            }
            if self.consciousness == ConsciousnessLevel::CVPU {
                has_red_score = true;
            }
            if self.temperature <= 35.0 {
                has_red_score = true;
            }

            if score >= 1 && has_red_score {
                News2RiskLevel::LowMedium
            } else if score >= 1 {
                News2RiskLevel::Low
            } else {
                News2RiskLevel::Low
            }
        };
    }
}
