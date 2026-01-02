use crate::models::apache::ApacheAssessment;
use crate::models::glasgow::GlasgowAssessment;
use crate::models::saps::SapsAssessment;
use crate::models::sofa::SofaAssessment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatientHistoryResponse {
    pub glasgow: Vec<GlasgowAssessment>,
    pub apache: Vec<ApacheAssessment>,
    pub sofa: Vec<SofaAssessment>,
    pub saps: Vec<SapsAssessment>,
}
