// src/actors/artemis/indexing.rs
// OLYMPUS v13 - Artemis: Lógica de Indexación

use tantivy::{Index, IndexWriter, TantivyDocument};
use crate::actors::artemis::schema::ArtemisSchema;
use crate::errors::ActorError;
use crate::actors::GodName;

pub struct ArtemisIndexer {
    writer: IndexWriter,
    schema_fields: ArtemisSchema,
}

impl ArtemisIndexer {
    pub fn new(index: &Index, schema_fields: ArtemisSchema) -> Result<Self, ActorError> {
        let writer = index
            .writer(50_000_000) // 50MB heap
            .map_err(|e| ActorError::SearchError {
                god: GodName::Artemis,
                message: format!("Failed to create index writer: {}", e),
            })?;

        Ok(Self { writer, schema_fields })
    }

    pub fn index_patient(
        &mut self,
        id: &str,
        first_name: &str,
        last_name: &str,
        birth_date: &str,
        clinical_history: &str,
        status: &str,
    ) -> Result<(), ActorError> {
        let mut doc = TantivyDocument::default();
        doc.add_text(self.schema_fields.patient_id, id);
        doc.add_text(self.schema_fields.first_name, first_name);
        doc.add_text(self.schema_fields.last_name, last_name);
        doc.add_text(self.schema_fields.birth_date, birth_date);
        doc.add_text(self.schema_fields.clinical_history, clinical_history);
        doc.add_text(self.schema_fields.status, status);

        self.writer.add_document(doc).map_err(|e| ActorError::SearchError {
            god: GodName::Artemis,
            message: format!("Failed to add document: {}", e),
        })?;

        // Committeamos para que sea visible
        self.writer.commit().map_err(|e| ActorError::SearchError {
            god: GodName::Artemis,
            message: format!("Failed to commit index: {}", e),
        })?;

        Ok(())
    }
}
