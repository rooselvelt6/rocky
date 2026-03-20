// src/uci/glasgow.rs
// OLYMPUS v16 - Glasgow Coma Scale Calculator

#[derive(Debug, Clone)]
pub struct GlasgowCalculator;

impl GlasgowCalculator {
    pub fn calculate(eye: u8, verbal: u8, motor: u8) -> u8 {
        eye.min(4) + verbal.min(5) + motor.min(6)
    }

    pub fn eye_response(score: u8) -> &'static str {
        match score {
            1 => "No response",
            2 => "To pain",
            3 => "To voice",
            4 => "Spontaneous",
            _ => "Unknown",
        }
    }

    pub fn verbal_response(score: u8) -> &'static str {
        match score {
            1 => "No response",
            2 => "Incomprehensible sounds",
            3 => "Inappropriate words",
            4 => "Confused",
            5 => "Oriented",
            _ => "Unknown",
        }
    }

    pub fn motor_response(score: u8) -> &'static str {
        match score {
            1 => "No response",
            2 => "Extension to pain",
            3 => "Abnormal flexion",
            4 => "Flexion withdrawal",
            5 => "Localizes pain",
            6 => "Obeys commands",
            _ => "Unknown",
        }
    }

    pub fn severity(score: u8) -> &'static str {
        match score {
            3..=8 => "Severe",
            9..=12 => "Moderate",
            13..=15 => "Mild",
            _ => "Unknown",
        }
    }
}
