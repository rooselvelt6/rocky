// src/actors/demeter/resources.rs
// OLYMPUS v15 - Gestión de recursos del sistema

use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};

/// Tipos de recursos monitoreados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// CPU
    Cpu,
    /// Memoria RAM
    Memory,
    /// Almacenamiento (disco)
    Storage,
    /// Red (network)
    Network,
}

impl ResourceType {
    /// Obtiene el nombre legible del recurso
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Cpu => "CPU",
            ResourceType::Memory => "Memoria",
            ResourceType::Storage => "Almacenamiento",
            ResourceType::Network => "Red",
        }
    }
}

/// Snapshot de recursos en un momento específico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Timestamp de la captura
    pub timestamp: DateTime<Utc>,
    /// Uso de CPU (0.0 - 1.0)
    pub cpu_usage: f64,
    /// Uso de memoria (0.0 - 1.0)
    pub memory_usage: f64,
    /// Uso de storage (0.0 - 1.0)
    pub storage_usage: f64,
    /// Uso de red (0.0 - 1.0, relativo al ancho de banda disponible)
    pub network_usage: f64,
    /// Métricas detalladas de CPU
    pub cpu_details: Option<CpuDetails>,
    /// Métricas detalladas de memoria
    pub memory_details: Option<MemoryDetails>,
    /// Métricas detalladas de storage
    pub storage_details: Option<StorageDetails>,
    /// Métricas detalladas de red
    pub network_details: Option<NetworkDetails>,
}

impl ResourceSnapshot {
    /// Captura una snapshot actual de recursos
    /// En producción, esto leería del sistema operativo
    pub async fn capture() -> Self {
        // Simulación de lectura de recursos
        // En una implementación real, usaría sysinfo o similar
        let now = Utc::now();
        
        // Generar valores pseudo-aleatorios pero realistas basados en el tiempo
        let time_factor = ((now.minute() as f64 * 60.0 + now.second() as f64) / 3600.0) * std::f64::consts::PI * 2.0;
        
        let cpu_usage = 0.3 + 0.2 * time_factor.sin() + random_offset(0.05);
        let memory_usage = 0.5 + 0.1 * (time_factor * 0.5).sin() + random_offset(0.03);
        let storage_usage = 0.6 + random_offset(0.02); // Storage cambia más lento
        let network_usage = 0.2 + 0.3 * (time_factor * 2.0).sin().abs() + random_offset(0.05);
        
        // Asegurar que estén en rango [0, 1]
        let cpu_usage = cpu_usage.clamp(0.0, 1.0);
        let memory_usage = memory_usage.clamp(0.0, 1.0);
        let storage_usage = storage_usage.clamp(0.0, 1.0);
        let network_usage = network_usage.clamp(0.0, 1.0);
        
        Self {
            timestamp: now,
            cpu_usage,
            memory_usage,
            storage_usage,
            network_usage,
            cpu_details: Some(CpuDetails::capture()),
            memory_details: Some(MemoryDetails::capture()),
            storage_details: Some(StorageDetails::capture()),
            network_details: Some(NetworkDetails::capture()),
        }
    }
    
    /// Obtiene el uso de un tipo específico de recurso
    pub fn get_usage(&self, resource_type: ResourceType) -> f64 {
        match resource_type {
            ResourceType::Cpu => self.cpu_usage,
            ResourceType::Memory => self.memory_usage,
            ResourceType::Storage => self.storage_usage,
            ResourceType::Network => self.network_usage,
        }
    }
    
    /// Verifica si algún recurso está por encima de un umbral
    pub fn is_above_threshold(&self, threshold: f64) -> Vec<ResourceType> {
        let mut resources = Vec::new();
        
        if self.cpu_usage >= threshold {
            resources.push(ResourceType::Cpu);
        }
        if self.memory_usage >= threshold {
            resources.push(ResourceType::Memory);
        }
        if self.storage_usage >= threshold {
            resources.push(ResourceType::Storage);
        }
        if self.network_usage >= threshold {
            resources.push(ResourceType::Network);
        }
        
        resources
    }
}

/// Detalles de CPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuDetails {
    /// Uso total del sistema (0.0 - 1.0)
    pub system_usage: f64,
    /// Uso de procesos de usuario (0.0 - 1.0)
    pub user_usage: f64,
    /// Uso de procesos del sistema (0.0 - 1.0)
    pub system_processes: f64,
    /// Tiempo idle (0.0 - 1.0)
    pub idle: f64,
    /// Número de cores
    pub cores: u32,
    /// Frecuencia promedio en MHz
    pub frequency_mhz: f64,
}

impl CpuDetails {
    /// Captura detalles de CPU
    pub fn capture() -> Self {
        // Simulación
        Self {
            system_usage: 0.3 + random_offset(0.05),
            user_usage: 0.4 + random_offset(0.05),
            system_processes: 0.1 + random_offset(0.02),
            idle: 0.2 + random_offset(0.05),
            cores: 8,
            frequency_mhz: 2400.0 + random_offset(200.0),
        }
    }
}

