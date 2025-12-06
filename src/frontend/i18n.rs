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

        _ => key.to_string(),
    }
}
