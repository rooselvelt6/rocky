-- ========================================
-- UCI Scales Database Schema
-- SurrealDB Initialization Script
-- ========================================

-- Definir namespace y database
USE NS hospital;
USE DB uci;

-- ========================================
-- TABLA: Patients (Pacientes)
-- ========================================
DEFINE TABLE patients SCHEMAFULL;
DEFINE FIELD name ON patients TYPE string;
DEFINE FIELD age ON patients TYPE int;
DEFINE FIELD gender ON patients TYPE string;
DEFINE FIELD medical_id ON patients TYPE string;
DEFINE FIELD admission_date ON patients TYPE datetime DEFAULT time::now();
DEFINE FIELD status ON patients TYPE string DEFAULT "active";
DEFINE FIELD created_at ON patients TYPE datetime DEFAULT time::now();

-- Índice único para medical_id
DEFINE INDEX idx_medical_id ON patients FIELDS medical_id UNIQUE;

-- ========================================
-- TABLA: Glasgow Assessments
-- ========================================
DEFINE TABLE glasgow_assessments SCHEMAFULL;
DEFINE FIELD patient ON glasgow_assessments TYPE record(patients);
DEFINE FIELD eye_response ON glasgow_assessments TYPE int ASSERT $value >= 1 AND $value <= 4;
DEFINE FIELD verbal_response ON glasgow_assessments TYPE int ASSERT $value >= 1 AND $value <= 5;
DEFINE FIELD motor_response ON glasgow_assessments TYPE int ASSERT $value >= 1 AND $value <= 6;
DEFINE FIELD score ON glasgow_assessments TYPE int;
DEFINE FIELD diagnosis ON glasgow_assessments TYPE string;
DEFINE FIELD recommendation ON glasgow_assessments TYPE string;
DEFINE FIELD assessed_by ON glasgow_assessments TYPE string;
DEFINE FIELD assessed_at ON glasgow_assessments TYPE datetime DEFAULT time::now();

-- Índice para búsquedas por paciente
DEFINE INDEX idx_patient ON glasgow_assessments FIELDS patient;
DEFINE INDEX idx_date ON glasgow_assessments FIELDS assessed_at;

-- ========================================
-- DATOS DE EJEMPLO (Opcional)
-- ========================================
-- Crear un paciente de prueba
CREATE patients:demo_patient SET
    name = "Juan Pérez",
    age = 45,
    gender = "M",
    medical_id = "HC-2024-001",
    admission_date = time::now(),
    status = "active";

-- Crear una evaluación de Glasgow de ejemplo
CREATE glasgow_assessments SET
    patient = patients:demo_patient,
    eye_response = 4,
    verbal_response = 5,
    motor_response = 6,
    score = 15,
    diagnosis = "Normal consciousness",
    recommendation = "Continue monitoring",
    assessed_by = "Dr. Sistema",
    assessed_at = time::now();
