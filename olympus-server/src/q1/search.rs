// src/q1/search.rs
// OLYMPUS v16 - Búsqueda Avanzada de Pacientes
// Filtros avanzados, ordenamiento, búsqueda en tiempo real

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub query: Option<String>,
    pub patient_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub severity: Option<SeverityLevel>,
    pub admission_type: Option<AdmissionTypeFilter>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub has_mechanical_ventilation: Option<bool>,
    pub has_uci_history: Option<bool>,
    pub principal_diagnosis: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            query: None,
            patient_id: None,
            first_name: None,
            last_name: None,
            severity: None,
            admission_type: None,
            date_from: None,
            date_to: None,
            has_mechanical_ventilation: None,
            has_uci_history: None,
            principal_diagnosis: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionTypeFilter {
    Urgent,
    Programmed,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub admission_date: DateTime<Utc>,
    pub severity: SeverityLevel,
    pub principal_diagnosis: String,
    pub sofa_score: Option<f64>,
    pub saps_score: Option<f64>,
    pub mechanical_ventilation: bool,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub filters_applied: Vec<String>,
}

pub struct PatientSearch {
    index: HashMap<String, SearchIndexEntry>,
}

#[derive(Debug, Clone)]
struct SearchIndexEntry {
    patient_id: String,
    first_name: String,
    last_name: String,
    full_text: String,
    admission_date: DateTime<Utc>,
    principal_diagnosis: String,
}

impl PatientSearch {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn index_patient(&mut self, patient: &SearchIndexEntry) {
        self.index
            .insert(patient.patient_id.clone(), patient.clone());
    }

    pub fn search(&self, filters: SearchFilters) -> SearchResponse {
        let start = std::time::Instant::now();
        let mut results: Vec<SearchResult> = Vec::new();
        let mut filters_applied = Vec::new();

        for entry in self.index.values() {
            let mut matches = true;
            let mut relevance = 0.0;

            if let Some(ref query) = filters.query {
                let query_lower = query.to_lowercase();
                let full_match = entry.full_text.to_lowercase().contains(&query_lower);
                let id_match = entry.patient_id.to_lowercase().contains(&query_lower);
                if full_match || id_match {
                    relevance += 1.0;
                    if id_match {
                        relevance += 0.5;
                    }
                } else {
                    matches = false;
                }
            }

            if let Some(ref first_name) = filters.first_name {
                if !entry
                    .first_name
                    .to_lowercase()
                    .contains(&first_name.to_lowercase())
                {
                    matches = false;
                } else {
                    relevance += 0.3;
                }
            }

            if let Some(ref last_name) = filters.last_name {
                if !entry
                    .last_name
                    .to_lowercase()
                    .contains(&last_name.to_lowercase())
                {
                    matches = false;
                } else {
                    relevance += 0.3;
                }
            }

            if let Some(ref patient_id) = filters.patient_id {
                if !entry.patient_id.contains(patient_id) {
                    matches = false;
                } else {
                    relevance += 0.5;
                }
            }

            if let Some(ref diagnosis) = filters.principal_diagnosis {
                if !entry
                    .principal_diagnosis
                    .to_lowercase()
                    .contains(&diagnosis.to_lowercase())
                {
                    matches = false;
                } else {
                    relevance += 0.4;
                }
            }

            if matches {
                results.push(SearchResult {
                    patient_id: entry.patient_id.clone(),
                    first_name: entry.first_name.clone(),
                    last_name: entry.last_name.clone(),
                    admission_date: entry.admission_date,
                    severity: SeverityLevel::Medium,
                    principal_diagnosis: entry.principal_diagnosis.clone(),
                    sofa_score: None,
                    saps_score: None,
                    mechanical_ventilation: false,
                    relevance_score: relevance,
                });
            }
        }

        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        let total_count = results.len();

        if let Some(offset) = filters.offset {
            results = results.into_iter().skip(offset).collect();
        }
        if let Some(limit) = filters.limit {
            results = results.into_iter().take(limit).collect();
        }

        let query_time_ms = start.elapsed().as_millis() as u64;

        SearchResponse {
            results,
            total_count,
            query_time_ms,
            filters_applied,
        }
    }

    pub fn search_by_severity(&self, severity: SeverityLevel) -> Vec<SearchResult> {
        self.search(SearchFilters {
            severity: Some(severity),
            ..Default::default()
        })
        .results
    }

    pub fn search_by_date_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<SearchResult> {
        self.search(SearchFilters {
            date_from: Some(from),
            date_to: Some(to),
            ..Default::default()
        })
        .results
    }
}

impl Default for PatientSearch {
    fn default() -> Self {
        Self::new()
    }
}
