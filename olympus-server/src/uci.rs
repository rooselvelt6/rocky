// src/uci.rs
// OLYMPUS v16 - UCI Clinical Scales Module
// Módulo de escalas clínicas de la Unidad de Cuidados Intensivos

pub mod sofa;
pub mod saps;
pub mod glasgow;
pub mod apache;
pub mod news2;

pub use sofa::SofaCalculator;
pub use saps::SapsCalculator;
pub use glasgow::GlasgowCalculator;
pub use apache::ApacheCalculator;
pub use news2::News2Calculator;
