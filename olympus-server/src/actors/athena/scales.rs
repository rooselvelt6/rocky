// src/actors/athena/scales.rs
// OLYMPUS v16 - Clinical Scale Calculation Engine

#[derive(Debug, Clone)]
pub struct GlasgowCalculator;

impl GlasgowCalculator {
    pub fn calculate(eye: u8, verbal: u8, motor: u8) -> u8 {
        eye.min(4) + verbal.min(5) + motor.min(6)
    }
}

pub struct SofaCalculator;

impl SofaCalculator {
    pub fn calculate(
        respiratory: u8,
        coagulation: u8,
        liver: u8,
        cardiovascular: u8,
        cns: u8,
        renal: u8,
    ) -> u8 {
        respiratory + coagulation + liver + cardiovascular + cns + renal
    }
}

pub struct SapsCalculator;

impl SapsCalculator {
    pub fn calculate(
        age: u8,
        heart_rate: u16,
        systolic_bp: u16,
        temperature: f64,
        gcs: u8,
        chronic: bool,
    ) -> u8 {
        let mut score: u8 = 0;

        score += match age {
            0..=39 => 0,
            40..=59 => 6,
            60..=69 => 11,
            70..=74 => 13,
            75..=79 => 16,
            _ => 18,
        };

        score += match heart_rate {
            0..=39 => 11,
            40..=69 => 2,
            70..=119 => 0,
            120..=159 => 4,
            _ => 7,
        };

        score += match systolic_bp {
            0..=69 => 13,
            70..=99 => 5,
            100..=159 => 0,
            160..=199 => 2,
            _ => 3,
        };

        score += (15 - gcs) as u8;

        if chronic {
            score += 6;
        }

        score
    }
}

pub struct News2Calculator;

impl News2Calculator {
    pub fn calculate(
        resp_rate: u8,
        spo2: u8,
        supplemental_o2: bool,
        temperature: f64,
        systolic_bp: u16,
        heart_rate: u8,
    ) -> u8 {
        let mut score: u8 = 0;

        score += match resp_rate {
            r if r <= 8 => 3,
            r if r <= 11 => 1,
            r if r <= 20 => 0,
            r if r <= 24 => 2,
            _ => 3,
        };

        score += match spo2 {
            s if s <= 91 => 3,
            s if s <= 93 => 2,
            s if s <= 95 => 1,
            _ => 0,
        };

        score += match systolic_bp {
            b if b <= 90 => 3,
            b if b <= 100 => 2,
            b if b <= 110 => 1,
            _ => 0,
        };

        if supplemental_o2 {
            score += 2;
        }

        score
    }
}
