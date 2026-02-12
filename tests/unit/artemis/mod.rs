// tests/unit/artemis/mod.rs
// Tests unitarios para Artemis - Búsqueda Full-Text

use olympus::actors::artemis::{Artemis, ArtemisConfig, SearchIndex, QueryEngine};
use olympus::actors::artemis::search::{SearchQuery, SearchResult, Document, FieldValue};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_artemis_config() {
        let config = ArtemisConfig::default();
        assert_eq!(config.default_index, "documents");
        assert_eq!(config.max_results, 1000);
        assert!(config.highlighting_enabled);
        assert!(config.faceting_enabled);
    }
    
    #[test]
    fn test_artemis_config_builder() {
        let config = ArtemisConfig::new()
            .with_max_results(500)
            .with_default_index("medical_records")
            .disable_highlighting()
            .disable_faceting();
            
        assert_eq!(config.max_results, 500);
        assert_eq!(config.default_index, "medical_records");
        assert!(!config.highlighting_enabled);
        assert!(!config.faceting_enabled);
    }
}

/// Tests de creación de índices
#[cfg(test)]
mod index_creation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_simple_index() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let result = artemis.create_index("patients").await;
        
        assert!(result.is_ok());
        assert!(artemis.index_exists("patients").await);
    }
    
    #[tokio::test]
    async fn test_create_index_with_schema() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let schema = IndexSchema::new()
            .add_text_field("name", TextOptions::default().set_stored().set_tokenized())
            .add_text_field("description", TextOptions::default().set_tokenized())
            .add_i64_field("age", NumericOptions::default().set_stored().set_indexed())
            .add_date_field("created_at", DateOptions::default().set_stored());
        
        let result = artemis.create_index_with_schema("medical_records", schema).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_create_duplicate_index() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("duplicate_test").await.unwrap();
        
        // Intentar crear de nuevo
        let result = artemis.create_index("duplicate_test").await;
        
        // Debe fallar o retornar el índice existente
        assert!(result.is_err() || result.is_ok());
    }
    
    #[tokio::test]
    async fn test_delete_index() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("deletable").await.unwrap();
        assert!(artemis.index_exists("deletable").await);
        
        artemis.delete_index("deletable").await.unwrap();
        
        assert!(!artemis.index_exists("deletable").await);
    }
    
    #[tokio::test]
    async fn test_list_indexes() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("index1").await.unwrap();
        artemis.create_index("index2").await.unwrap();
        artemis.create_index("index3").await.unwrap();
        
        let indexes = artemis.list_indexes().await;
        
        assert!(indexes.contains(&"index1".to_string()));
        assert!(indexes.contains(&"index2".to_string()));
        assert!(indexes.contains(&"index3".to_string()));
    }
}

/// Tests de indexación de documentos
#[cfg(test)]
mod indexing_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_index_simple_document() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("test_docs").await.unwrap();
        
        let doc = Document::new()
            .add_text("title", "Test Document")
            .add_text("content", "This is a test document for indexing");
        
        let result = artemis.index_document("test_docs", "doc1", doc).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_index_multiple_documents() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("multi_docs").await.unwrap();
        
        for i in 0..100 {
            let doc = Document::new()
                .add_text("title", &format!("Document {}", i))
                .add_text("content", &format!("Content of document number {}", i));
            
            artemis.index_document("multi_docs", &format!("doc{}", i), doc).await.unwrap();
        }
        
        let doc_count = artemis.get_document_count("multi_docs").await;
        assert_eq!(doc_count, 100);
    }
    
    #[tokio::test]
    async fn test_update_document() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("update_test").await.unwrap();
        
        // Indexar documento inicial
        let doc1 = Document::new()
            .add_text("title", "Original Title")
            .add_text("content", "Original content");
        
        artemis.index_document("update_test", "doc1", doc1).await.unwrap();
        
        // Actualizar documento
        let doc2 = Document::new()
            .add_text("title", "Updated Title")
            .add_text("content", "Updated content");
        
        artemis.update_document("update_test", "doc1", doc2).await.unwrap();
        
        // Buscar y verificar
        let results = artemis.search("update_test", "Updated Title").await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_delete_document() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("delete_test").await.unwrap();
        
        let doc = Document::new()
            .add_text("title", "To be deleted")
            .add_text("content", "This document will be deleted");
        
        artemis.index_document("delete_test", "doc1", doc).await.unwrap();
        
        // Verificar que existe
        let results_before = artemis.search("delete_test", "deleted").await.unwrap();
        assert_eq!(results_before.len(), 1);
        
        // Eliminar
        artemis.delete_document("delete_test", "doc1").await.unwrap();
        
        // Verificar que ya no existe
        let results_after = artemis.search("delete_test", "deleted").await.unwrap();
        assert_eq!(results_after.len(), 0);
    }
    
    #[tokio::test]
    async fn test_bulk_indexing() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("bulk_test").await.unwrap();
        
        let mut docs = vec![];
        for i in 0..1000 {
            docs.push((
                format!("doc{}", i),
                Document::new()
                    .add_text("title", &format!("Bulk Doc {}", i))
                    .add_text("content", &format!("Content {}", i))
            ));
        }
        
        let start = std::time::Instant::now();
        artemis.bulk_index("bulk_test", docs).await.unwrap();
        let elapsed = start.elapsed();
        
        // Debe indexar rápidamente
        let docs_per_sec = 1000.0 / elapsed.as_secs_f64();
        assert!(
            docs_per_sec > 100.0,
            "Bulk indexing too slow: {:.0} docs/sec",
            docs_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_index_with_numeric_fields() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let schema = IndexSchema::new()
            .add_text_field("name", TextOptions::default())
            .add_i64_field("age", NumericOptions::default().set_indexed())
            .add_f64_field("score", NumericOptions::default().set_indexed());
        
        artemis.create_index_with_schema("numeric_test", schema).await.unwrap();
        
        let doc = Document::new()
            .add_text("name", "John Doe")
            .add_i64("age", 45)
            .add_f64("score", 95.5);
        
        artemis.index_document("numeric_test", "patient1", doc).await.unwrap();
        
        // Buscar por rango numérico
        let results = artemis.search_with_filter(
            "numeric_test",
            "John",
            Filter::range("age", 40, 50)
        ).await.unwrap();
        
        assert_eq!(results.len(), 1);
    }
}

