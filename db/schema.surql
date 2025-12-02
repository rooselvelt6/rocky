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
-- Schema actualizado para coincidir con el modelo Rust Patient
DEFINE TABLE patients SCHEMAFULL;
DEFINE FIELD first_name ON patients TYPE string;
DEFINE FIELD last_name ON patients TYPE string;
DEFINE FIELD date_of_birth ON patients TYPE string;  -- ISO 8601 date YYYY-MM-DD
DEFINE FIELD gender ON patients TYPE string;
DEFINE FIELD admission_date ON patients TYPE string; -- ISO 8601 datetime (RFC3339)
DEFINE FIELD admission_diagnosis ON patients TYPE string;
DEFINE FIELD created_at ON patients TYPE string;     -- ISO 8601 datetime (RFC3339)

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
-- TABLA: APACHE II Assessments
-- ========================================
DEFINE TABLE apache_assessments SCHEMAFULL;
DEFINE FIELD patient ON apache_assessments TYPE option<record(patients)>;
DEFINE FIELD temperature ON apache_assessments TYPE float;
DEFINE FIELD mean_arterial_pressure ON apache_assessments TYPE int;
DEFINE FIELD heart_rate ON apache_assessments TYPE int;
DEFINE FIELD respiratory_rate ON apache_assessments TYPE int;
DEFINE FIELD oxygenation_type ON apache_assessments TYPE string;
DEFINE FIELD oxygenation_value ON apache_assessments TYPE int;
DEFINE FIELD arterial_ph ON apache_assessments TYPE float;
DEFINE FIELD serum_sodium ON apache_assessments TYPE int;
DEFINE FIELD serum_potassium ON apache_assessments TYPE float;
DEFINE FIELD serum_creatinine ON apache_assessments TYPE float;
DEFINE FIELD hematocrit ON apache_assessments TYPE float;
DEFINE FIELD white_blood_count ON apache_assessments TYPE float;
DEFINE FIELD glasgow_coma_score ON apache_assessments TYPE int ASSERT $value >= 3 AND $value <= 15;
DEFINE FIELD age ON apache_assessments TYPE int;
DEFINE FIELD chronic_health ON apache_assessments TYPE string;
DEFINE FIELD score ON apache_assessments TYPE int;
DEFINE FIELD predicted_mortality ON apache_assessments TYPE float;
DEFINE FIELD severity ON apache_assessments TYPE string;
DEFINE FIELD recommendation ON apache_assessments TYPE string;
DEFINE FIELD assessed_by ON apache_assessments TYPE string;
DEFINE FIELD assessed_at ON apache_assessments TYPE string;

-- Índice para búsquedas por paciente
DEFINE INDEX idx_apache_patient ON apache_assessments FIELDS patient;
DEFINE INDEX idx_apache_date ON apache_assessments FIELDS assessed_at;

-- ========================================
-- TABLA: SOFA Assessments
-- ========================================
DEFINE TABLE sofa_assessments SCHEMAFULL;
DEFINE FIELD patient ON sofa_assessments TYPE option<record(patients)>;
DEFINE FIELD pao2_fio2 ON sofa_assessments TYPE int;
DEFINE FIELD platelets ON sofa_assessments TYPE int;
DEFINE FIELD bilirubin ON sofa_assessments TYPE float;
DEFINE FIELD cardiovascular ON sofa_assessments TYPE string;
DEFINE FIELD glasgow ON sofa_assessments TYPE int ASSERT $value >= 3 AND $value <= 15;
DEFINE FIELD renal ON sofa_assessments TYPE string;
DEFINE FIELD score ON sofa_assessments TYPE int;
DEFINE FIELD severity ON sofa_assessments TYPE string;
DEFINE FIELD recommendation ON sofa_assessments TYPE string;
DEFINE FIELD assessed_by ON sofa_assessments TYPE string;
DEFINE FIELD assessed_at ON sofa_assessments TYPE string;

-- Índice para búsquedas por paciente
DEFINE INDEX idx_sofa_patient ON sofa_assessments FIELDS patient;
DEFINE INDEX idx_sofa_date ON sofa_assessments FIELDS assessed_at;

-- ========================================
-- TABLA: SAPS II Assessments
-- ========================================
DEFINE TABLE saps_assessments SCHEMAFULL;
DEFINE FIELD patient ON saps_assessments TYPE option<record(patients)>;
DEFINE FIELD age ON saps_assessments TYPE int;
DEFINE FIELD heart_rate ON saps_assessments TYPE int;
DEFINE FIELD systolic_bp ON saps_assessments TYPE int;
DEFINE FIELD temperature ON saps_assessments TYPE float;
DEFINE FIELD pao2_fio2 ON saps_assessments TYPE option<int>;
DEFINE FIELD urinary_output ON saps_assessments TYPE float;
DEFINE FIELD serum_urea ON saps_assessments TYPE float;
DEFINE FIELD white_blood_count ON saps_assessments TYPE float;
DEFINE FIELD serum_potassium ON saps_assessments TYPE float;
DEFINE FIELD serum_sodium ON saps_assessments TYPE int;
DEFINE FIELD serum_bicarbonate ON saps_assessments TYPE float;
DEFINE FIELD bilirubin ON saps_assessments TYPE float;
DEFINE FIELD glasgow ON saps_assessments TYPE int ASSERT $value >= 3 AND $value <= 15;
DEFINE FIELD chronic_disease ON saps_assessments TYPE string;
DEFINE FIELD admission_type ON saps_assessments TYPE string;
DEFINE FIELD score ON saps_assessments TYPE int;
DEFINE FIELD predicted_mortality ON saps_assessments TYPE float;
DEFINE FIELD severity ON saps_assessments TYPE string;
DEFINE FIELD recommendation ON saps_assessments TYPE string;
DEFINE FIELD assessed_by ON saps_assessments TYPE string;
DEFINE FIELD assessed_at ON saps_assessments TYPE string;

-- Índice para búsquedas por paciente
DEFINE INDEX idx_saps_patient ON saps_assessments FIELDS patient;
DEFINE INDEX idx_saps_date ON saps_assessments FIELDS assessed_at;

-- ========================================
-- DATOS DE EJEMPLO (Opcional)
-- ========================================
-- Crear un paciente de prueba
CREATE patients:demo_patient SET
    first_name = "Juan",
    last_name = "Pérez",
    date_of_birth = "1979-01-15",
    gender = "Male",
    admission_date = <string>time::now(),
    admission_diagnosis = "Traumatismo craneoencefálico severo",
    created_at = <string>time::now();

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
