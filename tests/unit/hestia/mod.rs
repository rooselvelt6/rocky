// tests/unit/hestia/mod.rs
// Tests unitarios para Hestia - Persistencia y Cache

use olympus::actors::hestia::{Hestia, HestiaConfig, CacheConfig};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

/// Tests de configuración
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_hestia_config() {
        let config = HestiaConfig::default();
        assert!(config.cache_enabled);
        assert_eq!(config.cache_size, 10000);
        assert_eq!(config.ttl_secs, 3600);
        assert!(config.persistence_enabled);
    }
    
    #[test]
    fn test_hestia_config_builder() {
        let config = HestiaConfig::new()
            .with_cache_size(5000)
            .with_ttl(1800)
            .disable_cache()
            .disable_persistence();
            
        assert_eq!(config.cache_size, 5000);
        assert_eq!(config.ttl_secs, 1800);
        assert!(!config.cache_enabled);
        assert!(!config.persistence_enabled);
    }
}

/// Tests de cache
#[cfg(test)]
mod cache_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_set_and_get() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "test-key";
        let value = b"test-value";
        
        hestia.cache_set(key, value, 60).await.expect("Cache set failed");
        
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert_eq!(cached, Some(value.to_vec()));
    }
    
    #[tokio::test]
    async fn test_cache_miss() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "non-existent-key";
        
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert_eq!(cached, None);
    }
    
    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "ttl-key";
        let value = b"ttl-value";
        
        // Set with 1 second TTL
        hestia.cache_set(key, value, 1).await.expect("Cache set failed");
        
        // Should exist immediately
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert!(cached.is_some());
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Should be expired
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert_eq!(cached, None);
    }
    
    #[tokio::test]
    async fn test_cache_delete() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "delete-key";
        let value = b"delete-value";
        
        hestia.cache_set(key, value, 60).await.expect("Cache set failed");
        
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert!(cached.is_some());
        
        hestia.cache_delete(key).await.expect("Cache delete failed");
        
        let cached = hestia.cache_get(key).await.expect("Cache get failed");
        assert_eq!(cached, None);
    }
    
    #[tokio::test]
    async fn test_cache_clear() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Set multiple keys
        for i in 0..10 {
            hestia.cache_set(&format!("key-{}", i), b"value", 60).await
                .expect("Cache set failed");
        }
        
        hestia.cache_clear().await.expect("Cache clear failed");
        
        // All keys should be gone
        for i in 0..10 {
            let cached = hestia.cache_get(&format!("key-{}", i)).await
                .expect("Cache get failed");
            assert_eq!(cached, None);
        }
    }
    
    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let config = HestiaConfig::new()
            .with_cache_size(3); // Very small cache
        
        let hestia = Hestia::with_config(config).await
            .expect("Failed to create Hestia");
        
        // Fill cache
        hestia.cache_set("key1", b"value1", 60).await.unwrap();
        hestia.cache_set("key2", b"value2", 60).await.unwrap();
        hestia.cache_set("key3", b"value3", 60).await.unwrap();
        
        // Add one more - should evict key1 (LRU)
        hestia.cache_set("key4", b"value4", 60).await.unwrap();
        
        let cached1 = hestia.cache_get("key1").await.unwrap();
        let cached4 = hestia.cache_get("key4").await.unwrap();
        
        assert_eq!(cached1, None, "Oldest key should be evicted");
        assert!(cached4.is_some(), "Newest key should exist");
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        let stats_before = hestia.cache_stats().await;
        
        hestia.cache_set("key1", b"value1", 60).await.unwrap();
        let _ = hestia.cache_get("key1").await.unwrap();
        let _ = hestia.cache_get("miss").await.unwrap();
        
        let stats_after = hestia.cache_stats().await;
        
        assert_eq!(stats_after.entries, stats_before.entries + 1);
        assert_eq!(stats_after.hits, stats_before.hits + 1);
        assert_eq!(stats_after.misses, stats_before.misses + 1);
    }
}