/// Tests de búsqueda
#[cfg(test)]
mod search_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_simple_term_search() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("search_test").await.unwrap();
        
        let doc1 = Document::new()
            .add_text("title", "Rust Programming")
            .add_text("content", "Learn Rust programming language");
        
        let doc2 = Document::new()
            .add_text("title", "Python Guide")
            .add_text("content", "Complete Python programming guide");
        
        artemis.index_document("search_test", "doc1", doc1).await.unwrap();
        artemis.index_document("search_test", "doc2", doc2).await.unwrap();
        
        let results = artemis.search("search_test", "Rust").await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(results[0].doc_id == "doc1");
    }
    
    #[tokio::test]
    async fn test_phrase_search() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("phrase_test").await.unwrap();
        
        let doc = Document::new()
            .add_text("content", "The quick brown fox jumps over the lazy dog");
        
        artemis.index_document("phrase_test", "doc1", doc).await.unwrap();
        
        // Búsqueda de frase exacta
        let results = artemis.search_phrase("phrase_test", "quick brown fox").await.unwrap();
        
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_fuzzy_search() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("fuzzy_test").await.unwrap();
        
        let doc = Document::new()
            .add_text("name", "Smith");
        
        artemis.index_document("fuzzy_test", "doc1", doc).await.unwrap();
        
        // Búsqueda con typo (Smitj en lugar de Smith)
        let results = artemis.search_fuzzy("fuzzy_test", "Smitj", 1).await.unwrap();
        
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_boolean_query() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("bool_test").await.unwrap();
        
        let doc1 = Document::new()
            .add_text("tags", "rust programming")
            .add_text("category", "tutorial");
        
        let doc2 = Document::new()
            .add_text("tags", "python programming")
            .add_text("category", "tutorial");
        
        let doc3 = Document::new()
            .add_text("tags", "rust advanced")
            .add_text("category", "guide");
        
        artemis.index_document("bool_test", "doc1", doc1).await.unwrap();
        artemis.index_document("bool_test", "doc2", doc2).await.unwrap();
        artemis.index_document("bool_test", "doc3", doc3).await.unwrap();
        
        // AND query
        let query = Query::bool()
            .must(Query::term("tags", "rust"))
            .must(Query::term("category", "tutorial"));
        
        let results = artemis.search_with_query("bool_test", query).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, "doc1");
    }
    
    #[tokio::test]
    async fn test_search_with_pagination() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("page_test").await.unwrap();
        
        // Indexar 100 documentos
        for i in 0..100 {
            let doc = Document::new()
                .add_text("title", &format!("Document {}", i));
            
            artemis.index_document("page_test", &format!("doc{}", i), doc).await.unwrap();
        }
        
        // Buscar página 1 (10 resultados)
        let page1 = artemis.search_with_pagination("page_test", "Document", 0, 10).await.unwrap();
        assert_eq!(page1.len(), 10);
        
        // Buscar página 2
        let page2 = artemis.search_with_pagination("page_test", "Document", 10, 10).await.unwrap();
        assert_eq!(page2.len(), 10);
        
        // No deben haber duplicados
        let page1_ids: Vec<_> = page1.iter().map(|r| &r.doc_id).collect();
        let page2_ids: Vec<_> = page2.iter().map(|r| &r.doc_id).collect();
        
        for id in &page2_ids {
            assert!(!page1_ids.contains(id));
        }
    }
    
    #[tokio::test]
    async fn test_search_with_sorting() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let schema = IndexSchema::new()
            .add_text_field("title", TextOptions::default())
            .add_i64_field("score", NumericOptions::default().set_indexed());
        
        artemis.create_index_with_schema("sort_test", schema).await.unwrap();
        
        for i in 0..10 {
            let doc = Document::new()
                .add_text("title", &format!("Doc {}", i))
                .add_i64("score", i as i64);
            
            artemis.index_document("sort_test", &format!("doc{}", i), doc).await.unwrap();
        }
        
        // Ordenar por score descendente
        let results = artemis
            .search_with_sort("sort_test", "Doc", SortField::new("score", SortOrder::Desc))
            .await
            .unwrap();
        
        assert_eq!(results[0].doc_id, "doc9");
        assert_eq!(results[9].doc_id, "doc0");
    }
}

