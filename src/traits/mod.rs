// src/traits/mod.rs
// OLYMPUS v13 - Traits Module

pub mod actor_trait;
pub mod message;
pub mod supervisor_trait;
pub mod persistable;

pub use actor_trait::{
    OlympianActor,
    ActorState,
    ActorConfig,
    ActorStatus,
    GodHeartbeat,
    HealthStatus,
    SharedActorState,
};

pub use message::{
    ActorMessage,
    MessagePayload,
    MessagePriority,
    CommandPayload,
    QueryPayload,
    EventPayload,
    ResponsePayload,
    RecoveryStrategy,
    SendResult,
    SendStatus,
    DeliveryConfirmation,
};

pub use supervisor_trait::{
    Supervisor,
    Supervisable,
    SupervisionTree,
    SupervisionMetrics,
    SupervisedActor,
    ActorSupervisionStatus,
};

pub use persistable::{
    Persistable,
    PersistenceTransaction,
    PendingTransactions,
    PersistenceError,
    PersistenceResult,
    TransactionStatus,
};
