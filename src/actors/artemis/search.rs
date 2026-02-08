// src/actors/artemis/search.rs
// OLYMPUS v13 - Artemis: Lógica de Búsqueda

use tantivy::{IndexReader, Index, collector::TopDocs, query::QueryParser};
use crate::actors::artemis::schema::ArtemisSchema;
use crate::errors::ActorError;
use crate::actors::GodName;
use serde_json::{json, Value};

pub struct ArtemisSearcher {
    reader: IndexReader,
    schema_fields: ArtemisSchema,
}

impl ArtemisSearcher {
    pub fn new(index: &Index, schema_fields: ArtemisSchema) -> Result<Self, ActorError> {
        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommit)
            .try_into()
            .map_err(|e| ActorError::SearchError {
                god: GodName::Artemis,
                message: format!("Failed to create index reader: {}", e),
            })?;

        Ok(Self { reader, schema_fields })
    }

    pub fn search_patients(&self, query_str: &str) -> Result<Vec<Value>, ActorError> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(
            &searcher.index(),
            vec![
                self.schema_fields.first_name,
                self.schema_fields.last_name,
                self.schema_fields.clinical_history,
                self.schema_fields.tags,
            ],
        );

        let query = query_parser.parse_query(query_str).map_err(|e| ActorError::SearchError {
            god: GodName::Artemis,
            message: format!("Invalid search query: {}", e),
        })?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(50)).map_err(|e| ActorError::SearchError {
            god: GodName::Artemis,
            message: format!("Search execution failed: {}", e),
        })?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).map_err(|e| ActorError::SearchError {
                god: GodName::Artemis,
                message: format!("Failed to retrieve document: {}", e),
            })?;
            
            // Convertir documento Tantivy a JSON
            let mut res_obj = json!({});
            for (field, value) in retrieved_doc.field_values() {
                let field_name = searcher.index().schema().get_field_name(*field);
                if let Some(text) = value.as_text() {
                    res_obj[field_name] = json!(text);
                }
            }
            results.push(res_obj);
        }

        Ok(results)
    }
}
