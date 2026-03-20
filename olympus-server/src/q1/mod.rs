// src/q1/mod.rs
// OLYMPUS v16 - Q1 2026 Features
// Búsqueda avanzada, reportes PDF, dashboard analítica, exportación

pub mod search;
pub mod reports;
pub mod analytics;
pub mod export;
pub mod ward_view;

pub use search::{PatientSearch, SearchFilters, SearchResult};
pub use reports::{ReportGenerator, ReportType, PatientReport};
pub use analytics::{AnalyticsDashboard, DashboardMetrics, ClinicalTrends};
pub use export::{DataExporter, ExportFormat, ExportRequest};
pub use ward_view::{WardViewManager, PatientCard, WardMetrics};
