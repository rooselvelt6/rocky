// src/q1/export.rs
// OLYMPUS v16 - Exportación de Datos
// Exportación CSV/Excel compatible con SPSS, R

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Tsv,
    Excel,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub data_type: ExportDataType,
    pub patient_ids: Option<Vec<String>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub include_headers: bool,
    pub delimiter: Option<char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportDataType {
    Patients,
    Evaluations,
    LabResults,
    VitalSigns,
    Medications,
    Protocols,
    FullHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientExport {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub gender: String,
    pub admission_date: String,
    pub uci_admission_date: String,
    pub principal_diagnosis: String,
    pub sofa_score: Option<f64>,
    pub saps_score: Option<f64>,
    pub glasgow_score: Option<f64>,
    pub news2_score: Option<f64>,
    pub mechanical_ventilation: bool,
    pub uci_history: bool,
    pub transfer_from_other_center: bool,
    pub admission_type: String,
    pub invasive_processes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationExport {
    pub evaluation_id: String,
    pub patient_id: String,
    pub evaluation_date: String,
    pub scale_type: String,
    pub score: f64,
    pub outcome: Option<String>,
    pub severity: Option<String>,
    pub evaluator: Option<String>,
}

pub struct DataExporter {
    include_pii: bool,
    anonymize: bool,
}

impl DataExporter {
    pub fn new() -> Self {
        Self {
            include_pii: true,
            anonymize: false,
        }
    }

    pub fn anonymize(&mut self) {
        self.include_pii = false;
        self.anonymize = true;
    }

    pub fn export_patients(
        &self,
        patients: &[PatientExport],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        match request.format {
            ExportFormat::Csv => self.export_csv(patients, request),
            ExportFormat::Tsv => self.export_tsv(patients, request),
            ExportFormat::Json => self.export_json(patients),
            ExportFormat::Excel => Err(ExportError::NotSupported(
                "Excel export not yet implemented".to_string(),
            )),
        }
    }

    pub fn export_evaluations(
        &self,
        evaluations: &[EvaluationExport],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        match request.format {
            ExportFormat::Csv => self.export_evaluations_csv(evaluations, request),
            ExportFormat::Tsv => self.export_evaluations_tsv(evaluations, request),
            ExportFormat::Json => self.export_json(evaluations),
            ExportFormat::Excel => Err(ExportError::NotSupported(
                "Excel export not yet implemented".to_string(),
            )),
        }
    }

    fn export_csv<T: serde::Serialize>(
        &self,
        data: &[T],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        let delimiter = request.delimiter.unwrap_or(',');
        let mut output = Vec::new();

        if request.include_headers {
            if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(&data.first()) {
                let headers: Vec<String> = map.keys().cloned().collect();
                writeln!(output, "{}", headers.join(&delimiter.to_string()))
                    .map_err(|e| ExportError::IoError(e.to_string()))?;
            }
        }

        for item in data {
            if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(item) {
                let values: Vec<String> = map
                    .values()
                    .map(|v| self.format_value(v, delimiter))
                    .collect();
                writeln!(output, "{}", values.join(&delimiter.to_string()))
                    .map_err(|e| ExportError::IoError(e.to_string()))?;
            }
        }

        String::from_utf8(output).map_err(|e| ExportError::EncodingError(e.to_string()))
    }

    fn export_tsv<T: serde::Serialize>(
        &self,
        data: &[T],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        let mut req = request.clone();
        req.delimiter = Some('\t');
        self.export_csv(data, &req)
    }

    fn export_evaluations_csv(
        &self,
        evaluations: &[EvaluationExport],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        let delimiter = request.delimiter.unwrap_or(',');
        let mut output = Vec::new();

        if request.include_headers {
            writeln!(output, "evaluation_id{},patient_id{},evaluation_date{},scale_type{},score{},outcome{},severity{},evaluator{}", 
                     delimiter, delimiter, delimiter, delimiter, delimiter, delimiter, delimiter).map_err(|e| ExportError::IoError(e.to_string()))?;
        }

        for eval in evaluations {
            writeln!(
                output,
                "{}{}{}{}{}{}{}{}{}{}{}{}{}",
                eval.evaluation_id,
                delimiter,
                eval.patient_id,
                delimiter,
                eval.evaluation_date,
                delimiter,
                eval.scale_type,
                delimiter,
                eval.score,
                delimiter,
                eval.outcome.as_deref().unwrap_or(""),
                delimiter,
                eval.severity.as_deref().unwrap_or(""),
                delimiter,
                eval.evaluator.as_deref().unwrap_or("")
            )
            .map_err(|e| ExportError::IoError(e.to_string()))?;
        }

        String::from_utf8(output).map_err(|e| ExportError::EncodingError(e.to_string()))
    }

    fn export_evaluations_tsv(
        &self,
        evaluations: &[EvaluationExport],
        request: &ExportRequest,
    ) -> Result<String, ExportError> {
        let mut req = request.clone();
        req.delimiter = Some('\t');
        self.export_evaluations_csv(evaluations, &req)
    }

    fn export_json<T: serde::Serialize>(&self, data: &[T]) -> Result<String, ExportError> {
        serde_json::to_string_pretty(data)
            .map_err(|e| ExportError::SerializationError(e.to_string()))
    }

    fn format_value(&self, value: &serde_json::Value, delimiter: char) -> String {
        match value {
            serde_json::Value::String(s) => {
                if s.contains(delimiter) || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.clone()
                }
            }
            serde_json::Value::Null => String::new(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }

    pub fn generate_spss_syntax(&self, variable_names: &[&str]) -> String {
        let mut syntax = String::from("* SPSS Import Syntax for OLYMPUS UCI Data.\n");
        syntax.push_str("* Generated by OLYMPUS v16.\n\n");

        for (i, name) in variable_names.iter().enumerate() {
            syntax.push_str(&format!("VARIABLE LABELS v{} '{}'.\n", i + 1, name));
        }

        syntax.push_str("\n* End of syntax.\n");
        syntax
    }

    pub fn generate_r_script(&self, variable_names: &[&str]) -> String {
        let mut script = String::from("# R Script for OLYMPUS UCI Data Analysis\n");
        script.push_str("# Generated by OLYMPUS v16\n\n");
        script.push_str("library(ggplot2)\nlibrary(dplyr)\n\n");

        script.push_str("data <- read.csv('olympus_export.csv')\n\n");
        script.push_str("# Summary statistics\n");
        script.push_str("summary(data)\n\n");

        script.push_str("# Plot SOFA over time\n");
        script.push_str("ggplot(data, aes(x=date, y=sofa_score)) + geom_line() + geom_point()\n\n");

        script
    }
}

impl Default for DataExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ExportError {
    IoError(String),
    EncodingError(String),
    SerializationError(String),
    NotSupported(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(msg) => write!(f, "IO Error: {}", msg),
            ExportError::EncodingError(msg) => write!(f, "Encoding Error: {}", msg),
            ExportError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            ExportError::NotSupported(msg) => write!(f, "Not Supported: {}", msg),
        }
    }
}

impl std::error::Error for ExportError {}
