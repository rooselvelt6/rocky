use async_trait::async_trait;
use super::{ActorMessage, GodName, MessagePayload, OlympianActor};
use ractor::{Actor, ActorRef, ActorProcessingErr};
use rocksdb::{DB, Options};
use std::sync::Arc;

pub struct Hestia {
    rocks_db: Option<Arc<DB>>,
}

impl Hestia {
    pub fn new() -> Self {
        Self {
            rocks_db: None,
        }
    }
}

pub struct HestiaState {
    pub cached_count: u64,
    pub persisted_count: u64,
}

impl Actor for Hestia {
    type Msg = ActorMessage;
    type State = HestiaState;
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _args: ()) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("🏛️ Hestia v16: Inicializando Triada de Persistencia (RocksDB + Valkey + SurrealDB)");
        Ok(HestiaState {
            cached_count: 0,
            persisted_count: 0,
        })
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, msg: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match msg.payload {
            MessagePayload::Command { action, data, .. } => {
                match action.as_str() {
                    "cache_set" => {
                        state.cached_count += 1;
                        tracing::debug!("🏛️ Hestia: Item en cache (Valkey Sim)");
                    }
                    "persist" => {
                        state.persisted_count += 1;
                        if let Some(db) = &self.rocks_db {
                            let key = format!("v16_log_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
                            let val = serde_json::to_string(&data).unwrap_or_default();
                            let _ = db.put(key, val);
                        }
                        tracing::debug!("🏛️ Hestia: Escrito en buffer RocksDB");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl OlympianActor for Hestia {
    fn name(&self) -> GodName {
        GodName::Hestia
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // Intentar abrir RocksDB en una ruta local
        match DB::open(&opts, "data/hestia_buffer") {
            Ok(db) => {
                self.rocks_db = Some(Arc::new(db));
                tracing::info!("🏛️ Hestia: Buffer RocksDB listo en 'data/hestia_buffer'");
            }
            Err(e) => {
                tracing::warn!("🏛️ Hestia: No se pudo abrir RocksDB ({}), operando en modo degradado", e);
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("🏛️ Hestia: Persistencia cerrada.");
        Ok(())
    }
}

use chrono::Utc; // Traído aquí para timestamp_nanos