/// Tests de persistencia
#[cfg(test)]
mod persistence_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_persist_and_retrieve() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "patient:123";
        let data = serde_json::json!({
            "name": "John Doe",
            "age": 45,
            "condition": "stable"
        });
        
        hestia.persist(key, &data).await.expect("Persist failed");
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }
    
    #[tokio::test]
    async fn test_retrieve_nonexistent() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "non-existent";
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert_eq!(retrieved, None);
    }
    
    #[tokio::test]
    async fn test_delete_persisted() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "delete-test";
        let data = serde_json::json!({"test": "data"});
        
        hestia.persist(key, &data).await.expect("Persist failed");
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert!(retrieved.is_some());
        
        hestia.delete(key).await.expect("Delete failed");
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert_eq!(retrieved, None);
    }
    
    #[tokio::test]
    async fn test_persist_with_ttl() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "ttl-persist";
        let data = serde_json::json!({"temp": "data"});
        
        hestia.persist_with_ttl(key, &data, 1).await.expect("Persist failed");
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert!(retrieved.is_some());
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let retrieved = hestia.retrieve(key).await.expect("Retrieve failed");
        assert_eq!(retrieved, None);
    }
    
    #[tokio::test]
    async fn test_transaction_commit() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        let txn = hestia.begin_transaction().await.expect("Begin transaction failed");
        
        txn.set("key1", &serde_json::json!("value1")).await.unwrap();
        txn.set("key2", &serde_json::json!("value2")).await.unwrap();
        
        txn.commit().await.expect("Commit failed");
        
        let val1 = hestia.retrieve("key1").await.unwrap();
        let val2 = hestia.retrieve("key2").await.unwrap();
        
        assert!(val1.is_some());
        assert!(val2.is_some());
    }
    
    #[tokio::test]
    async fn test_transaction_rollback() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        let txn = hestia.begin_transaction().await.expect("Begin transaction failed");
        
        txn.set("rollback-key", &serde_json::json!("value")).await.unwrap();
        
        txn.rollback().await.expect("Rollback failed");
        
        let val = hestia.retrieve("rollback-key").await.unwrap();
        assert_eq!(val, None);
    }
}

/// Tests de sincronización cache-persistencia
#[cfg(test)]
mod sync_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_through_read() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "sync-test";
        let data = serde_json::json!({"sync": "test"});
        
        // Persist data
        hestia.persist(key, &data).await.unwrap();
        
        // Read through cache (should populate cache)
        let cached = hestia.cache_get(key).await.unwrap();
        assert!(cached.is_none(), "Cache should be empty initially");
        
        let from_persist = hestia.read_through_cache(key).await.unwrap();
        assert_eq!(from_persist, Some(data));
        
        // Now should be in cache
        let cached = hestia.cache_get(key).await.unwrap();
        assert!(cached.is_some(), "Cache should be populated after read-through");
    }
    
    #[tokio::test]
    async fn test_write_through_cache() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "write-through";
        let data = serde_json::json!({"write": "through"});
        
        // Write through (writes to both cache and persistence)
        hestia.write_through(key, &data, 60).await.unwrap();
        
        // Should be in cache
        let cached = hestia.cache_get(key).await.unwrap();
        assert!(cached.is_some());
        
        // Should be in persistence
        let persisted = hestia.retrieve(key).await.unwrap();
        assert_eq!(persisted, Some(data));
    }
    
    #[tokio::test]
    async fn test_cache_invalidation_on_update() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let key = "invalidation-test";
        let data1 = serde_json::json!({"version": 1});
        let data2 = serde_json::json!({"version": 2});
        
        // Initial write
        hestia.write_through(key, &data1, 60).await.unwrap();
        
        // Update (should invalidate cache)
        hestia.persist(key, &data2).await.unwrap();
        hestia.cache_invalidate(key).await.unwrap();
        
        // Cache should be invalidated
        let cached = hestia.cache_get(key).await.unwrap();
        assert_eq!(cached, None);
        
        // But persistence should have new value
        let persisted = hestia.retrieve(key).await.unwrap();
        assert_eq!(persisted, Some(data2));
    }
}

/// Tests de queries
#[cfg(test)]
mod query_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_query_by_field() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Insert test data
        for i in 0..5 {
            let data = serde_json::json!({
                "id": format!("patient-{}", i),
                "name": format!("Patient {}", i),
                "status": if i % 2 == 0 { "active" } else { "inactive" }
            });
            hestia.persist(&format!("patient-{}", i), &data).await.unwrap();
        }
        
        // Query by status
        let active = hestia.query_by_field("status", "active").await.unwrap();
        assert_eq!(active.len(), 3); // patients 0, 2, 4
        
        let inactive = hestia.query_by_field("status", "inactive").await.unwrap();
        assert_eq!(inactive.len(), 2); // patients 1, 3
    }
    
    #[tokio::test]
    async fn test_query_with_pagination() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Insert 10 items
        for i in 0..10 {
            hestia.persist(&format!("item-{}", i), &serde_json::json!({"id": i})).await.unwrap();
        }
        
        // Query page 1 (5 items)
        let page1 = hestia.query_paginated(0, 5).await.unwrap();
        assert_eq!(page1.len(), 5);
        
        // Query page 2 (5 items)
        let page2 = hestia.query_paginated(5, 5).await.unwrap();
        assert_eq!(page2.len(), 5);
    }
}

