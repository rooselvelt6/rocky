// server/src/actors/athena.rs
// Athena: Escalas Médicas, ML y Análisis Clínico

use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor, GodHealth};
use chrono::Utc;

pub struct Athena {
    scales_calculated: u64,
    messages_count: u64,
}

impl Athena {
    pub fn new() -> Self {
        Self {
            scales_calculated: 0,
            messages_count: 0,
        }
    }

    fn calculate_glasgow(&mut self, eye: i32, verbal: i32, motor: i32) -> serde_json::Value {
        self.scales_calculated += 1;
        let total = eye + verbal + motor;
        
        let interpretation = match total {
            3..=8 => "Coma severo",
            9..=12 => "Coma moderado",
            13..=15 => "Coma leve/Normal",
            _ => "Error",
        };

        serde_json::json!({
            "eye": eye,
            "verbal": verbal,
            "motor": motor,
            "total": total,
            "interpretation": interpretation,
            "scale": "Glasgow"
        })
    }

    fn calculate_sofa(&mut self, resp: i32, coag: i32, liver: i32, cardio: i32, cns: i32, renal: i32) -> serde_json::Value {
        self.scales_calculated += 1;
        let total = resp + coag + liver + cardio + cns + renal;
        
        let mortality = match total {
            0..=6 => "< 10%",
            7..=9 => "15-20%",
            10..=12 => "40-50%",
            13..=24 => "> 80%",
            _ => "Error",
        };

        serde_json::json!({
            "respiratory": resp,
            "coagulation": coag,
            "liver": liver,
            "cardiovascular": cardio,
            "cns": cns,
            "renal": renal,
            "total": total,
            "predicted_mortality": mortality,
            "scale": "SOFA"
        })
    }

    fn calculate_news2(&mut self, resp_rate: i32, spo2: i32, temp: f32, hr: i32, systolic: i32) -> serde_json::Value {
        self.scales_calculated += 1;
        
        // Simplificación de NEWS2
        let resp_score = match resp_rate {
            0..=8 => 3,
            9..=11 => 1,
            12..=20 => 0,
            21..=24 => 2,
            25..=i32::MAX => 3,
            _ => 0,
        };

        let spo2_score = match spo2 {
            0..=91 => 3,
            92..=93 => 2,
            94..=95 => 1,
            96..=100 => 0,
            _ => 0,
        };

        let temp_score = match temp {
            t if t < 35.0 => 3,
            t if t >= 35.0 && t <= 36.0 => 1,
            t if t > 36.0 && t <= 38.0 => 0,
            t if t > 38.0 && t <= 39.0 => 1,
            t if t > 39.0 => 2,
            _ => 0,
        };

        let hr_score = match hr {
            0..=40 => 3,
            41..=50 => 1,
            51..=90 => 0,
            91..=110 => 1,
            111..=130 => 2,
            131..=i32::MAX => 3,
            _ => 0,
        };

        let bp_score = match systolic {
            0..=90 => 3,
            91..=100 => 2,
            101..=110 => 1,
            111..=219 => 0,
            220..=i32::MAX => 3,
            _ => 0,
        };

        let total = resp_score + spo2_score + temp_score + hr_score + bp_score;

        let risk = match total {
            0..=4 => "Bajo riesgo",
            5..=6 => "Riesgo moderado - revisar urgentemente",
            7..=i32::MAX => "Alto riesgo - respuesta de emergencia",
            _ => "Error",
        };

        serde_json::json!({
            "respiration_score": resp_score,
            "spo2_score": spo2_score,
            "temperature_score": temp_score,
            "heart_rate_score": hr_score,
            "blood_pressure_score": bp_score,
            "total": total,
            "risk_level": risk,
            "scale": "NEWS2"
        })
    }
}

#[async_trait]
impl OlympianActor for Athena {
    fn name(&self) -> GodName {
        GodName::Athena
    }

    async fn handle_message(&mut self, msg: ActorMessage) -> Option<ActorMessage> {
        self.messages_count += 1;

        match &msg.payload {
            MessagePayload::Command { action, data } => {
                let result = match action.as_str() {
                    "calculate_glasgow" => {
                        let eye = data.get("eye")?.as_i64()? as i32;
                        let verbal = data.get("verbal")?.as_i64()? as i32;
                        let motor = data.get("motor")?.as_i64()? as i32;
                        self.calculate_glasgow(eye, verbal, motor)
                    }

                    "calculate_sofa" => {
                        let resp = data.get("respiratory")?.as_i64()? as i32;
                        let coag = data.get("coagulation")?.as_i64()? as i32;
                        let liver = data.get("liver")?.as_i64()? as i32;
                        let cardio = data.get("cardiovascular")?.as_i64()? as i32;
                        let cns = data.get("cns")?.as_i64()? as i32;
                        let renal = data.get("renal")?.as_i64()? as i32;
                        self.calculate_sofa(resp, coag, liver, cardio, cns, renal)
                    }

                    "calculate_news2" => {
                        let resp = data.get("respiration_rate")?.as_i64()? as i32;
                        let spo2 = data.get("oxygen_saturation")?.as_i64()? as i32;
                        let temp = data.get("temperature")?.as_f64()? as f32;
                        let hr = data.get("heart_rate")?.as_i64()? as i32;
                        let systolic = data.get("systolic_bp")?.as_i64()? as i32;
                        self.calculate_news2(resp, spo2, temp, hr, systolic)
                    }

                    _ => return None,
                };

                Some(ActorMessage::new(
                    GodName::Athena,
                    msg.from,
                    MessagePayload::Response {
                        success: true,
                        data: result,
                        error: None,
                    }
                ))
            }

            _ => None
        }
    }

    async fn health(&self) -> GodHealth {
        GodHealth {
            name: GodName::Athena,
            healthy: true,
            last_heartbeat: Utc::now(),
            messages_processed: self.messages_count,
            uptime_seconds: 0,
            status: format!("Analyzed {} scales", self.scales_calculated),
        }
    }

    async fn initialize(&mut self) -> Result<(), String> {
        tracing::info!("🧠 Athena: Inicializando modelos clínicos...");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🧠 Athena: Guardando {} cálculos...", self.scales_calculated);
        Ok(())
    }
}