/// Detalles de memoria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetails {
    /// Memoria total en MB
    pub total_mb: u64,
    /// Memoria usada en MB
    pub used_mb: u64,
    /// Memoria libre en MB
    pub free_mb: u64,
    /// Memoria disponible (incluyendo caché) en MB
    pub available_mb: u64,
    /// Memoria en caché en MB
    pub cached_mb: u64,
    /// Memoria en buffers en MB
    pub buffers_mb: u64,
}

impl MemoryDetails {
    /// Captura detalles de memoria
    pub fn capture() -> Self {
        // Simulación: 16GB total
        let total_mb = 16384;
        let used_mb = (total_mb as f64 * (0.5 + random_offset(0.1))) as u64;
        let cached_mb = (total_mb as f64 * 0.15) as u64;
        let buffers_mb = (total_mb as f64 * 0.05) as u64;
        
        Self {
            total_mb,
            used_mb,
            free_mb: total_mb - used_mb,
            available_mb: total_mb - used_mb + cached_mb + buffers_mb,
            cached_mb,
            buffers_mb,
        }
    }
}

/// Detalles de storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDetails {
    /// Espacio total en GB
    pub total_gb: u64,
    /// Espacio usado en GB
    pub used_gb: u64,
    /// Espacio libre en GB
    pub free_gb: u64,
    /// Porcentaje de uso
    pub usage_percent: f64,
    /// Lecturas por segundo
    pub reads_per_sec: u64,
    /// Escrituras por segundo
    pub writes_per_sec: u64,
    /// Bytes leídos por segundo
    pub bytes_read_per_sec: u64,
    /// Bytes escritos por segundo
    pub bytes_written_per_sec: u64,
}

impl StorageDetails {
    /// Captura detalles de storage
    pub fn capture() -> Self {
        // Simulación: 500GB total
        let total_gb = 500;
        let used_gb = (total_gb as f64 * (0.6 + random_offset(0.05))) as u64;
        
        Self {
            total_gb,
            used_gb,
            free_gb: total_gb - used_gb,
            usage_percent: (used_gb as f64 / total_gb as f64) * 100.0,
            reads_per_sec: (50.0 + random_offset(20.0)) as u64,
            writes_per_sec: (30.0 + random_offset(10.0)) as u64,
            bytes_read_per_sec: (1024.0 * 1024.0 * (5.0 + random_offset(2.0))) as u64,
            bytes_written_per_sec: (1024.0 * 1024.0 * (3.0 + random_offset(1.0))) as u64,
        }
    }
}

/// Detalles de red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetails {
    /// Bytes recibidos por segundo
    pub bytes_recv_per_sec: u64,
    /// Bytes transmitidos por segundo
    pub bytes_sent_per_sec: u64,
    /// Paquetes recibidos por segundo
    pub packets_recv_per_sec: u64,
    /// Paquetes transmitidos por segundo
    pub packets_sent_per_sec: u64,
    /// Errores de entrada
    pub errors_in: u64,
    /// Errores de salida
    pub errors_out: u64,
    /// Latencia promedio en ms
    pub latency_ms: f64,
}

impl NetworkDetails {
    /// Captura detalles de red
    pub fn capture() -> Self {
        Self {
            bytes_recv_per_sec: (1024.0 * 1024.0 * (2.0 + random_offset(1.0))) as u64,
            bytes_sent_per_sec: (1024.0 * 1024.0 * (1.0 + random_offset(0.5))) as u64,
            packets_recv_per_sec: (1000.0 + random_offset(500.0)) as u64,
            packets_sent_per_sec: (800.0 + random_offset(400.0)) as u64,
            errors_in: 0,
            errors_out: 0,
            latency_ms: 10.0 + random_offset(5.0),
        }
    }
}

/// Métricas calculadas a partir de múltiples snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Uso promedio de CPU
    pub avg_cpu_usage: f64,
    /// Uso promedio de memoria
    pub avg_memory_usage: f64,
    /// Uso promedio de storage
    pub avg_storage_usage: f64,
    /// Uso promedio de red
    pub avg_network_usage: f64,
    /// Uso máximo de CPU
    pub max_cpu_usage: f64,
    /// Uso máximo de memoria
    pub max_memory_usage: f64,
    /// Uso máximo de storage
    pub max_storage_usage: f64,
    /// Uso máximo de red
    pub max_network_usage: f64,
    /// Uso mínimo de CPU
    pub min_cpu_usage: f64,
    /// Uso mínimo de memoria
    pub min_memory_usage: f64,
    /// Uso mínimo de storage
    pub min_storage_usage: f64,
    /// Uso mínimo de red
    pub min_network_usage: f64,
    /// Cantidad de snapshots analizados
    pub sample_count: usize,
    /// Timestamp de inicio del período
    pub period_start: Option<DateTime<Utc>>,
    /// Timestamp de fin del período
    pub period_end: Option<DateTime<Utc>>,
}