/// Tests de backup y restore
#[cfg(test)]
mod backup_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_backup_creation() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Insert data
        hestia.persist("key1", &serde_json::json!("value1")).await.unwrap();
        hestia.persist("key2", &serde_json::json!("value2")).await.unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("backup.json");
        
        hestia.backup_to_file(&backup_path).await.expect("Backup failed");
        
        assert!(backup_path.exists());
    }
    
    #[tokio::test]
    async fn test_restore_from_backup() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Insert data
        hestia.persist("restore-key", &serde_json::json!("restore-value")).await.unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("backup.json");
        
        hestia.backup_to_file(&backup_path).await.unwrap();
        
        // Clear data
        hestia.delete("restore-key").await.unwrap();
        
        // Restore
        hestia.restore_from_file(&backup_path).await.expect("Restore failed");
        
        let restored = hestia.retrieve("restore-key").await.unwrap();
        assert!(restored.is_some());
    }
}

/// Tests de performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_cache_performance() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let iterations = 10000;
        
        // Set
        let start = Instant::now();
        for i in 0..iterations {
            hestia.cache_set(&format!("key-{}", i), b"value", 3600).await.unwrap();
        }
        let set_time = start.elapsed();
        
        // Get
        let start = Instant::now();
        for i in 0..iterations {
            let _ = hestia.cache_get(&format!("key-{}", i)).await.unwrap();
        }
        let get_time = start.elapsed();
        
        let sets_per_sec = iterations as f64 / set_time.as_secs_f64();
        let gets_per_sec = iterations as f64 / get_time.as_secs_f64();
        
        assert!(
            sets_per_sec > 10000.0,
            "Cache set too slow: {:.0} ops/sec",
            sets_per_sec
        );
        assert!(
            gets_per_sec > 10000.0,
            "Cache get too slow: {:.0} ops/sec",
            gets_per_sec
        );
    }
    
    #[tokio::test]
    async fn test_persistence_performance() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let iterations = 1000;
        
        let data = serde_json::json!({"test": "data", "nested": {"value": 123}});
        
        let start = Instant::now();
        for i in 0..iterations {
            hestia.persist(&format!("perf-{}", i), &data).await.unwrap();
        }
        let elapsed = start.elapsed();
        
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        assert!(
            ops_per_sec > 100.0,
            "Persistence too slow: {:.0} ops/sec",
            ops_per_sec
        );
    }
}

/// Tests de manejo de errores
#[cfg(test)]
mod error_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_invalid_key_handling() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        // Key vacío
        let result = hestia.cache_set("", b"value", 60).await;
        // Debe manejarlo gracefulmente
        let _ = result;
        
        // Key muy largo
        let long_key = "a".repeat(10000);
        let result = hestia.cache_set(&long_key, b"value", 60).await;
        let _ = result;
    }
    
    #[tokio::test]
    async fn test_concurrent_access() {
        use tokio::task;
        
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        let hestia = std::sync::Arc::new(tokio::sync::Mutex::new(hestia));
        
        let mut handles = vec![];
        
        for i in 0..50 {
            let hestia_clone = hestia.clone();
            let handle = task::spawn(async move {
                hestia_clone.lock().await
                    .cache_set(&format!("concurrent-{}", i), b"value", 60)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        // Verificar que todos los valores están presentes
        for i in 0..50 {
            let cached = hestia.lock().await
                .cache_get(&format!("concurrent-{}", i))
                .await
                .unwrap();
            assert!(cached.is_some());
        }
    }
}

/// Tests de ciclo de vida
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hestia_creation() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        assert_eq!(hestia.name(), GodName::Hestia);
        assert_eq!(hestia.domain(), DivineDomain::Persistence);
    }
    
    #[tokio::test]
    async fn test_hestia_health_check() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        let health = hestia.health_check().await;
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_hestia_metrics() {
        let hestia = Hestia::new().await.expect("Failed to create Hestia");
        
        let metrics = hestia.collect_metrics().await;
        
        assert!(metrics.cache_size >= 0);
        assert!(metrics.persistence_entries >= 0);
        assert!(metrics.memory_usage_bytes >= 0);
    }
}
