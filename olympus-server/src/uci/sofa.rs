// src/uci/sofa.rs
// OLYMPUS v16 - SOFA Score Calculator

#[derive(Debug, Clone)]
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

    pub fn respiratory_score(pao2_fio2: f64, mechanical_ventilation: bool) -> u8 {
        if mechanical_ventilation {
            if pao2_fio2 < 100.0 {
                4
            } else if pao2_fio2 < 200.0 {
                3
            } else if pao2_fio2 < 300.0 {
                2
            } else if pao2_fio2 < 400.0 {
                1
            } else {
                0
            }
        } else {
            if pao2_fio2 < 100.0 {
                4
            } else if pao2_fio2 < 200.0 {
                3
            } else if pao2_fio2 < 300.0 {
                2
            } else {
                0
            }
        }
    }
}
