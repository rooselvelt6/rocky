// src/errors/mod.rs
// OLYMPUS v13 - Errors Module

mod actor_error;
mod olympus_error;
mod persistence_error;

pub use actor_error::ActorError;
pub use olympus_error::OlympusError;
pub use persistence_error::PersistenceError;

pub type OlympusResult<T> = Result<T, OlympusError>;
pub type ActorResult<T> = Result<T, ActorError>;
pub type PersistenceResult<T> = Result<T, PersistenceError>;
