// src/actors/hermes/router.rs
// OLYMPUS v13 - Hermes Message Router
// Routing de mensajes por patrón

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::GodName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub pattern: String,
    pub handler: GodName,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct MessageRouter {
    routes: Arc<RwLock<HashMap<String, Route>>>,
    wildcard_routes: Arc<RwLock<Vec<Route>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            wildcard_routes: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn register_route(&self, pattern: &str, handler: GodName, priority: u32) {
        let route = Route {
            pattern: pattern.to_string(),
            handler,
            priority,
        };
        
        let mut routes = self.routes.write().await;
        routes.insert(pattern.to_string(), route);
    }
    
    pub async fn register_wildcard(&self, handler: GodName, priority: u32) {
        let route = Route {
            pattern: "*".to_string(),
            handler,
            priority,
        };
        
        let mut wildcards = self.wildcard_routes.write().await;
        wildcards.push(route);
        wildcards.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    pub async fn route(&self, to: &GodName) -> Option<GodName> {
        let routes = self.routes.read().await;
        let wildcards = self.wildcard_routes.read().await;
        
        // Try exact match first
        if let Some(route) = routes.get(&to.to_string()) {
            return Some(route.handler);
        }
        
        // Try wildcards
        for wildcard in wildcards.iter() {
            if wildcard.pattern == "*" {
                return Some(wildcard.handler);
            }
        }
        
        None
    }
    
    pub fn route_count(&self) -> usize {
        self.routes.blocking_read().len()
    }
}
