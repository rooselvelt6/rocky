/// Demeter v12 - Diosa de la Agricultura y los Ciclos
/// Gestión de archivos y recursos del sistema

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResource {
    pub id: String,
    pub path: PathBuf,
    pub resource_type: ResourceType,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub access_count: u64,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ResourceType {
    PatientData,
    ClinicalAssessment,
    SystemConfiguration,
    SecurityKey,
    LogFile,
    BackupFile,
    TemporaryFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePolicy {
    pub resource_type: ResourceType,
    pub retention_days: u32,
    pub compression_enabled: bool,
    pub auto_cleanup: bool,
    pub max_versions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub operation_type: FileOperationType,
    pub resource_id: String,
    pub timestamp: DateTime<Utc>,
    pub performed_by: String,
    pub status: FileOperationStatus,
    pub details: Option<String>,
    pub affected_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileOperationType {
    Created,
    Read,
    Updated,
    Deleted,
    Archived,
    Restored,
    Accessed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileOperationStatus {
    Success,
    Failed,
    Partial,
    PermissionDenied,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct DemeterV12 {
    resources: HashMap<String, FileResource>,
    archive_policies: HashMap<ResourceType, ArchivePolicy>,
    storage_stats: StorageStatistics,
    base_directory: PathBuf,
    operation_history: Vec<FileOperation>,
    auto_cleanup_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageStatistics {
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub resource_type_counts: HashMap<String, u64>,
    pub oldest_file: Option<DateTime<Utc>>,
    pub newest_file: Option<DateTime<Utc>>,
    pub accessed_files: u64,
    pub created_files: u64,
    pub updated_files: u64,
    pub deleted_files: u64,
}

impl DemeterV12 {
    pub fn new() -> Self {
        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp/olympus"));
        
        Self {
            resources: HashMap::new(),
            archive_policies: Self::create_default_policies(),
            storage_stats: StorageStatistics {
                total_files: 0,
                total_size_bytes: 0,
                resource_type_counts: HashMap::new(),
                oldest_file: None,
                newest_file: None,
                accessed_files: 0,
                created_files: 0,
                updated_files: 0,
                deleted_files: 0,
            },
            base_directory: base_dir,
            operation_history: Vec::new(),
            auto_cleanup_enabled: true,
        }
    }

    pub fn with_base_directory(mut self, base_dir: PathBuf) -> Self {
        // Crear directorios necesarios
        if let Err(e) = std::fs::create_dir_all(&base_dir) {
            tracing::error!("🌾 Demeter: Error creando directorio base: {}", e);
        }
        
        self.base_directory = base_dir;
        self
    }

    pub async fn create_resource(&mut self, path: &str, resource_type: ResourceType, data: Option<&[u8]>) -> Result<String, String> {
        let resource_path = self.base_directory.join(path);
        let resource_id = uuid::Uuid::new_v4().to_string();
        
        // Crear directorio padre si no existe
        if let Some(parent) = resource_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("Error creando directorio: {}", e));
            }
        }

        let file_size = if let Some(file_data) = data {
            std::fs::write(&resource_path, file_data).map_err(|e| format!("Error escribiendo archivo: {}", e))?;
            file_data.len() as u64
        } else {
            std::fs::File::create(&resource_path)
                .map_err(|e| format!("Error creando archivo: {}", e))?;
            0
        };

        // Crear el recurso
        let resource = FileResource {
            id: resource_id.clone(),
            path: resource_path.clone(),
            resource_type: resource_type.clone(),
            size_bytes: file_size,
            created_at: Utc::now(),
            last_modified: Utc::now(),
            metadata: HashMap::new(),
            access_count: 0,
            version: 1,
        };

        // Registrar operación
        let operation = FileOperation {
            operation_type: FileOperationType::Created,
            resource_id: resource_id.clone(),
            timestamp: Utc::now(),
            performed_by: "demeter".to_string(),
            status: FileOperationStatus::Success,
            details: Some(format!("Recurso {} creado", path)),
            affected_bytes: file_size,
        };

        self.operation_history.push(operation);
        self.resources.insert(resource_id.clone(), resource);
        
        // Actualizar estadísticas
        self.update_storage_stats();
        
        tracing::info!("🌾 Demeter: Recurso {} creado - {} ({} bytes)", resource_id, path, file_size);
        Ok(resource_id)
    }

    pub async fn read_resource(&mut self, resource_id: &str) -> Result<Vec<u8>, String> {
        let path = if let Some(resource) = self.resources.get(resource_id) {
            resource.path.clone()
        } else {
            return Err(format!("Recurso {} no encontrado", resource_id));
        };
        
        // Actualizar contador de acceso
        self.increment_access_count(resource_id);
        
        // Actualizar timestamp de último acceso
        if let Some(resource) = self.resources.get(resource_id) {
            let mut updated_resource = resource.clone();
            updated_resource.last_modified = Utc::now();
            self.resources.insert(resource_id.to_string(), updated_resource);
        }
        
        // Leer archivo
        std::fs::read(&path)
            .map_err(|e| format!("Error leyendo archivo {}: {}", path.display(), e))
    }

    pub async fn update_resource(&mut self, resource_id: &str, data: Option<&[u8]>) -> Result<(), String> {
        let resource = if let Some(r) = self.resources.get(resource_id) {
            r.clone()
        } else {
            return Err(format!("Recurso {} no encontrado", resource_id));
        };
        
        let path = resource.path.clone();
        let file_size = if let Some(file_data) = data {
            std::fs::write(&path, file_data)
                .map_err(|e| format!("Error actualizando archivo: {}", e))?;
            file_data.len() as u64
        } else {
            // Truncar archivo existente
            let mut file = std::fs::File::options().write(true).open(&path)
                .map_err(|e| format!("Error abriendo archivo para escritura: {}", e))?;
            
            let mut existing_data = Vec::new();
            file.read_to_end(&mut existing_data)
                .map_err(|e| format!("Error leyendo contenido existente: {}", e))?;
            
            file.set_len(0);
            file.write_all(&existing_data)
                .map_err(|e| format!("Error escribiendo contenido truncado: {}", e))?;
            
            existing_data.len() as u64
        };

        // Actualizar recurso
        let mut updated_resource = resource.clone();
        updated_resource.last_modified = Utc::now();
        updated_resource.size_bytes = file_size;
        updated_resource.version += 1;
        updated_resource.metadata.insert("last_updated_by".to_string(), "demeter".to_string());
        
        self.resources.insert(resource_id.to_string(), updated_resource);
        
        // Registrar operación
        let operation = FileOperation {
            operation_type: FileOperationType::Updated,
            resource_id: resource_id.to_string(),
            timestamp: Utc::now(),
            performed_by: "demeter".to_string(),
            status: FileOperationStatus::Success,
            details: Some(format!("Recurso {} actualizado", path.display())),
            affected_bytes: file_size,
        };

        self.operation_history.push(operation);
        self.update_storage_stats();
        
        tracing::info!("🌾 Demeter: Recurso {} actualizado", resource_id);
        Ok(())
    }

    pub async fn delete_resource(&mut self, resource_id: &str) -> Result<(), String> {
        if let Some(resource) = self.resources.remove(resource_id) {
            // Registrar operación
            let operation = FileOperation {
                operation_type: FileOperationType::Deleted,
                resource_id: resource_id.to_string(),
                timestamp: Utc::now(),
                performed_by: "demeter".to_string(),
                status: FileOperationStatus::Success,
                details: Some(format!("Recurso {} eliminado: {}", resource_id, resource.path.display())),
                affected_bytes: resource.size_bytes,
            };

            self.operation_history.push(operation);
            
            // Mover a archivo si está configurado para archivar
            if let Some(policy) = self.archive_policies.get(&resource.resource_type) {
                if policy.auto_cleanup {
                    // Eliminar permanentemente
                    if let Err(e) = std::fs::remove_file(&resource.path) {
                        tracing::error!("🌾 Demeter: Error eliminando recurso {}: {}", resource_id, e);
                        return Err(format!("Error eliminando recurso: {}", e));
                    }
                    
                    tracing::info!("🌾 Demeter: Recurso {} eliminado permanentemente", resource_id);
                } else {
                    if let Err(e) = self.archive_resource(&resource, policy).await {
                        tracing::error!("🌾 Demeter: Error archivando recurso {}: {}", resource_id, e);
                        return Err(e);
                    }
                    
                    tracing::info!("🌾 Demeter: Recurso {} archivado", resource_id);
                }
            } else {
                // Eliminar permanentemente
                if let Err(e) = std::fs::remove_file(&resource.path) {
                    tracing::error!("🌾 Demeter: Error eliminando recurso {}: {}", resource_id, e);
                    return Err(format!("Error eliminando recurso: {}", e));
                }
                
                tracing::info!("🌾 Demeter: Recurso {} eliminado permanentemente", resource_id);
            }
            
            self.update_storage_stats();
            Ok(())
        } else {
            Err(format!("Recurso {} no encontrado", resource_id))
        }
    }

    async fn archive_resource(&self, resource: &FileResource, _policy: &ArchivePolicy) -> Result<(), String> {
        let archive_dir = self.base_directory.join("archive");
        
        // Crear directorio de archivos si no existe
        if let Err(e) = std::fs::create_dir_all(&archive_dir) {
            return Err(format!("Error creando directorio de archivos: {}", e));
        }
        
        // Crear nombre de archivo archivado
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let file_name = resource.path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        let archive_name = format!("{}_v{}_{}", 
                                     file_name, 
                                     resource.version,
                                     timestamp);
        let archive_path = archive_dir.join(format!("{:?}_{}", resource.resource_type, archive_name));
        
        // Mover archivo a archivo
        std::fs::rename(&resource.path, &archive_path)
            .map_err(|e| format!("Error archivando recurso {}: {}", resource.path.display(), e))?;
        
        tracing::info!("🌾 Demeter: Recurso {} archivado como {}", resource.id, archive_name);
        Ok(())
    }

    pub fn get_resource(&self, resource_id: &str) -> Option<&FileResource> {
        self.resources.get(resource_id)
    }

    pub fn get_resources_by_type(&self, resource_type: &ResourceType) -> Vec<&FileResource> {
        self.resources
            .values()
            .filter(|resource| resource.resource_type == *resource_type)
            .collect()
    }

    pub fn search_resources(&self, query: &str, limit: Option<usize>) -> Vec<&FileResource> {
        let mut matches = Vec::new();
        
        for resource in self.resources.values() {
            let search_in = [
                resource.id.to_lowercase(),
                resource.path.to_string_lossy().to_lowercase(),
                format!("{:?}", resource.resource_type).to_lowercase(),
            ];
            
            let query_lower = query.to_lowercase();
            if search_in.iter().any(|s| s.contains(&query_lower)) {
                matches.push(resource);
            }
        }
        
        // Ordenar por relevancia (más reciente primero)
        matches.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        
        if let Some(limit) = limit {
            matches.truncate(limit);
        }
        
        matches
    }

    pub fn set_archive_policy(&mut self, resource_type: ResourceType, policy: ArchivePolicy) {
        let resource_type_name = format!("{:?}", resource_type);
        self.archive_policies.insert(resource_type, policy);
        tracing::info!("🌾 Demeter: Política de archivado actualizada para {:?}", resource_type_name);
    }

    pub async fn cleanup_old_files(&mut self) -> Result<u64, String> {
        let cutoff_time = Utc::now();
        let mut deleted_count = 0u64;
        let resources_to_check: Vec<FileResource> = self.resources.values().cloned().collect();
        
        for resource in resources_to_check {
            let policy = self.archive_policies.get(&resource.resource_type);
            
            if let Some(policy) = policy {
                let retention_cutoff = cutoff_time - Duration::days(policy.retention_days as i64);
                
                if resource.last_modified < retention_cutoff {
                    // Eliminar versiones antiguas
                    let versions_to_keep = policy.max_versions;
                    let resource_base_name = resource.path.file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    // Encontrar todos los archivos con el mismo nombre base
                    let mut archive_files = Vec::new();
                    if let Some(parent) = resource.path.parent() {
                        if let Ok(entries) = std::fs::read_dir(parent) {
                            for entry in entries.flatten() {
                                if let Some(file_name) = entry.file_name().to_str() {
                                    if file_name.starts_with(&resource_base_name) {
                                        archive_files.push(file_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Ordenar por nombre para identificar versiones
                    archive_files.sort();
                    
                    // Eliminar archivos viejos
                    if let Some(parent) = resource.path.parent() {
                        for file_to_delete in archive_files.iter().skip(versions_to_keep as usize) {
                            let file_path = parent.join(file_to_delete);
                            if std::fs::remove_file(&file_path).is_ok() {
                                deleted_count += 1;
                            }
                        }
                    }
                    
                    // Eliminar el recurso actual si está más viejo que las versiones guardadas
                    if archive_files.is_empty() || resource.last_modified < retention_cutoff {
                        if let Err(e) = std::fs::remove_file(&resource.path) {
                            tracing::error!("🌾 Demeter: Error eliminando archivo antiguo: {}", e);
                        } else {
                            deleted_count += 1;
                            self.resources.remove(&resource.id);
                        }
                    }
                }
            }
        }
        
        if deleted_count > 0 {
            tracing::info!("🌾 Demeter: {} archivos antiguos eliminados", deleted_count);
        }
        
        self.update_storage_stats();
        Ok(deleted_count)
    }

    pub fn enable_auto_cleanup(&mut self) {
        self.auto_cleanup_enabled = true;
        tracing::info!("🌾 Demeter: Auto-limpieza habilitada");
    }

    pub fn disable_auto_cleanup(&mut self) {
        self.auto_cleanup_enabled = false;
        tracing::info!("🌾 Demeter: Auto-limpieza deshabilitada");
    }

    fn increment_access_count(&mut self, resource_id: &str) {
        if let Some(resource) = self.resources.get_mut(resource_id) {
            resource.access_count += 1;
            resource.metadata.insert("last_accessed".to_string(), Utc::now().to_rfc3339());
        }
    }

    fn update_storage_stats(&mut self) {
        let mut stats = self.storage_stats.clone();
        
        stats.total_files = self.resources.len() as u64;
        stats.total_size_bytes = self.resources.values().map(|r| r.size_bytes).sum();
        stats.resource_type_counts.clear();
        
        let mut oldest_date: Option<DateTime<Utc>> = None;
        let mut newest_date: Option<DateTime<Utc>> = None;
        let mut accessed_count: u64 = 0;
        let mut created_count: u64 = 0;
        let mut updated_count: u64 = 0;
        let mut deleted_count: u64 = 0;
        
        for resource in self.resources.values() {
            let type_key = format!("{:?}", resource.resource_type);
            let current_count = stats.resource_type_counts.get(&type_key).copied().unwrap_or(0);
            stats.resource_type_counts.insert(type_key, current_count + 1);
            
            oldest_date = oldest_date.map(|d| d.min(resource.created_at)).or(Some(resource.created_at));
            newest_date = newest_date.map(|d| d.max(resource.created_at)).or(Some(resource.created_at));
            
            accessed_count += resource.access_count;
            created_count += 1;
            updated_count += resource.version.saturating_sub(1) as u64;
            
            // Contar operaciones del historial
            for operation in &self.operation_history {
                match operation.operation_type {
                    FileOperationType::Deleted => deleted_count += 1,
                    FileOperationType::Accessed => accessed_count += 1,
                    FileOperationType::Created => created_count += 1,
                    FileOperationType::Updated => updated_count += 1,
                    FileOperationType::Archived => {},
                    _ => {}
                }
            }
        }
        
        stats.oldest_file = oldest_date;
        stats.newest_file = newest_date;
        stats.accessed_files = accessed_count;
        stats.created_files = created_count;
        stats.updated_files = updated_count;
        stats.deleted_files = deleted_count;
        
        self.storage_stats = stats;
    }

    fn create_default_policies() -> HashMap<ResourceType, ArchivePolicy> {
        let mut policies = HashMap::new();
        
        // Política de archivos de pacientes (7 años)
        policies.insert(ResourceType::PatientData, ArchivePolicy {
            resource_type: ResourceType::PatientData,
            retention_days: 2555, // 7 años
            compression_enabled: true,
            auto_cleanup: true,
            max_versions: 10,
        });
        
        // Política de evaluaciones clínicas (5 años)
        policies.insert(ResourceType::ClinicalAssessment, ArchivePolicy {
            resource_type: ResourceType::ClinicalAssessment,
            retention_days: 1825, // 5 años
            compression_enabled: true,
            auto_cleanup: true,
            max_versions: 20,
        });
        
        // Política de configuración (2 años)
        policies.insert(ResourceType::SystemConfiguration, ArchivePolicy {
            resource_type: ResourceType::SystemConfiguration,
            retention_days: 730, // 2 años
            compression_enabled: true,
            auto_cleanup: true,
            max_versions: 5,
        });
        
        // Política de claves de seguridad (permanentes)
        policies.insert(ResourceType::SecurityKey, ArchivePolicy {
            resource_type: ResourceType::SecurityKey,
            retention_days: 3650, // 10 años
            compression_enabled: true,
            auto_cleanup: false, // Nunca limpiar claves automáticamente
            max_versions: 1, // Solo una versión
        });
        
        // Política de logs (90 días)
        policies.insert(ResourceType::LogFile, ArchivePolicy {
            resource_type: ResourceType::LogFile,
            retention_days: 90,
            compression_enabled: true,
            auto_cleanup: true,
            max_versions: 50,
        });
        
        // Política de backups (5 años)
        policies.insert(ResourceType::BackupFile, ArchivePolicy {
            resource_type: ResourceType::BackupFile,
            retention_days: 1825, // 5 años
            compression_enabled: true,
            auto_cleanup: true,
            max_versions: 25,
        });
        
        // Archivos temporarios (30 días)
        policies.insert(ResourceType::TemporaryFile, ArchivePolicy {
            resource_type: ResourceType::TemporaryFile,
            retention_days: 30,
            compression_enabled: false,
            auto_cleanup: true,
            max_versions: 5,
        });
        
        policies
    }

    pub fn get_storage_statistics(&self) -> &StorageStatistics {
        &self.storage_stats
    }

    pub fn get_operation_history(&self, limit: Option<usize>) -> Vec<FileOperation> {
        let mut history = self.operation_history.clone();
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            history.truncate(limit);
        }
        
        history
    }

    pub fn cleanup_temporary_files(&mut self) -> Result<u64, String> {
        let mut deleted_count = 0u64;
        
        let temp_dir = self.base_directory.join("temp");
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Err(e) = std::fs::remove_file(&file_path) {
                        tracing::warn!("🌾 Demeter: Error eliminando archivo temporal: {}", e);
                    } else {
                        deleted_count += 1;
                    }
                }
            }
        }
        
        if deleted_count > 0 {
            tracing::info!("🌾 Demeter: {} archivos temporales eliminados", deleted_count);
        }
        
        Ok(deleted_count)
    }

    pub fn get_file_path(&self, resource_id: &str) -> Option<String> {
        self.resources
            .get(resource_id)
            .map(|r| r.path.to_string_lossy().to_string())
    }
}

impl Default for DemeterV12 {
    fn default() -> Self {
        Self::new()
    }
}