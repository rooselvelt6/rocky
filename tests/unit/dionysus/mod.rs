// tests/unit/dionysus/mod.rs
// Tests unitarios para Dionysus - Análisis de Datos

use olympus::actors::dionysus::{Dionysus, DionysusConfig, AnalyticsEngine, Visualization};
use olympus::actors::dionysus::types::{DataSet, ChartType, Metric, Trend};
use olympus::actors::{GodName, DivineDomain};
use olympus::traits::actor_trait::{OlympianActor, ActorMessage};

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_dionysus_config() {
        let config = DionysusConfig::default();
        assert!(config.real_time_analytics);
        assert_eq!(config.retention_days, 90);
        assert!(config.visualization_enabled);
    }
}

#[cfg(test)]
mod analytics_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_statistical_analysis() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        
        let stats = dionysus.calculate_statistics(&data).await.unwrap();
        
        assert_eq!(stats.mean, 30.0);
        assert!(stats.median >= 20.0 && stats.median <= 40.0);
        assert!(stats.std_dev > 0.0);
    }
    
    #[tokio::test]
    async fn test_trend_analysis() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let time_series = vec![
            ("2024-01-01", 100.0),
            ("2024-01-02", 105.0),
            ("2024-01-03", 110.0),
            ("2024-01-04", 115.0),
            ("2024-01-05", 120.0),
        ];
        
        let trend = dionysus.analyze_trend(&time_series).await.unwrap();
        
        assert_eq!(trend.direction, TrendDirection::Increasing);
        assert!(trend.slope > 0.0);
    }
    
    #[tokio::test]
    async fn test_correlation_analysis() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let series_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let series_b = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // Perfectamente correlacionado
        
        let correlation = dionysus.calculate_correlation(&series_a, &series_b).await.unwrap();
        
        assert!(correlation > 0.99); // Casi 1.0
    }
    
    #[tokio::test]
    async fn test_anomaly_detection() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = vec![
            10.0, 11.0, 10.5, 11.5, 10.8,
            100.0, // Anomalía!
            11.0, 10.9, 11.2, 10.7
        ];
        
        let anomalies = dionysus.detect_anomalies(&data).await.unwrap();
        
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].index, 5);
    }
}

#[cfg(test)]
mod visualization_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_line_chart() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = DataSet::new()
            .with_labels(vec!["Jan", "Feb", "Mar", "Apr"])
            .with_series("Revenue", vec![100.0, 120.0, 130.0, 140.0]);
        
        let chart = dionysus.create_chart(data, ChartType::Line).await.unwrap();
        
        assert!(!chart.data.is_empty());
        assert_eq!(chart.chart_type, ChartType::Line);
    }
    
    #[tokio::test]
    async fn test_create_bar_chart() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = DataSet::new()
            .with_labels(vec!["A", "B", "C"])
            .with_series("Sales", vec![50.0, 75.0, 25.0]);
        
        let chart = dionysus.create_chart(data, ChartType::Bar).await.unwrap();
        
        assert_eq!(chart.chart_type, ChartType::Bar);
    }
    
    #[tokio::test]
    async fn test_create_pie_chart() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = DataSet::new()
            .with_labels(vec!["Product A", "Product B", "Product C"])
            .with_series("Market Share", vec![40.0, 35.0, 25.0]);
        
        let chart = dionysus.create_chart(data, ChartType::Pie).await.unwrap();
        
        assert_eq!(chart.chart_type, ChartType::Pie);
    }
    
    #[tokio::test]
    async fn test_export_chart() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data = DataSet::new()
            .with_labels(vec!["Q1", "Q2", "Q3", "Q4"])
            .with_series("Growth", vec![10.0, 15.0, 20.0, 25.0]);
        
        let chart = dionysus.create_chart(data, ChartType::Line).await.unwrap();
        
        let svg = dionysus.export_chart(&chart, ExportFormat::SVG).await.unwrap();
        
        assert!(svg.contains("<svg") || svg.contains("svg"));
    }
}

#[cfg(test)]
mod metrics_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculate_derived_metrics() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let base_metrics = vec![
            Metric::new("revenue", 1000.0),
            Metric::new("costs", 600.0),
        ];
        
        let derived = dionysus.calculate_derived_metrics(&base_metrics).await;
        
        // Debe calcular profit = revenue - costs
        assert!(derived.iter().any(|m| m.name == "profit" && m.value == 400.0));
    }
    
    #[tokio::test]
    async fn test_aggregation_by_time() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let data_points = vec![
            ("2024-01-01 10:00", 10.0),
            ("2024-01-01 10:05", 15.0),
            ("2024-01-01 10:10", 20.0),
        ];
        
        let hourly = dionysus.aggregate_by_time(
            &data_points,
            AggregationPeriod::Hourly,
            AggregationFunction::Mean
        ).await.unwrap();
        
        assert_eq!(hourly.len(), 1);
        assert_eq!(hourly[0].1, 15.0); // Promedio
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dionysus_creation() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        assert_eq!(dionysus.name(), GodName::Dionysus);
        assert_eq!(dionysus.domain(), DivineDomain::DataAnalysis);
    }
    
    #[tokio::test]
    async fn test_dionysus_health_check() {
        let dionysus = Dionysus::new().await.expect("Failed to create Dionysus");
        
        let health = dionysus.health_check().await;
        assert!(health.is_healthy());
    }
}
