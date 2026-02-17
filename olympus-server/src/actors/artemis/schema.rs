// src/actors/artemis/schema.rs
// OLYMPUS v13 - Artemis: Esquema de Búsqueda

use tantivy::schema::*;

pub struct ArtemisSchema {
    pub schema: Schema,
    pub patient_id: Field,
    pub first_name: Field,
    pub last_name: Field,
    pub birth_date: Field,
    pub tags: Field,
    pub clinical_history: Field,
    pub status: Field,
}

impl ArtemisSchema {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        // Campos de identificación
        let patient_id = schema_builder.add_text_field("patient_id", STRING | STORED);
        let first_name = schema_builder.add_text_field("first_name", TEXT | STORED);
        let last_name = schema_builder.add_text_field("last_name", TEXT | STORED);
        
        // Campos de metadatos (fechas como texto ISO8601 para simplicidad en búsqueda por texto)
        let birth_date = schema_builder.add_text_field("birth_date", STRING | STORED);
        
        // Tags para búsqueda rápida (ej: "critico", "estable", "vip")
        let tags = schema_builder.add_text_field("tags", TEXT | STORED);
        
        // Texto completo de la historia clínica (con indexación profunda)
        let clinical_history = schema_builder.add_text_field("clinical_history", TEXT);
        
        // Estado actual del paciente
        let status = schema_builder.add_text_field("status", STRING | STORED);

        Self {
            schema: schema_builder.build(),
            patient_id,
            first_name,
            last_name,
            birth_date,
            tags,
            clinical_history,
            status,
        }
    }
}
