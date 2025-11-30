//! Glasgow Coma Scale (GCS) implementation
//!
//! This module provides a complete implementation of the Glasgow Coma Scale,
//! a neurological scale used to assess the level of consciousness in ICU patients.
//!
//! The GCS evaluates three components:
//! - Eye opening response (1-4 points)
//! - Verbal response (1-5 points)
//! - Motor response (1-6 points)
//!
//! Total scores range from 3 (deep coma) to 15 (fully conscious).

/// Eye opening response component of the Glasgow Coma Scale.
///
/// Evaluates the patient's eye opening response to stimuli.
/// Scores range from 1 (no response) to 4 (spontaneous opening).
///
/// # Scoring
/// - **4 points**: Spontaneous eye opening
/// - **3 points**: Eye opening to verbal command
/// - **2 points**: Eye opening to pain
/// - **1 point**: No eye opening response
#[derive(Debug)]
pub enum Eye {
    /// Spontaneous eye opening (4 points)
    Espontaneo,
    /// Eye opening to verbal command (3 points)
    OrdenVerbal,
    /// Eye opening to pain (2 points)
    Dolor,
    /// No eye opening response (1 point)
    NoResponde,
}

impl Eye {
    /// Creates an `Eye` variant from a numeric score.
    ///
    /// # Arguments
    /// * `value` - Numeric score (1-4)
    ///
    /// # Returns
    /// * `Ok(Eye)` - Valid eye response variant
    /// * `Err(String)` - Error if value is outside valid range
    ///
    /// # Examples
    /// ```
    /// let eye = Eye::from_u8(4).unwrap();
    /// assert_eq!(eye.to_u8(), 4);
    /// ```
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            4 => Ok(Eye::Espontaneo),
            3 => Ok(Eye::OrdenVerbal),
            2 => Ok(Eye::Dolor),
            1 => Ok(Eye::NoResponde),
            _ => Err("Invalid value for Eye".to_string()),
        }
    }

    /// Converts the eye response to its numeric score.
    ///
    /// # Returns
    /// Numeric score from 1 to 4
    pub fn to_u8(&self) -> u8 {
        match self {
            Eye::Espontaneo => 4,
            Eye::OrdenVerbal => 3,
            Eye::Dolor => 2,
            Eye::NoResponde => 1,
        }
    }
}

/// Verbal response component of the Glasgow Coma Scale.
///
/// Evaluates the patient's verbal and communication abilities.
/// Scores range from 1 (no response) to 5 (oriented and conversing).
///
/// # Scoring
/// - **5 points**: Oriented and conversing normally
/// - **4 points**: Disoriented but conversing
/// - **3 points**: Inappropriate words
/// - **2 points**: Incomprehensible sounds
/// - **1 point**: No verbal response
#[derive(Debug)]
pub enum Verbal {
    /// Oriented and conversing (5 points)
    OrientadoConversando,
    /// Disoriented but conversing (4 points)
    DesorientadoHablando,
    /// Inappropriate words (3 points)
    PalabrasInapropiadas,
    /// Incomprehensible sounds (2 points)
    SonidosIncomprensibles,
    /// No verbal response (1 point)
    NingunaRespuesta,
}

impl Verbal {
    /// Creates a `Verbal` variant from a numeric score.
    ///
    /// # Arguments
    /// * `value` - Numeric score (1-5)
    ///
    /// # Returns
    /// * `Ok(Verbal)` - Valid verbal response variant
    /// * `Err(String)` - Error if value is outside valid range
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

