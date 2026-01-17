use leptos::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Es,
}

impl Default for Language {
    fn default() -> Self {
        Language::Es // Default to Spanish as requested implicitly by "cambiar el idioma... a español"
    }
}

pub fn use_i18n() -> Signal<Language> {
    use_context::<Signal<Language>>().expect("I18n context should be provided")
}

pub fn t(lang: Language, key: &str) -> String {
    match (lang, key) {
        // Navigation & Common
        (Language::En, "home") => "Home".to_string(),
        (Language::Es, "home") => "Inicio".to_string(),
        (Language::En, "register_patient") => "Register Patient".to_string(),
        (Language::Es, "register_patient") => "Registrar Paciente".to_string(),
        (Language::En, "glasgow_scale") => "Glasgow Scale".to_string(),
        (Language::Es, "glasgow_scale") => "Escala Glasgow".to_string(),
        (Language::En, "apache_ii") => "APACHE II".to_string(),
        (Language::Es, "apache_ii") => "APACHE II".to_string(),
        (Language::En, "sofa_score") => "SOFA Score".to_string(),
        (Language::Es, "sofa_score") => "Escala SOFA".to_string(),
        (Language::En, "welcome_title") => "Welcome to UCI System".to_string(),
        (Language::Es, "welcome_title") => "Bienvenido al Sistema UCI".to_string(),
        (Language::En, "welcome_subtitle") => {
            "Comprehensive ICU Patient Assessment Tools".to_string()
        }
        (Language::Es, "welcome_subtitle") => {
            "Herramientas Integrales de Evaluación en UCI".to_string()
        }
        (Language::En, "made_with_love") => "Made with ❤️ for improving ICU care".to_string(),
        (Language::Es, "made_with_love") => {
            "Hecho con ❤️ para mejorar el cuidado en UCI".to_string()
        }
        (Language::En, "clinical_dashboard") => "Clinical Dashboard".to_string(),
        (Language::Es, "clinical_dashboard") => "Panel Clínico".to_string(),
        (Language::En, "dashboard_subtitle") => {
            "Access clinical tools and patient records".to_string()
        }
        (Language::Es, "dashboard_subtitle") => {
            "Acceso a herramientas clínicas y registros de pacientes".to_string()
        }
        (Language::En, "icu_info_title") => "About our ICU".to_string(),
        (Language::Es, "icu_info_title") => "Sobre nuestra UCI".to_string(),
        (Language::En, "icu_info_desc") => {
            "The Intensive Care Unit provides specialized care for patients with life-threatening illnesses or injuries.".to_string()
        }
        (Language::Es, "icu_info_desc") => {
            "La Unidad de Cuidados Intensivos brinda atención especializada para pacientes con enfermedades o lesiones potencialmente mortales.".to_string()
        }
        (Language::En, "medical_disclaimer_title") => "Medical Disclaimer".to_string(),
        (Language::Es, "medical_disclaimer_title") => "Aviso Médico".to_string(),
        (Language::En, "medical_disclaimer_text") => "This is a clinical decision support tool. It does NOT replace professional medical judgment. All scores and calculations must be verified by qualified medical personnel before clinical application.".to_string(),
        (Language::Es, "medical_disclaimer_text") => "Esta es una herramienta de apoyo a la decisión clínica. NO reemplaza el juicio médico profesional. Todas las puntuaciones y cálculos deben ser verificados por personal médico calificado antes de su aplicación clínica.".to_string(),
        (Language::En, "footer_license") => "Licensed under GNU GPL v3.0".to_string(),
        (Language::Es, "footer_license") => "Licenciado bajo GNU GPL v3.0".to_string(),
        (Language::En, "login_required_desc") => {
            "Access to clinical scales and patient data is restricted to authorized medical staff.".to_string()
        }
        (Language::Es, "login_required_desc") => {
            "El acceso a las escalas clínicas y datos de pacientes está restringido al personal médico autorizado.".to_string()
        }

        // Actions
        (Language::En, "calculate") => "Calculate".to_string(),
        (Language::Es, "calculate") => "Calcular".to_string(),
        (Language::En, "calculating") => "Calculating...".to_string(),
        (Language::Es, "calculating") => "Calculando...".to_string(),
        (Language::En, "complete_form_hint") => {
            "Complete the form and click Calculate to see results".to_string()
        }
        (Language::Es, "complete_form_hint") => {
            "Complete el formulario y haga clic en Calcular para ver resultados".to_string()
        }

        // Results
        (Language::En, "score") => "Score".to_string(),
        (Language::Es, "score") => "Puntaje".to_string(),
        (Language::En, "severity") => "Severity".to_string(),
        (Language::Es, "severity") => "Severidad".to_string(),
        (Language::En, "mortality") => "Predicted Mortality".to_string(),
        (Language::Es, "mortality") => "Mortalidad Predicha".to_string(),
        (Language::En, "recommendation") => "Recommendation".to_string(),
        (Language::Es, "recommendation") => "Recomendación".to_string(),

        // APACHE II Fields
        (Language::En, "apache_title") => "APACHE II Score".to_string(),
        (Language::Es, "apache_title") => "Puntaje APACHE II".to_string(),
        (Language::En, "apache_subtitle") => {
            "Acute Physiology and Chronic Health Evaluation II".to_string()
        }
        (Language::Es, "apache_subtitle") => {
            "Evaluación de Fisiología Aguda y Salud Crónica II".to_string()
        }
        (Language::En, "calculate_apache") => "Calculate APACHE II Score".to_string(),
        (Language::Es, "calculate_apache") => "Calcular Puntaje APACHE II".to_string(),

        (Language::En, "age") => "Age".to_string(),
        (Language::Es, "age") => "Edad".to_string(),
        (Language::En, "temperature") => "Temperature (°C)".to_string(),
        (Language::Es, "temperature") => "Temperatura (°C)".to_string(),
        (Language::En, "map") => "Mean Arterial Pressure (mmHg)".to_string(),
        (Language::Es, "map") => "Presión Arterial Media (mmHg)".to_string(),
        (Language::En, "heart_rate") => "Heart Rate (bpm)".to_string(),
        (Language::Es, "heart_rate") => "Frecuencia Cardíaca (lpm)".to_string(),
        (Language::En, "respiratory_rate") => "Respiratory Rate (bpm)".to_string(),
        (Language::Es, "respiratory_rate") => "Frecuencia Respiratoria (rpm)".to_string(),
        (Language::En, "a_a_gradient") => "A-a Gradient (if FiO2 ≥ 0.5)".to_string(),
        (Language::Es, "a_a_gradient") => "Gradiente A-a (si FiO2 ≥ 0.5)".to_string(),
        (Language::En, "pao2") => "PaO2 (if FiO2 < 0.5)".to_string(),
        (Language::Es, "pao2") => "PaO2 (si FiO2 < 0.5)".to_string(),
        (Language::En, "arterial_ph") => "Arterial pH".to_string(),
        (Language::Es, "arterial_ph") => "pH Arterial".to_string(),
        (Language::En, "sodium") => "Serum Sodium (mMol/L)".to_string(),
        (Language::Es, "sodium") => "Sodio Sérico (mMol/L)".to_string(),
        (Language::En, "potassium") => "Serum Potassium (mMol/L)".to_string(),
        (Language::Es, "potassium") => "Potasio Sérico (mMol/L)".to_string(),
        (Language::En, "creatinine") => "Serum Creatinine (mg/dL)".to_string(),
        (Language::Es, "creatinine") => "Creatinina Sérica (mg/dL)".to_string(),
        (Language::En, "hematocrit") => "Hematocrit (%)".to_string(),
        (Language::Es, "hematocrit") => "Hematocrito (%)".to_string(),
        (Language::En, "wbc") => "White Blood Count (x10³/µL)".to_string(),
        (Language::Es, "wbc") => "Leucocitos (x10³/µL)".to_string(),
        (Language::En, "gcs") => "Glasgow Coma Score (15 - Actual)".to_string(),
        (Language::Es, "gcs") => "Escala de Glasgow (15 - Actual)".to_string(),
        (Language::En, "chronic_health") => "Chronic Health Problems".to_string(),
        (Language::Es, "chronic_health") => "Problemas de Salud Crónicos".to_string(),
        (Language::En, "arf") => "Acute Renal Failure".to_string(),
        (Language::Es, "arf") => "Insuficiencia Renal Aguda".to_string(),

        // SOFA Fields
        (Language::En, "sofa_title") => "SOFA Score".to_string(),
        (Language::Es, "sofa_title") => "Puntaje SOFA".to_string(),
        (Language::En, "sofa_subtitle") => "Sequential Organ Failure Assessment".to_string(),
        (Language::Es, "sofa_subtitle") => "Evaluación Secuencial de Falla de Órganos".to_string(),
        (Language::En, "calculate_sofa") => "Calculate SOFA Score".to_string(),
        (Language::Es, "calculate_sofa") => "Calcular Puntaje SOFA".to_string(),

        (Language::En, "respiration") => "Respiration".to_string(),
        (Language::Es, "respiration") => "Respiración".to_string(),
        (Language::En, "pao2_fio2") => "PaO2/FiO2 Ratio (mmHg)".to_string(),
        (Language::Es, "pao2_fio2") => "Relación PaO2/FiO2 (mmHg)".to_string(),

        (Language::En, "coagulation") => "Coagulation".to_string(),
        (Language::Es, "coagulation") => "Coagulación".to_string(),
        (Language::En, "platelets") => "Platelets (×10³/µL)".to_string(),
        (Language::Es, "platelets") => "Plaquetas (×10³/µL)".to_string(),

        (Language::En, "liver") => "Liver".to_string(),
        (Language::Es, "liver") => "Hígado".to_string(),
        (Language::En, "bilirubin") => "Bilirubin (mg/dL)".to_string(),
        (Language::Es, "bilirubin") => "Bilirrubina (mg/dL)".to_string(),

        (Language::En, "cardiovascular") => "Cardiovascular".to_string(),
        (Language::Es, "cardiovascular") => "Cardiovascular".to_string(),
        (Language::En, "hemodynamic_status") => "Hemodynamic Status".to_string(),
        (Language::Es, "hemodynamic_status") => "Estado Hemodinámico".to_string(),

        (Language::En, "cns") => "Central Nervous System".to_string(),
        (Language::Es, "cns") => "Sistema Nervioso Central".to_string(),
        (Language::En, "gcs_score") => "Glasgow Coma Score (3-15)".to_string(),
        (Language::Es, "gcs_score") => "Puntaje Glasgow (3-15)".to_string(),

        (Language::En, "renal") => "Renal".to_string(),
        (Language::Es, "renal") => "Renal".to_string(),
        (Language::En, "creatinine_level") => "Creatinine Level".to_string(),
        (Language::Es, "creatinine_level") => "Nivel de Creatinina".to_string(),

        // SAPS II Fields
        (Language::En, "saps_title") => "SAPS II Score".to_string(),
        (Language::Es, "saps_title") => "Puntaje SAPS II".to_string(),
        (Language::En, "saps_subtitle") => "Simplified Acute Physiology Score II".to_string(),
        (Language::Es, "saps_subtitle") => {
            "Puntaje de Fisiología Aguda Simplificado II".to_string()
        }
        (Language::En, "calculate_saps") => "Calculate SAPS II Score".to_string(),
        (Language::Es, "calculate_saps") => "Calcular Puntaje SAPS II".to_string(),

        // Sections
        (Language::En, "vitals") => "Vitals".to_string(),
        (Language::Es, "vitals") => "Signos Vitales".to_string(),
        (Language::En, "oxygenation") => "Oxygenation".to_string(),
        (Language::Es, "oxygenation") => "Oxigenación".to_string(),
        (Language::En, "labs") => "Labs".to_string(),
        (Language::Es, "labs") => "Laboratorio".to_string(),
        (Language::En, "more_labs_cns") => "More Labs / CNS".to_string(),
        (Language::Es, "more_labs_cns") => "Más Lab / SNC".to_string(),
        (Language::En, "demographics_type") => "Demographics / Type".to_string(),
        (Language::Es, "demographics_type") => "Demografía / Tipo".to_string(),

        // Parameters
        (Language::En, "systolic_bp") => "Systolic BP (mmHg)".to_string(),
        (Language::Es, "systolic_bp") => "Presión Sistólica (mmHg)".to_string(),
        (Language::En, "ventilated_cpap") => "Ventilated / CPAP?".to_string(),
        (Language::Es, "ventilated_cpap") => "¿Ventilación Mecánica / CPAP?".to_string(),
        (Language::En, "urinary_output") => "Urinary Output (L/day)".to_string(),
        (Language::Es, "urinary_output") => "Gasto Urinario (L/día)".to_string(),
        (Language::En, "serum_urea") => "Serum Urea (g/L)".to_string(),
        (Language::Es, "serum_urea") => "Urea Sérica (g/L)".to_string(),
        (Language::En, "bicarbonate") => "Bicarbonate (mmol/L)".to_string(),
        (Language::Es, "bicarbonate") => "Bicarbonato (mmol/L)".to_string(),
        (Language::En, "admission_type") => "Admission Type".to_string(),
        (Language::Es, "admission_type") => "Tipo de Admisión".to_string(),
        (Language::En, "chronic_disease") => "Chronic Disease".to_string(),
        (Language::Es, "chronic_disease") => "Enfermedad Crónica".to_string(),

        // Select Options
        (Language::En, "none") => "None".to_string(),
        (Language::Es, "none") => "Ninguna".to_string(),
        (Language::En, "scheduled_surgical") => "Scheduled Surgical".to_string(),
        (Language::Es, "scheduled_surgical") => "Cirugía Programada".to_string(),
        (Language::En, "medical") => "Medical".to_string(),
        (Language::Es, "medical") => "Médico".to_string(),
        (Language::En, "unscheduled_surgical") => "Unscheduled Surgical".to_string(),
        (Language::Es, "unscheduled_surgical") => "Cirugía No Programada".to_string(),
        (Language::En, "metastatic_cancer") => "Metastatic Cancer".to_string(),
        (Language::Es, "metastatic_cancer") => "Cáncer Metastásico".to_string(),
        (Language::En, "hematologic_malignancy") => "Hematologic Malignancy".to_string(),
        (Language::Es, "hematologic_malignancy") => "Malignidad Hematológica".to_string(),
        (Language::En, "aids") => "AIDS".to_string(),
        (Language::Es, "aids") => "SIDA".to_string(),

        // Glasgow Fields
        (Language::En, "eye_response") => "Eye Response (1-4)".to_string(),
        (Language::Es, "eye_response") => "Respuesta Ocular (1-4)".to_string(),
        (Language::En, "verbal_response") => "Verbal Response (1-5)".to_string(),
        (Language::Es, "verbal_response") => "Respuesta Verbal (1-5)".to_string(),
        (Language::En, "motor_response") => "Motor Response (1-6)".to_string(),
        (Language::Es, "motor_response") => "Respuesta Motora (1-6)".to_string(),
        (Language::En, "spontaneous") => "Spontaneous".to_string(),
        (Language::Es, "spontaneous") => "Espontánea".to_string(),
        (Language::En, "to_voice") => "To Voice".to_string(),
        (Language::Es, "to_voice") => "Al hablarle".to_string(),
        (Language::En, "to_pain") => "To Pain".to_string(),
        (Language::Es, "to_pain") => "Al dolor".to_string(),
        (Language::En, "oriented") => "Oriented".to_string(),
        (Language::Es, "oriented") => "Orientado".to_string(),
        (Language::En, "confused") => "Confused".to_string(),
        (Language::Es, "confused") => "Confuso".to_string(),
        (Language::En, "words") => "Words".to_string(),
        (Language::Es, "words") => "Palabras".to_string(),
        (Language::En, "sounds") => "Sounds".to_string(),
        (Language::Es, "sounds") => "Sonidos".to_string(),
        (Language::En, "obeys") => "Obeys".to_string(),
        (Language::Es, "obeys") => "Obedece".to_string(),
        (Language::En, "localizes") => "Localizes".to_string(),
        (Language::Es, "localizes") => "Localiza".to_string(),
        (Language::En, "withdraws") => "Withdraws".to_string(),
        (Language::Es, "withdraws") => "Retira".to_string(),
        (Language::En, "flexion") => "Flexion".to_string(),
        (Language::Es, "flexion") => "Flexión".to_string(),
        (Language::En, "extension") => "Extension".to_string(),
        (Language::Es, "extension") => "Extensión".to_string(),
        (Language::En, "diagnosis") => "Diagnosis".to_string(),
        (Language::Es, "diagnosis") => "Diagnóstico".to_string(),
        (Language::En, "action") => "Action".to_string(),
        (Language::Es, "action") => "Acción".to_string(),

        // Patient Dashboard
        (Language::En, "patient_list") => "Patient List".to_string(),
        (Language::Es, "patient_list") => "Lista de Pacientes".to_string(),
        (Language::En, "add_patient") => "Add Patient".to_string(),
        (Language::Es, "add_patient") => "Agregar Paciente".to_string(),
        (Language::En, "view_history") => "View History".to_string(),
        (Language::Es, "view_history") => "Ver Historial".to_string(),
        (Language::En, "stable") => "Stable".to_string(),
        (Language::Es, "stable") => "Estable".to_string(),
        (Language::En, "critical") => "Critical".to_string(),
        (Language::Es, "critical") => "Crítico".to_string(),
        (Language::En, "bed") => "Bed".to_string(),
        (Language::Es, "bed") => "Cama".to_string(),
        (Language::En, "patient_detail") => "Patient Detail".to_string(),
        (Language::Es, "patient_detail") => "Detalle del Paciente".to_string(),
        (Language::En, "bio") => "Bio".to_string(),
        (Language::Es, "bio") => "Datos".to_string(),
        (Language::En, "history_assessments") => "Assessment History".to_string(),
        (Language::Es, "history_assessments") => "Historial de Evaluaciones".to_string(),
        (Language::En, "new_assessment") => "New Assessment".to_string(),
        (Language::Es, "new_assessment") => "Nueva Evaluación".to_string(),
        (Language::En, "no_history") => "No history available".to_string(),
        (Language::Es, "no_history") => "No hay historial disponible".to_string(),

        // Patient Registration
        (Language::En, "patient_registration") => "Patient Registration".to_string(),
        (Language::Es, "patient_registration") => "Registro de Paciente".to_string(),
        (Language::En, "enter_clinical_details") => "Enter clinical admission details".to_string(),
        (Language::Es, "enter_clinical_details") => {
            "Ingrese detalles clínicos de admisión".to_string()
        }
        (Language::En, "personal_information") => "Personal Information".to_string(),
        (Language::Es, "personal_information") => "Información Personal".to_string(),
        (Language::En, "first_name") => "First Name".to_string(),
        (Language::Es, "first_name") => "Nombre".to_string(),
        (Language::En, "last_name") => "Last Name".to_string(),
        (Language::Es, "last_name") => "Apellido".to_string(),
        (Language::En, "dob") => "Date of Birth".to_string(),
        (Language::Es, "dob") => "Fecha de Nacimiento".to_string(),
        (Language::En, "gender") => "Gender".to_string(),
        (Language::Es, "gender") => "Género".to_string(),
        (Language::En, "male") => "Male".to_string(),
        (Language::Es, "male") => "Masculino".to_string(),
        (Language::En, "female") => "Female".to_string(),
        (Language::Es, "female") => "Femenino".to_string(),
        (Language::En, "other") => "Other".to_string(),
        (Language::Es, "other") => "Otro".to_string(),
        (Language::En, "skin_color") => "Skin Color".to_string(),
        (Language::Es, "skin_color") => "Color de Piel".to_string(),
        (Language::En, "white") => "White".to_string(),
        (Language::Es, "white") => "Blanco".to_string(),
        (Language::En, "mixed") => "Mixed/Moreno".to_string(),
        (Language::Es, "mixed") => "Mixto/Moreno".to_string(),
        (Language::En, "black") => "Black".to_string(),
        (Language::Es, "black") => "Negro".to_string(),
        (Language::En, "clinical_information") => "Clinical Information".to_string(),
        (Language::Es, "clinical_information") => "Información Clínica".to_string(),
        (Language::En, "hospital_adm") => "Hospital Adm.".to_string(),
        (Language::Es, "hospital_adm") => "Ingreso Hosp.".to_string(),
        (Language::En, "uci_adm") => "UCI Adm.".to_string(),
        (Language::Es, "uci_adm") => "Ingreso UCI".to_string(),
        (Language::En, "days_in_hospital") => "Days in Hospital (Pre-UCI):".to_string(),
        (Language::Es, "days_in_hospital") => "Días en Hosp. (Pre-UCI):".to_string(),
        (Language::En, "principal_diagnosis") => "Principal Diagnosis".to_string(),
        (Language::Es, "principal_diagnosis") => "Diagnóstico Principal".to_string(),
        (Language::En, "enter_diagnosis_placeholder") => "Enter diagnosis here...".to_string(),
        (Language::Es, "enter_diagnosis_placeholder") => "Ingrese diagnóstico aquí...".to_string(),
        (Language::En, "mech_ventilation") => "Mech. Ventilation".to_string(),
        (Language::Es, "mech_ventilation") => "Vent. Mecánica".to_string(),
        (Language::En, "history_uci") => "History in UCI".to_string(),
        (Language::Es, "history_uci") => "Historial en UCI".to_string(),
        (Language::En, "transfer_other_center") => "Transfer (Other Center)".to_string(),
        (Language::Es, "transfer_other_center") => "Traslado (Otro Centro)".to_string(),
        (Language::En, "invasive_processes") => "Invasive Processes".to_string(),
        (Language::Es, "invasive_processes") => "Procesos Invasivos".to_string(),
        (Language::En, "urgent") => "Urgent".to_string(),
        (Language::Es, "urgent") => "Urgente".to_string(),
        (Language::En, "programmed") => "Programmed".to_string(),
        (Language::Es, "programmed") => "Programada".to_string(),
        (Language::En, "transfer") => "Transfer".to_string(),
        (Language::Es, "transfer") => "Traslado".to_string(),
        (Language::En, "register_patient_btn") => "Register Patient".to_string(),
        (Language::Es, "register_patient_btn") => "Registrar Paciente".to_string(),
        (Language::En, "success_register") => "Patient registered successfully!".to_string(),
        (Language::Es, "success_register") => "¡Paciente registrado con éxito!".to_string(),
        (Language::En, "invalid_dates") => "Invalid Dates".to_string(),
        (Language::Es, "invalid_dates") => "Fechas Inválidas".to_string(),
        (Language::En, "days") => "days".to_string(),
        (Language::Es, "days") => "días".to_string(),

        // Apache & SOFA Extra
        (Language::En, "vital_signs") => "Vital Signs".to_string(),
        (Language::Es, "vital_signs") => "Signos Vitales".to_string(),
        (Language::En, "oxygenation_ph") => "Oxygenation & pH".to_string(),
        (Language::Es, "oxygenation_ph") => "Oxigenación y pH".to_string(),
        (Language::En, "oxygenation_type") => "Oxygenation Type".to_string(),
        (Language::Es, "oxygenation_type") => "Tipo de Oxigenación".to_string(),
        (Language::En, "laboratory_values") => "Laboratory Values".to_string(),
        (Language::Es, "laboratory_values") => "Valores de Laboratorio".to_string(),
        (Language::En, "patient_data") => "Patient Data".to_string(),
        (Language::Es, "patient_data") => "Datos del Paciente".to_string(),
        (Language::En, "value_mmhg") => "Value (mmHg)".to_string(),
        (Language::Es, "value_mmhg") => "Valor (mmHg)".to_string(),
        (Language::En, "elective_surgery") => "Elective Surgery".to_string(),
        (Language::Es, "elective_surgery") => "Cirugía Electiva".to_string(),
        (Language::En, "non_elective_surgery") => "Non-Elective Surgery".to_string(),
        (Language::Es, "non_elective_surgery") => "Cirugía No Electiva".to_string(),
        (Language::En, "non_operative") => "Non-Operative".to_string(),
        (Language::Es, "non_operative") => "No Operatorio".to_string(),

        // SOFA Hints (Summarized)
        (Language::En, "pao2_hint") => "≥400: Normal | <100: Critical".to_string(),
        (Language::Es, "pao2_hint") => "≥400: Normal | <100: Crítico".to_string(),
        (Language::En, "platelets_hint") => "≥150: Normal | <20: Critical".to_string(),
        (Language::Es, "platelets_hint") => "≥150: Normal | <20: Crítico".to_string(),
        (Language::En, "bilirubin_hint") => "<1.2: Normal | ≥12: Critical".to_string(),
        (Language::Es, "bilirubin_hint") => "<1.2: Normal | ≥12: Crítico".to_string(),
        (Language::En, "vasopressor_hint") => "Vasopressor doses in µg/kg/min".to_string(),
        (Language::Es, "vasopressor_hint") => "Dosis vasopresores en µg/kg/min".to_string(),
        (Language::En, "gcs_hint") => "15: Normal | 3-5: Critical".to_string(),
        (Language::Es, "gcs_hint") => "15: Normal | 3-5: Crítico".to_string(),
        (Language::En, "renal_hint") => "Or urine output <500 mL/day".to_string(),
        (Language::Es, "renal_hint") => "O gasto urinario <500 mL/día".to_string(),

        // SOFA Options
        (Language::En, "map_70_plus") => "MAP ≥70 mmHg".to_string(),
        (Language::Es, "map_70_plus") => "PAM ≥70 mmHg".to_string(),
        (Language::En, "map_lt_70") => "MAP <70 mmHg".to_string(),
        (Language::Es, "map_lt_70") => "PAM <70 mmHg".to_string(),
        (Language::En, "dopa_lte5") => "Dopamine ≤5 or Dobutamine".to_string(),
        (Language::Es, "dopa_lte5") => "Dopamina ≤5 o Dobutamina".to_string(),
        (Language::En, "dopa_gt5") => "Dopamine >5".to_string(),
        (Language::Es, "dopa_gt5") => "Dopamina >5".to_string(),
        (Language::En, "dopa_gt15") => "Dopamine >15 or Epi/Norepi >0.1".to_string(),
        (Language::Es, "dopa_gt15") => "Dopamina >15 o Epi/Norepi >0.1".to_string(),
        (Language::En, "cr_lt_1_2") => "<1.2 mg/dL".to_string(),
        (Language::Es, "cr_lt_1_2") => "<1.2 mg/dL".to_string(),
        (Language::En, "cr_1_2_1_9") => "1.2 - 1.9 mg/dL".to_string(),
        (Language::Es, "cr_1_2_1_9") => "1.2 - 1.9 mg/dL".to_string(),
        (Language::En, "cr_2_0_3_4") => "2.0 - 3.4 mg/dL".to_string(),
        (Language::Es, "cr_2_0_3_4") => "2.0 - 3.4 mg/dL".to_string(),
        (Language::En, "cr_3_5_4_9") => "3.5 - 4.9 mg/dL".to_string(),
        (Language::Es, "cr_3_5_4_9") => "3.5 - 4.9 mg/dL".to_string(),
        (Language::En, "cr_gte_5") => "≥5.0 mg/dL".to_string(),
        (Language::Es, "cr_gte_5") => "≥5.0 mg/dL".to_string(),

        // Login Page
        (Language::En, "secure_access") => "Secure Access".to_string(),
        (Language::Es, "secure_access") => "Acceso Seguro".to_string(),
        (Language::En, "login_subtitle") => "Enter UCI control panel".to_string(),
        (Language::Es, "login_subtitle") => "Ingrese al panel de control UCI".to_string(),
        (Language::En, "username_placeholder") => "Username".to_string(),
        (Language::Es, "username_placeholder") => "Nombre de usuario".to_string(),
        (Language::En, "password_placeholder") => "••••••••".to_string(),
        (Language::Es, "password_placeholder") => "••••••••".to_string(),
        (Language::En, "login_btn") => "ENTER SYSTEM".to_string(),
        (Language::Es, "login_btn") => "ENTRAR AL SISTEMA".to_string(),
        (Language::En, "invalid_credentials") => "Invalid username or password".to_string(),
        (Language::Es, "invalid_credentials") => "Usuario o contraseña inválidos".to_string(),

        // Ward View
        (Language::En, "uci_monitor_title") => "UCI Central Monitor".to_string(),
        (Language::Es, "uci_monitor_title") => "Monitor Central UCI".to_string(),
        (Language::En, "patients_active") => "PATIENTS ACTIVE".to_string(),
        (Language::Es, "patients_active") => "PACIENTES ACTIVOS".to_string(),
        (Language::En, "next_check") => "NEXT CHECK:".to_string(),
        (Language::Es, "next_check") => "PRÓXIMA EVALUACIÓN:".to_string(),
        (Language::En, "view_details") => "View Details >".to_string(),
        (Language::Es, "view_details") => "Ver Detalles >".to_string(),

        // Admin Panel
        (Language::En, "system_config_title") => "System Configuration".to_string(),
        (Language::Es, "system_config_title") => "Configuración del Sistema".to_string(),
        (Language::En, "admin_panel_subtitle") => "Admin Panel - Full UCI Control".to_string(),
        (Language::Es, "admin_panel_subtitle") => "Panel Administrativo - Control Total UCI".to_string(),
        (Language::En, "tab_personal") => "Staff".to_string(),
        (Language::Es, "tab_personal") => "Personal".to_string(),
        (Language::En, "tab_interface") => "Interface".to_string(),
        (Language::Es, "tab_interface") => "Interfaz".to_string(),
        (Language::En, "authorized_personnel") => "Authorized Personnel".to_string(),
        (Language::Es, "authorized_personnel") => "Personal Autorizado".to_string(),
        (Language::En, "add_staff") => "ADD STAFF".to_string(),
        (Language::Es, "add_staff") => "AGREGAR PERSONAL".to_string(),
        (Language::En, "table_user") => "User".to_string(),
        (Language::Es, "table_user") => "Usuario".to_string(),
        (Language::En, "table_name") => "Full Name".to_string(),
        (Language::Es, "table_name") => "Nombre Completo".to_string(),
        (Language::En, "table_role") => "Role".to_string(),
        (Language::Es, "table_role") => "Rol".to_string(),
        (Language::En, "table_status") => "Status".to_string(),
        (Language::Es, "table_status") => "Estado".to_string(),
        (Language::En, "table_actions") => "Actions".to_string(),
        (Language::Es, "table_actions") => "Acciones".to_string(),
        (Language::En, "user_active") => "Active".to_string(),
        (Language::Es, "user_active") => "Activo".to_string(),
        (Language::En, "brand_colors") => "Brand Colors".to_string(),
        (Language::Es, "brand_colors") => "Colores de Marca".to_string(),
        (Language::En, "primary_color") => "Primary Color".to_string(),
        (Language::Es, "primary_color") => "Color Principal".to_string(),
        (Language::En, "save_visual_changes") => "Save Visual Changes".to_string(),
        (Language::Es, "save_visual_changes") => "Guardar Cambios Visuales".to_string(),
        (Language::En, "preview_view") => "Live Preview".to_string(),
        (Language::Es, "preview_view") => "Vista Previa".to_string(),
        (Language::En, "preview_button_label") => "Main Button Example".to_string(),
        (Language::Es, "preview_button_label") => "Ejemplo de Botón Principal".to_string(),
        (Language::En, "preview_button_text") => "ACCESS SYSTEM".to_string(),
        (Language::Es, "preview_button_text") => "ACCEDER AL SISTEMA".to_string(),
        (Language::En, "preview_description") => "The selected color will be applied to all key elements like buttons, active cards, and corporate branding.".to_string(),
        (Language::Es, "preview_description") => "El color seleccionado se aplicará a todos los elementos clave como botones, tarjetas activas y branding corporativo.".to_string(),

        // Navigation
        (Language::En, "nav_patients") => "Patients".to_string(),
        (Language::Es, "nav_patients") => "Pacientes".to_string(),
        (Language::En, "nav_scales") => "Scales".to_string(),
        (Language::Es, "nav_scales") => "Escalas".to_string(),
        (Language::En, "nav_monitor") => "Monitor".to_string(),
        (Language::Es, "nav_monitor") => "Monitor".to_string(),
        (Language::En, "nav_admin") => "Admin".to_string(),
        (Language::Es, "nav_admin") => "Admin".to_string(),
        (Language::En, "nav_logout") => "Logout".to_string(),
        (Language::Es, "nav_logout") => "Cerrar Sesión".to_string(),
        (Language::En, "nav_login") => "Login".to_string(),
        (Language::Es, "nav_login") => "Acceder".to_string(),

        // Home Features
        (Language::En, "feature_monitoring") => "Continuous Monitoring".to_string(),
        (Language::Es, "feature_monitoring") => "Monitoreo Continuo".to_string(),
        (Language::En, "feature_monitoring_desc") => "24/7 surveillance of vital signs and critical parameters in patients with life-threatening conditions.".to_string(),
        (Language::Es, "feature_monitoring_desc") => "Vigilancia 24/7 de signos vitales y parámetros críticos en pacientes con condiciones potencialmente mortales.".to_string(),
        (Language::En, "feature_team") => "Specialized Team".to_string(),
        (Language::Es, "feature_team") => "Equipo Especializado".to_string(),
        (Language::En, "feature_team_desc") => "Highly trained medical staff in critical care and intensive medicine.".to_string(),
        (Language::Es, "feature_team_desc") => "Personal médico altamente capacitado en cuidados críticos y medicina intensiva.".to_string(),
        (Language::En, "feature_scales") => "Clinical Scales".to_string(),
        (Language::Es, "feature_scales") => "Escalas Clínicas".to_string(),
        (Language::En, "feature_scales_desc") => "Objective assessment using APACHE II, SOFA, SAPS II and Glasgow to optimize treatments.".to_string(),
        (Language::Es, "feature_scales_desc") => "Evaluación objetiva mediante APACHE II, SOFA, SAPS II y Glasgow para optimizar tratamientos.".to_string(),

        // Additional Patient List & Trends
        (Language::En, "patient_list_overview") => "Overview of all active patients".to_string(),
        (Language::Es, "patient_list_overview") => "Vista general de todos los pacientes activos".to_string(),
        (Language::En, "export_csv") => "Export CSV".to_string(),
        (Language::Es, "export_csv") => "Exportar CSV".to_string(),
        (Language::En, "search_placeholder") => "Search by name, diagnosis or ID...".to_string(),
        (Language::Es, "search_placeholder") => "Buscar por nombre, diagnóstico o ID...".to_string(),
        (Language::En, "showing_n_patients") => "Showing {} patients".to_string(),
        (Language::Es, "showing_n_patients") => "Mostrando {} pacientes".to_string(),
        (Language::En, "of_n_patients") => " of {} patients".to_string(),
        (Language::Es, "of_n_patients") => " de {} pacientes".to_string(),
        (Language::En, "trend") => "Trend".to_string(),
        (Language::Es, "trend") => "Tendencia".to_string(),
        (Language::En, "no_trend_data") => "No trend data".to_string(),
        (Language::Es, "no_trend_data") => "Sin datos de tendencia".to_string(),
        (Language::En, "updated_today") => "Updated: Today".to_string(),
        (Language::Es, "updated_today") => "Actualizado: Hoy".to_string(),
        (Language::En, "saps_ii") => "SAPS II".to_string(),
        (Language::Es, "saps_ii") => "SAPS II".to_string(),

        // Alerts & Transitions
        (Language::En, "sepsis_alert") => "Sepsis Alert".to_string(),
        (Language::Es, "sepsis_alert") => "Alerta de Sepsis".to_string(),
        (Language::En, "qsofa_high_risk") => "High risk of sepsis (qSOFA ≥ 2)".to_string(),
        (Language::Es, "qsofa_high_risk") => "Alto riesgo de sepsis (qSOFA ≥ 2)".to_string(),
        (Language::En, "continue_assessment") => "Continue Assessment".to_string(),
        (Language::Es, "continue_assessment") => "Continuar Evaluación".to_string(),
        (Language::En, "print") => "Print".to_string(),
        (Language::Es, "print") => "Imprimir".to_string(),
        (Language::En, "print_report") => "Print Report".to_string(),
        (Language::Es, "print_report") => "Imprimir Reporte".to_string(),
        (Language::En, "load_last") => "Load Last".to_string(),
        (Language::Es, "load_last") => "Cargar Última".to_string(),
        (Language::En, "load_previous_desc") => "Load previous assessment".to_string(),
        (Language::Es, "load_previous_desc") => "Cargar evaluación previa".to_string(),
        (Language::En, "not_applicable_non_vent") => "Not applicable for non-ventilated patients".to_string(),
        (Language::Es, "not_applicable_non_vent") => "No aplicable para pacientes no ventilados".to_string(),
        (Language::En, "hr_hint") => "<40: 11pts | <70: 2pts | <120: 0pts | <160: 4pts | >160: 7pts".to_string(),
        (Language::Es, "hr_hint") => "<40: 11pts | <70: 2pts | <120: 0pts | <160: 4pts | >160: 7pts".to_string(),
        (Language::En, "gcs_saps_hint") => "<6: Severe | 6-8: Moderate | 9-10: Mild | 11-13: Normal".to_string(),
        (Language::Es, "gcs_saps_hint") => "<6: Severo | 6-8: Moderado | 9-10: Leve | 11-13: Normal".to_string(),

        // Detail View & Trends
        (Language::En, "sofa_trend") => "SOFA Trend (Last 24h)".to_string(),
        (Language::Es, "sofa_trend") => "Tendencia SOFA (Últimas 24h)".to_string(),
        (Language::En, "apache_trend") => "APACHE II Trend".to_string(),
        (Language::Es, "apache_trend") => "Tendencia APACHE II".to_string(),
        (Language::En, "visual_trends_soon") => "Visual Trends Chart (Coming Soon)".to_string(),
        (Language::Es, "visual_trends_soon") => "Gráfico de Tendencias (Próximamente)".to_string(),
        (Language::En, "table_date") => "Date".to_string(),
        (Language::Es, "table_date") => "Fecha".to_string(),
        (Language::En, "table_score") => "Score".to_string(),
        (Language::Es, "table_score") => "Puntaje".to_string(),
        (Language::En, "table_severity") => "Severity".to_string(),
        (Language::Es, "table_severity") => "Severidad".to_string(),
        (Language::En, "table_mortality") => "Mortality".to_string(),
        (Language::Es, "table_mortality") => "Mortalidad".to_string(),

        (Language::En, "no_data") => "No data enough for chart".to_string(),
        (Language::Es, "no_data") => "Sin datos suficientes".to_string(),

        // Delete functionality
        (Language::En, "delete") => "Delete".to_string(),
        (Language::Es, "delete") => "Eliminar".to_string(),
        (Language::En, "confirm_delete") => "Are you sure you want to delete this assessment?".to_string(),
        (Language::Es, "confirm_delete") => "¿Está seguro de eliminar esta evaluación?".to_string(),
        (Language::En, "delete_success") => "Assessment deleted successfully".to_string(),
        (Language::Es, "delete_success") => "Evaluación eliminada exitosamente".to_string(),
        (Language::En, "delete_error") => "Error deleting assessment".to_string(),
        (Language::Es, "delete_error") => "Error al eliminar evaluación".to_string(),

        _ => key.to_string(),
    }
}
