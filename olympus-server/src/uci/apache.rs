// src/uci/apache.rs
// OLYMPUS v16 - APACHE II Score Calculator

#[derive(Debug, Clone)]
pub struct ApacheCalculator;

impl ApacheCalculator {
    pub fn calculate(
        age: u8,
        temp: f64,
        map: u16,
        hr: u16,
        rr: u16,
        pao2_fio2: f64,
        ph: f64,
        na: f64,
        k: f64,
        creatinine: f64,
        hematocrit: f8,
        wbc: f64,
        gcs: u8,
        chronic_health: bool,
    ) -> u8 {
        let mut score: u8 = 0;

        // APS (Acute Physiology Score) components
        // Temperature
        score += match temp {
            t if t < 30.0 => 4,
            t if t < 32.0 => 3,
            t if t < 34.0 => 2,
            t if t < 36.0 => 1,
            t if t < 38.0 => 0,
            t if t < 38.5 => 1,
            t if t < 39.0 => 2,
            t if t < 39.5 => 3,
            _ => 4,
        };

        // GCS
        score += (15 - gcs) as u8;

        // Age
        score += match age {
            a if a < 44 => 0,
            a if a < 54 => 2,
            a if a < 64 => 3,
            a if a < 74 => 5,
            _ => 6,
        };

        // Chronic health points
        if chronic_health {
            score += 5;
        }

        score
    }
}

type f8 = f64;