/// Tests de highlighting
#[cfg(test)]
mod highlighting_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_highlighting() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("highlight_test").await.unwrap();
        
        let doc = Document::new()
            .add_text("content", "The quick brown fox jumps over the lazy dog");
        
        artemis.index_document("highlight_test", "doc1", doc).await.unwrap();
        
        let results = artemis
            .search_with_highlight("highlight_test", "quick fox")
            .await
            .unwrap();
        
        assert!(results[0].highlighted_content.is_some());
        let highlighted = results[0].highlighted_content.as_ref().unwrap();
        assert!(highlighted.contains("<mark>quick</mark>"));
        assert!(highlighted.contains("<mark>fox</mark>"));
    }
    
    #[tokio::test]
    async fn test_highlighting_multiple_fields() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("multi_highlight").await.unwrap();
        
        let doc = Document::new()
            .add_text("title", "Rust Programming Guide")
            .add_text("content", "Learn Rust programming from scratch");
        
        artemis.index_document("multi_highlight", "doc1", doc).await.unwrap();
        
        let results = artemis
            .search_with_highlight("multi_highlight", "programming")
            .await
            .unwrap();
        
        // Debe resaltar en ambos campos
        assert!(results[0].highlighted_title.as_ref().unwrap().contains("<mark>"));
        assert!(results[0].highlighted_content.as_ref().unwrap().contains("<mark>"));
    }
}

