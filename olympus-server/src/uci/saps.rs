// src/uci/saps.rs
// OLYMPUS v16 - SAPS II Score Calculator

#[derive(Debug, Clone)]
pub struct SapsCalculator;

impl SapsCalculator {
    pub fn calculate(
        age: u8,
        heart_rate: u16,
        systolic_bp: u16,
        temperature: f64,
        pao2_fio2: f64,
        urine_output: f64,
        bun: f64,
        wbc: f64,
        sodium: f64,
        potassium: f64,
        bicarbonate: f64,
        bilirubin: f64,
        gcs: u8,
        chronic: bool,
    ) -> u8 {
        let mut score: u8 = 0;

        // Age
        score += match age {
            0..=39 => 0,
            40..=59 => 6,
            60..=69 => 11,
            70..=74 => 13,
            75..=79 => 16,
            _ => 18,
        };

        // Heart rate
        score += match heart_rate {
            0..=39 => 11,
            40..=69 => 2,
            70..=119 => 0,
            120..=159 => 4,
            _ => 7,
        };

        // Systolic BP
        score += match systolic_bp {
            0..=69 => 13,
            70..=99 => 5,
            100..=159 => 0,
            160..=199 => 2,
            _ => 3,
        };

        // Temperature
        score += match temperature {
            t if t < 30.0 => 0,
            t if t < 32.0 => 3,
            t if t < 35.0 => 5,
            t if t < 39.0 => 0,
            _ => 3,
        };

        // PaO2/FiO2
        score += match pao2_fio2 {
            p if p < 100.0 => 11,
            p if p < 200.0 => 9,
            p if p < 300.0 => 7,
            _ => 5,
        };

        // GCS
        score += (15 - gcs) as u8;

        // Chronic condition
        if chronic {
            score += 6;
        }

        score
    }
}
