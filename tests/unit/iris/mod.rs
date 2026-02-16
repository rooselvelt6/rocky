// tests/unit/iris/mod.rs
// Tests unitarios para Iris - Service Mesh y Comunicaciones

use olympus::actors::iris::{Iris, Connection, ConnectionStatus};
use olympus::actors::{GodName, DivineDomain};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_iris_creation() {
        let iris = Iris::new().await;
        
        assert_eq!(iris.name(), GodName::Iris);
        assert_eq!(iris.domain(), DivineDomain::Communications);
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_creation() {
        let conn = Connection {
            connection_id: "conn-1".to_string(),
            protocol: "http/2".to_string(),
            status: ConnectionStatus::Active,
            last_activity: chrono::Utc::now(),
        };
        
        assert_eq!(conn.connection_id, "conn-1");
        assert_eq!(conn.protocol, "http/2");
    }
    
    #[tokio::test]
    async fn test_connection_status_variants() {
        assert_eq!(ConnectionStatus::Active, ConnectionStatus::Active);
        assert_eq!(ConnectionStatus::Idle, ConnectionStatus::Idle);
        assert_eq!(ConnectionStatus::Disconnected, ConnectionStatus::Disconnected);
    }
    
    #[tokio::test]
    async fn test_connection_serialization() {
        let conn = Connection {
            connection_id: "test-1".to_string(),
            protocol: "websocket".to_string(),
            status: ConnectionStatus::Active,
            last_activity: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&conn).unwrap();
        assert!(json.contains("test-1"));
        
        let decoded: Connection = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.connection_id, "test-1");
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_iris_initialization() {
        let mut iris = Iris::new().await;
        let result = iris.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_iris_shutdown() {
        let mut iris = Iris::new().await;
        iris.initialize().await.unwrap();
        
        let result = iris.shutdown().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_actor_state() {
        let iris = Iris::new().await;
        let state = iris.actor_state();
        
        assert_eq!(state.god, GodName::Iris);
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;
    use olympus::traits::message::ActorMessage;
    
    #[tokio::test]
    async fn test_handle_message() {
        let mut iris = Iris::new().await;
        
        let message = ActorMessage::ping();
        let response = iris.handle_message(message).await;
        
        assert!(response.is_ok());
    }
}