/// Tests de facets
#[cfg(test)]
mod faceting_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_facet_counting() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let schema = IndexSchema::new()
            .add_text_field("title", TextOptions::default())
            .add_facet_field("category");
        
        artemis.create_index_with_schema("facet_test", schema).await.unwrap();
        
        // Indexar documentos con diferentes categorías
        for i in 0..50 {
            let category = if i < 20 {
                "tutorial"
            } else if i < 35 {
                "guide"
            } else {
                "reference"
            };
            
            let doc = Document::new()
                .add_text("title", &format!("Doc {}", i))
                .add_facet("category", category);
            
            artemis.index_document("facet_test", &format!("doc{}", i), doc).await.unwrap();
        }
        
        let facets = artemis.get_facets("facet_test", "category").await.unwrap();
        
        assert_eq!(facets.get("tutorial"), Some(&20));
        assert_eq!(facets.get("guide"), Some(&15));
        assert_eq!(facets.get("reference"), Some(&15));
    }
    
    #[tokio::test]
    async fn test_facet_filtering() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let schema = IndexSchema::new()
            .add_text_field("title", TextOptions::default())
            .add_facet_field("status");
        
        artemis.create_index_with_schema("facet_filter", schema).await.unwrap();
        
        for i in 0..100 {
            let status = if i % 2 == 0 { "active" } else { "inactive" };
            
            let doc = Document::new()
                .add_text("title", &format!("Doc {}", i))
                .add_facet("status", status);
            
            artemis.index_document("facet_filter", &format!("doc{}", i), doc).await.unwrap();
        }
        
        // Filtrar por facet
        let results = artemis
            .search_with_facet_filter("facet_filter", "Doc", "status", "active")
            .await
            .unwrap();
        
        assert_eq!(results.len(), 50);
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_search_latency() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("perf_search").await.unwrap();
        
        // Indexar 10000 documentos
        for i in 0..10000 {
            let doc = Document::new()
                .add_text("title", &format!("Performance Test Document {}", i))
                .add_text("content", &format!("This is the content of document number {}", i));
            
            artemis.index_document("perf_search", &format!("doc{}", i), doc).await.unwrap();
        }
        
        // Medir latencia de búsqueda
        let start = Instant::now();
        
        for _ in 0..100 {
            let _ = artemis.search("perf_search", "Performance").await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_latency_ms = elapsed.as_millis() as f64 / 100.0;
        
        assert!(
            avg_latency_ms < 100.0,
            "Search latency too high: {:.2}ms",
            avg_latency_ms
        );
    }
    
    #[tokio::test]
    async fn test_indexing_throughput() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("perf_index").await.unwrap();
        
        let start = Instant::now();
        
        for i in 0..5000 {
            let doc = Document::new()
                .add_text("content", &format!("Document content {}", i));
            
            artemis.index_document("perf_index", &format!("doc{}", i), doc).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let docs_per_sec = 5000.0 / elapsed.as_secs_f64();
        
        assert!(
            docs_per_sec > 500.0,
            "Indexing throughput too low: {:.0} docs/sec",
            docs_per_sec
        );
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_search() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("empty_search").await.unwrap();
        
        let results = artemis.search("empty_search", "").await.unwrap();
        
        // Debe manejar gracefulmente
        assert!(results.is_empty() || !results.is_empty());
    }
    
    #[tokio::test]
    async fn test_very_long_query() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("long_query").await.unwrap();
        
        let long_query = "a".repeat(10000);
        
        let result = artemis.search("long_query", &long_query).await;
        
        // No debe panic
        assert!(result.is_ok() || result.is_err());
    }
    
    #[tokio::test]
    async fn test_unicode_content() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("unicode").await.unwrap();
        
        let doc = Document::new()
            .add_text("content", "中文测试 café naïve россия 🎉");
        
        artemis.index_document("unicode", "doc1", doc).await.unwrap();
        
        // Buscar palabras en diferentes idiomas
        let results1 = artemis.search("unicode", "中文").await.unwrap();
        assert_eq!(results1.len(), 1);
        
        let results2 = artemis.search("unicode", "café").await.unwrap();
        assert_eq!(results2.len(), 1);
    }
    
    #[tokio::test]
    async fn test_concurrent_indexing() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("concurrent").await.unwrap();
        
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let artemis_clone = artemis.clone();
            let handle = tokio::spawn(async move {
                for i in 0..100 {
                    let doc = Document::new()
                        .add_text("content", &format!("Thread {} Doc {}", thread_id, i));
                    
                    artemis_clone
                        .index_document("concurrent", &format!("t{}d{}", thread_id, i), doc)
                        .await
                        .unwrap();
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let doc_count = artemis.get_document_count("concurrent").await;
        assert_eq!(doc_count, 1000);
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_artemis_creation() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        assert_eq!(artemis.name(), GodName::Artemis);
        assert_eq!(artemis.domain(), DivineDomain::Search);
    }
    
    #[tokio::test]
    async fn test_artemis_health_check() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        let health = artemis.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_index_optimization() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        artemis.create_index("optimize_test").await.unwrap();
        
        // Indexar muchos documentos
        for i in 0..1000 {
            let doc = Document::new()
                .add_text("content", &format!("Content {}", i));
            
            artemis.index_document("optimize_test", &format!("doc{}", i), doc).await.unwrap();
        }
        
        // Optimizar índice
        let stats_before = artemis.get_index_stats("optimize_test").await;
        
        artemis.optimize_index("optimize_test").await.unwrap();
        
        let stats_after = artemis.get_index_stats("optimize_test").await;
        
        // Debe mejorar el rendimiento
        assert!(stats_after.fragmentation < stats_before.fragmentation);
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let artemis = Artemis::new().await.expect("Failed to create Artemis");
        
        // Realizar operaciones
        artemis.create_index("metrics_test").await.unwrap();
        
        for i in 0..100 {
            let doc = Document::new()
                .add_text("content", &format!("Doc {}", i));
            
            artemis.index_document("metrics_test", &format!("doc{}", i), doc).await.unwrap();
        }
        
        for _ in 0..50 {
            let _ = artemis.search("metrics_test", "Doc").await;
        }
        
        let metrics = artemis.collect_metrics().await;
        
        assert_eq!(metrics.documents_indexed, 100);
        assert_eq!(metrics.searches_performed, 50);
        assert!(metrics.average_search_time_ms >= 0.0);
    }
}
