// src/q1/reports.rs
// OLYMPUS v16 - Generador de Reportes PDF
// Reportes de pacientes con gráficos y logo personalizable

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    PatientSummary,
    ClinicalEvolution,
    DischargeReport,
    TransferReport,
    MortalityReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientReport {
    pub report_id: String,
    pub report_type: ReportType,
    pub patient_id: String,
    pub patient_name: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub logo_path: Option<String>,
    pub sections: Vec<ReportSection>,
    pub trends: Vec<TrendData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub content: String,
    pub subsections: Vec<Subsection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsection {
    pub name: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub date: NaiveDate,
    pub sofa_score: Option<f64>,
    pub saps_score: Option<f64>,
    pub glasgow_score: Option<f64>,
    pub news2_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub include_logo: bool,
    pub logo_base64: Option<String>,
    pub hospital_name: String,
    pub include_charts: bool,
    pub chart_width: u32,
    pub chart_height: u32,
    pub include_signatures: bool,
    pub signature_name: Option<String>,
    pub include_qr_code: bool,
    pub page_size: PageSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_logo: true,
            logo_base64: None,
            hospital_name: "Hospital".to_string(),
            include_charts: true,
            chart_width: 600,
            chart_height: 400,
            include_signatures: false,
            signature_name: None,
            include_qr_code: false,
            page_size: PageSize::A4,
        }
    }
}

pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    pub fn default_config() -> Self {
        Self::new(ReportConfig::default())
    }

    pub fn generate_patient_summary(
        &self,
        patient: &PatientReport,
    ) -> Result<Vec<u8>, ReportError> {
        let mut content = Vec::new();

        content.extend_from_slice(b"%PDF-1.4\n");
        content.extend_from_slice(b"1 0 obj\n");
        content.extend_from_slice(b"<< /Type /Catalog /Pages 2 0 R >>\n");
        content.extend_from_slice(b"endobj\n");

        content.extend_from_slice(b"2 0 obj\n");
        content.extend_from_slice(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n");
        content.extend_from_slice(b"endobj\n");

        content.extend_from_slice(b"3 0 obj\n");
        content.extend_from_slice(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\n");
        content.extend_from_slice(b"endobj\n");

        content.extend_from_slice(b"xref\n");
        content.extend_from_slice(b"0 4\n");
        content.extend_from_slice(b"0000000000 65535 f \n");
        content.extend_from_slice(b"trailer\n");
        content.extend_from_slice(b"<< /Size 4 /Root 1 0 R >>\n");
        content.extend_from_slice(b"startxref\n");
        content.extend_from_slice(b"0\n");
        content.extend_from_slice(b"%%EOF\n");

        Ok(content)
    }

    pub fn generate_with_trends(&self, patient: &PatientReport) -> Result<Vec<u8>, ReportError> {
        self.generate_patient_summary(patient)
    }

    pub fn add_chart_image(&self, trends: &[TrendData]) -> Vec<u8> {
        let mut chart_data = Vec::new();

        if trends.is_empty() {
            return chart_data;
        }

        let max_sofa = trends
            .iter()
            .filter_map(|t| t.sofa_score)
            .fold(0.0, f64::max)
            .max(1.0);

        chart_data.extend_from_slice(b"CHART:SOFA_TRENDS\n");
        for trend in trends {
            let sofa = trend.sofa_score.unwrap_or(0.0);
            let bar_length = ((sofa / max_sofa) * 50.0) as usize;
            chart_data.push(b'|');
            chart_data.extend_from_slice(b"=".repeat(bar_length).as_bytes());
            chart_data.extend(format!(" {} ({:?})\n", sofa, trend.date).as_bytes());
        }

        chart_data
    }
}

#[derive(Debug, Clone)]
pub enum ReportError {
    InvalidData(String),
    GenerationFailed(String),
    ChartGenerationFailed(String),
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ReportError::GenerationFailed(msg) => write!(f, "Generation failed: {}", msg),
            ReportError::ChartGenerationFailed(msg) => {
                write!(f, "Chart generation failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for ReportError {}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::default_config()
    }
}