    /// Converts the verbal response to its numeric score.
    ///
    /// # Returns
    /// Numeric score from 1 to 5
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

/// Motor response component of the Glasgow Coma Scale.
///
/// Evaluates the patient's motor response to stimuli.
/// Scores range from 1 (no response) to 6 (obeys commands).
///
/// # Scoring
/// - **6 points**: Obeys verbal commands
/// - **5 points**: Localizes pain stimulus
/// - **4 points**: Withdrawal from pain
/// - **3 points**: Flexion to pain (decorticate posture)
/// - **2 points**: Extension to pain (decerebrate posture)
/// - **1 point**: No motor response
#[derive(Debug)]
pub enum Motor {
    /// Obeys verbal commands (6 points)
    OrdenVerbalObedece,
    /// Localizes pain (5 points)
    LocalizaElDolor,
    /// Withdrawal and flexion from pain (4 points)
    RetiradaYFlexion,
    /// Abnormal flexion to pain - decorticate posture (3 points)
    FlexionNormal,
    /// Extension to pain - decerebrate posture (2 points)
    Extension,
    /// No motor response (1 point)
    NingunaRespuesta,
}

impl Motor {
    /// Creates a `Motor` variant from a numeric score.
    ///
    /// # Arguments
    /// * `value` - Numeric score (1-6)
    ///
    /// # Returns
    /// * `Ok(Motor)` - Valid motor response variant
    /// * `Err(String)` - Error if value is outside valid range
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

    /// Converts the motor response to its numeric score.
    ///
    /// # Returns
    /// Numeric score from 1 to 6
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

/// Complete Glasgow Coma Scale assessment.
///
/// Represents a full GCS evaluation with all three components:
/// eye opening, verbal response, and motor response.
///
/// # Total Score Interpretation
/// - **15**: Normal consciousness - Alert and Oriented
/// - **13-14**: Mild Traumatic Brain Injury (TBI)
/// - **9-12**: Moderate TBI
/// - **3-8**: Severe TBI
///
/// # Example
/// ```
/// use uci::scale::glasgow::Glasgow;
///
/// let gcs = Glasgow::from_u8(4, 5, 6).unwrap();
/// let score = gcs.score(); // Returns 15
/// let (diagnosis, recommendation) = gcs.result();
/// ```
#[derive(Debug)]
pub struct Glasgow {
    /// Eye opening response component
    pub eye: Eye,
    /// Verbal response component
    pub verbal: Verbal,
    /// Motor response component
    pub motor: Motor,
}

impl Glasgow {
    /// Creates a new Glasgow Coma Scale assessment from numeric scores.
    ///
    /// # Arguments
    /// * `eye` - Eye opening score (1-4)
    /// * `verbal` - Verbal response score (1-5)
    /// * `motor` - Motor response score (1-6)
    ///
    /// # Returns
    /// * `Ok(Glasgow)` - Valid GCS assessment
    /// * `Err(String)` - Error if any value is outside valid range
    ///
    /// # Example
    /// ```
    /// let gcs = Glasgow::from_u8(3, 4, 5)?;
    /// assert_eq!(gcs.score(), 12); // Moderate TBI
    /// ```
    pub fn from_u8(eye: u8, verbal: u8, motor: u8) -> Result<Self, String> {
        Ok(Glasgow {
            eye: Eye::from_u8(eye)?,
            verbal: Verbal::from_u8(verbal)?,
            motor: Motor::from_u8(motor)?,
        })
    }

    /// Calculates the total GCS score.
    ///
    /// # Returns
    /// Total score ranging from 3 (deep coma) to 15 (fully conscious)
    pub fn score(&self) -> u8 {
        self.eye.to_u8() + self.verbal.to_u8() + self.motor.to_u8()
    }

    /// Returns the clinical interpretation and recommendation based on the total score.
    ///
    /// # Returns
    /// A tuple containing:
    /// - Diagnosis/severity classification
    /// - Clinical recommendation
    ///
    /// # Clinical Guidelines
    /// - **Score 15**: No TBI - Patient is alert and oriented
    /// - **Score 13-14**: Mild TBI - Clinical observation or discharge with instructions
    /// - **Score 9-12**: Moderate TBI - Requires CT scan and/or hospitalization
    /// - **Score 3-8**: Severe TBI - Requires immediate resuscitation, airway management, and ICU
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
