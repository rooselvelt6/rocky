#[derive(Debug)]
pub enum Eye {
    Espontaneo,
    OrdenVerbal,
    Dolor,
    NoResponde,
}

impl Eye {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            4 => Ok(Eye::Espontaneo),
            3 => Ok(Eye::OrdenVerbal),
            2 => Ok(Eye::Dolor),
            1 => Ok(Eye::NoResponde),
            _ => Err("Invalid value for Eye".to_string()),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Eye::Espontaneo => 4,
            Eye::OrdenVerbal => 3,
            Eye::Dolor => 2,
            Eye::NoResponde => 1,
        }
    }
}

#[derive(Debug)]
pub enum Verbal {
    OrientadoConversando,
    DesorientadoHablando,
    PalabrasInapropiadas,
    SonidosIncomprensibles,
    NingunaRespuesta,
}

impl Verbal {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            5 => Ok(Verbal::OrientadoConversando),
            4 => Ok(Verbal::DesorientadoHablando),
            3 => Ok(Verbal::PalabrasInapropiadas),
            2 => Ok(Verbal::SonidosIncomprensibles),
            1 => Ok(Verbal::NingunaRespuesta),
            _ => Err("Invalid value for Verbal".to_string()),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Verbal::OrientadoConversando => 5,
            Verbal::DesorientadoHablando => 4,
            Verbal::PalabrasInapropiadas => 3,
            Verbal::SonidosIncomprensibles => 2,
            Verbal::NingunaRespuesta => 1,
        }
    }
}

#[derive(Debug)]
pub enum Motor {
    OrdenVerbalObedece,
    LocalizaElDolor,
    RetiradaYFlexion,
    FlexionNormal,
    Extension,
    NingunaRespuesta,
}

impl Motor {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            6 => Ok(Motor::OrdenVerbalObedece),
            5 => Ok(Motor::LocalizaElDolor),
            4 => Ok(Motor::RetiradaYFlexion),
            3 => Ok(Motor::FlexionNormal),
            2 => Ok(Motor::Extension),
            1 => Ok(Motor::NingunaRespuesta),
            _ => Err("Invalid value for Motor".to_string()),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Motor::OrdenVerbalObedece => 6,
            Motor::LocalizaElDolor => 5,
            Motor::RetiradaYFlexion => 4,
            Motor::FlexionNormal => 3,
            Motor::Extension => 2,
            Motor::NingunaRespuesta => 1,
        }
    }
}

#[derive(Debug)]
pub struct Glasgow {
    pub eye: Eye,
    pub verbal: Verbal,
    pub motor: Motor,
}

impl Glasgow {
    pub fn from_u8(eye: u8, verbal: u8, motor: u8) -> Result<Self, String> {
        Ok(Glasgow {
            eye: Eye::from_u8(eye)?,
            verbal: Verbal::from_u8(verbal)?,
            motor: Motor::from_u8(motor)?,
        })
    }

    pub fn score(&self) -> u8 {
        self.eye.to_u8() + self.verbal.to_u8() + self.motor.to_u8()
    }

    pub fn result(&self) -> (String, String) {
        match self.score() {
            15 => (
                "Sin traumatismo craneoencefálico (TCE)".to_string(),
                "Paciente Alerta y Orientado".to_string(),
            ),

            13..=14 => (
                "Traumatismo Craneoencefálico (TCE) Leve".to_string(),
                "Observación clínica o alta con instrucciones claras".to_string(),
            ),

            9..=12 => (
                "Traumatismo Craneoencefálico (TCE) Moderado".to_string(),
                "Requiere tomografía computarizada (CT) y/u hospitalización.".to_string(),
            ),

            3..=8 => (
                "Traumatismo Craneoencefálico (TCE) Grave".to_string(),
                "Requiere reanimación inmediata, control de la vía aérea (intubación) y UCI."
                    .to_string(),
            ),

            _ => (
                "Puntuación inválida (Fuera de rango 3-15)".to_string(),
                "Puntuación Inválida. Revisa los valores ingresados.".to_string(),
            ),
        }
    }
}
