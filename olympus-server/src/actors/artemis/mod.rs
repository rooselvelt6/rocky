// src/actors/artemis/mod.rs
// OLYMPUS v16 - Artemis: Diosa de la Caza y Búsqueda (Motor de Búsqueda)
// Implementación sobre Ractor

#![allow(dead_code)]

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tantivy::Index;
use ractor::{Actor, ActorRef, ActorProcessingErr};

use crate::actors::{GodName, DivineDomain};
use crate::traits::{OlympianActor, ActorState, ActorConfig, ActorStatus, GodHeartbeat, HealthStatus};
use crate::traits::message::{ActorMessage, MessagePayload, CommandPayload, QueryPayload, ResponsePayload};
use crate::errors::ActorError;

pub mod schema;
pub mod indexing;
pub mod search;

use crate::actors::artemis::schema::ArtemisSchema;
use crate::actors::artemis::indexing::ArtemisIndexer;
use crate::actors::artemis::search::ArtemisSearcher;

/// Artemis State for Ractor
pub struct ArtemisState {
    pub name: GodName,
    pub metadata: ActorState,
    pub indexer: ArtemisIndexer,
    pub searcher: ArtemisSearcher,
}

pub struct Artemis;

#[async_trait]
impl Actor for Artemis {
    type Msg = ActorMessage;
    type State = ArtemisState;
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        let schema_fields = ArtemisSchema::new();
        let index = Index::create_in_ram(schema_fields.schema.clone());
        
        let indexer = ArtemisIndexer::new(&index, ArtemisSchema::new())
            .map_err(|e| ActorProcessingErr::from(e.to_string()))?;
        let searcher = ArtemisSearcher::new(&index, ArtemisSchema::new())
            .map_err(|e| ActorProcessingErr::from(e.to_string()))?;

        Ok(ArtemisState {
            name: GodName::Artemis,
            metadata: ActorState::new(GodName::Artemis),
            indexer,
            searcher,
        })
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message.payload {
            MessagePayload::Command(cmd) => {
                let res = self.handle_command(cmd, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            MessagePayload::Query(query) => {
                let res = self.handle_query(query, state).await;
                if let Some(reply) = message.reply_to { let _ = reply.send(res); }
            }
            _ => if let Some(reply) = message.reply_to { let _ = reply.send(Ok(ResponsePayload::Ack { message_id: message.id })); }
        }
        Ok(())
    }
}

impl Artemis {
    async fn handle_command(&self, cmd: CommandPayload, state: &mut ArtemisState) -> Result<ResponsePayload, ActorError> {
        match cmd {
            CommandPayload::Custom(data) => {
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");
                if action == "index_patient" {
                    let id = data["id"].as_str().unwrap_or("unknown");
                    let first_name = data["first_name"].as_str().unwrap_or("");
                    let last_name = data["last_name"].as_str().unwrap_or("");
                    let birth_date = data["birth_date"].as_str().unwrap_or("");
                    let clinical_history = data["clinical_history"].as_str().unwrap_or("");
                    let status = data["status"].as_str().unwrap_or("stable");

                    state.indexer.index_patient(
                        id, first_name, last_name, birth_date, clinical_history, status
                    )?;

                    Ok(ResponsePayload::Ack { message_id: "idx_done".to_string() })
                } else {
                    Err(ActorError::InvalidCommand { god: GodName::Artemis, reason: format!("Action '{}' not supported", action) })
                }
            }
            _ => Err(ActorError::InvalidCommand { god: GodName::Artemis, reason: "Command not supported".to_string() }),
        }
    }

    async fn handle_query(&self, query: QueryPayload, state: &ArtemisState) -> Result<ResponsePayload, ActorError> {
        match query {
            QueryPayload::Search { query } => {
                let results = state.searcher.search_patients(&query)?;
                Ok(ResponsePayload::Data { data: serde_json::json!(results) })
            }
            _ => Err(ActorError::InvalidQuery { god: GodName::Artemis, reason: "Query not supported".to_string() }),
        }
    }
}
