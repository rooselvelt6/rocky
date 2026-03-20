// src/uci/news2.rs
// OLYMPUS v16 - NEWS2 Score Calculator

#[derive(Debug, Clone)]
pub struct News2Calculator;

#[derive(Debug, Clone, Copy)]
pub enum ConsciousnessLevel {
    Alert,
    Confusion,
    Voice,
    Pain,
    Unresponsive,
}

#[derive(Debug, Clone, Copy)]
pub enum News2RiskLevel {
    Low,
    LowMedium,
    Medium,
    High,
}

impl News2Calculator {
    pub fn calculate(
        resp_rate: u8,
        spo2_scale_1: u8,
        spo2_scale_2: u8,
        supplemental_o2: bool,
        temperature: f64,
        systolic_bp: u16,
        heart_rate: u8,
        consciousness: ConsciousnessLevel,
    ) -> u8 {
        let mut score: u8 = 0;

        // Respiratory rate
        score += match resp_rate {
            r if r <= 8 => 3,
            r if r <= 11 => 1,
            r if r <= 20 => 0,
            r if r <= 24 => 2,
            _ => 3,
        };

        // SpO2
        if supplemental_o2 {
            score += match spo2_scale_2 {
                s if s <= 83 => 3,
                s if s <= 85 => 2,
                s if s <= 87 => 1,
                s if s <= 89 => 2,
                s if s <= 91 => 1,
                _ => 0,
            };
        } else {
            score += match spo2_scale_1 {
                s if s <= 91 => 3,
                s if s <= 93 => 2,
                s if s <= 95 => 1,
                _ => 0,
            };
        }

        // Temperature
        score += match temperature {
            t if t <= 35.0 => 3,
            t if t <= 36.0 => 1,
            t if t <= 38.0 => 0,
            t if t <= 39.0 => 1,
            _ => 2,
        };

        // Systolic BP
        score += match systolic_bp {
            b if b <= 90 => 3,
            b if b <= 100 => 2,
            b if b <= 110 => 1,
            b if b <= 219 => 0,
            _ => 3,
        };

        // Heart rate
        score += match heart_rate {
            h if h <= 40 => 3,
            h if h <= 50 => 1,
            h if h <= 90 => 0,
            h if h <= 110 => 1,
            h if h <= 130 => 2,
            _ => 3,
        };

        // Consciousness
        score += match consciousness {
            ConsciousnessLevel::Alert => 0,
            ConsciousnessLevel::Confusion => 1,
            _ => 3,
        };

        if supplemental_o2 {
            score += 2;
        }

        score
    }

    pub fn risk_level(score: u8) -> News2RiskLevel {
        match score {
            0..=4 => News2RiskLevel::Low,
            5..=6 => News2RiskLevel::LowMedium,
            7..=8 => News2RiskLevel::Medium,
            _ => News2RiskLevel::High,
        }
    }
}