impl ResourceMetrics {
    /// Calcula métricas a partir de un vector de snapshots
    pub fn from_snapshots(snapshots: &[ResourceSnapshot]) -> Self {
        if snapshots.is_empty() {
            return Self::default();
        }
        
        let mut avg_cpu = 0.0;
        let mut avg_memory = 0.0;
        let mut avg_storage = 0.0;
        let mut avg_network = 0.0;
        
        let mut max_cpu: f64 = 0.0;
        let mut max_memory: f64 = 0.0;
        let mut max_storage: f64 = 0.0;
        let mut max_network: f64 = 0.0;
        
        let mut min_cpu: f64 = 1.0;
        let mut min_memory: f64 = 1.0;
        let mut min_storage: f64 = 1.0;
        let mut min_network: f64 = 1.0;
        
        for snapshot in snapshots {
            avg_cpu += snapshot.cpu_usage;
            avg_memory += snapshot.memory_usage;
            avg_storage += snapshot.storage_usage;
            avg_network += snapshot.network_usage;
            
            max_cpu = max_cpu.max(snapshot.cpu_usage);
            max_memory = max_memory.max(snapshot.memory_usage);
            max_storage = max_storage.max(snapshot.storage_usage);
            max_network = max_network.max(snapshot.network_usage);
            
            min_cpu = min_cpu.min(snapshot.cpu_usage);
            min_memory = min_memory.min(snapshot.memory_usage);
            min_storage = min_storage.min(snapshot.storage_usage);
            min_network = min_network.min(snapshot.network_usage);
        }
        
        let count = snapshots.len() as f64;
        
        Self {
            avg_cpu_usage: avg_cpu / count,
            avg_memory_usage: avg_memory / count,
            avg_storage_usage: avg_storage / count,
            avg_network_usage: avg_network / count,
            max_cpu_usage: max_cpu,
            max_memory_usage: max_memory,
            max_storage_usage: max_storage,
            max_network_usage: max_network,
            min_cpu_usage: if min_cpu == 1.0 { 0.0 } else { min_cpu },
            min_memory_usage: if min_memory == 1.0 { 0.0 } else { min_memory },
            min_storage_usage: if min_storage == 1.0 { 0.0 } else { min_storage },
            min_network_usage: if min_network == 1.0 { 0.0 } else { min_network },
            sample_count: snapshots.len(),
            period_start: snapshots.first().map(|s| s.timestamp),
            period_end: snapshots.last().map(|s| s.timestamp),
        }
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            avg_cpu_usage: 0.0,
            avg_memory_usage: 0.0,
            avg_storage_usage: 0.0,
            avg_network_usage: 0.0,
            max_cpu_usage: 0.0,
            max_memory_usage: 0.0,
            max_storage_usage: 0.0,
            max_network_usage: 0.0,
            min_cpu_usage: 0.0,
            min_memory_usage: 0.0,
            min_storage_usage: 0.0,
            min_network_usage: 0.0,
            sample_count: 0,
            period_start: None,
            period_end: None,
        }
    }
}

/// Genera un valor aleatorio pequeño para simulación
fn random_offset(magnitude: f64) -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let random = (nanos as f64 / u32::MAX as f64) * 2.0 - 1.0; // [-1, 1]
    random * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_snapshot_capture() {
        let snapshot = ResourceSnapshot::capture().await;
        
        assert!(snapshot.cpu_usage >= 0.0 && snapshot.cpu_usage <= 1.0);
        assert!(snapshot.memory_usage >= 0.0 && snapshot.memory_usage <= 1.0);
        assert!(snapshot.storage_usage >= 0.0 && snapshot.storage_usage <= 1.0);
        assert!(snapshot.network_usage >= 0.0 && snapshot.network_usage <= 1.0);
    }

    #[tokio::test]
    async fn test_resource_metrics_calculation() {
        let snapshots = vec![
            ResourceSnapshot {
                timestamp: Utc::now(),
                cpu_usage: 0.5,
                memory_usage: 0.6,
                storage_usage: 0.7,
                network_usage: 0.3,
                cpu_details: None,
                memory_details: None,
                storage_details: None,
                network_details: None,
            },
            ResourceSnapshot {
                timestamp: Utc::now(),
                cpu_usage: 0.7,
                memory_usage: 0.5,
                storage_usage: 0.8,
                network_usage: 0.4,
                cpu_details: None,
                memory_details: None,
                storage_details: None,
                network_details: None,
            },
        ];

        let metrics = ResourceMetrics::from_snapshots(&snapshots);
        
        assert_eq!(metrics.sample_count, 2);
        assert_eq!(metrics.avg_cpu_usage, 0.6);
        assert_eq!(metrics.avg_memory_usage, 0.55);
        assert_eq!(metrics.max_cpu_usage, 0.7);
        assert_eq!(metrics.min_cpu_usage, 0.5);
    }

    #[test]
    fn test_resource_type_display() {
        assert_eq!(ResourceType::Cpu.display_name(), "CPU");
        assert_eq!(ResourceType::Memory.display_name(), "Memoria");
    }
}
