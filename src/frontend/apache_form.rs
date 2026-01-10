use crate::frontend::i18n::{t, use_i18n};
use crate::uci::scale::apache::{ApacheIIRequest, ApacheIIResponse};
use leptos::*;
use leptos_router::use_query_map;
use reqwasm::http::Request;

/// APACHE II Score form component - Comprehensive ICU severity assessment
#[component]
pub fn ApacheForm() -> impl IntoView {
    let lang = use_i18n();
    let query = use_query_map();
    let patient_id = move || query.get().get("patient_id").cloned();

    // Reactive signals for form inputs
    let (temperature, set_temperature) = create_signal(37.0f32);
    let (mean_arterial_pressure, set_mean_arterial_pressure) = create_signal(80i32);
    let (heart_rate, set_heart_rate) = create_signal(80i32);
    let (respiratory_rate, set_respiratory_rate) = create_signal(16i32);
    let (oxygenation_type, set_oxygenation_type) = create_signal("pao2".to_string());
    let (oxygenation_value, set_oxygenation_value) = create_signal(80i32);
    let (arterial_ph, set_arterial_ph) = create_signal(7.4f32);
    let (serum_sodium, set_serum_sodium) = create_signal(140i32);
    let (serum_potassium, set_serum_potassium) = create_signal(4.0f32);
    let (serum_creatinine, set_serum_creatinine) = create_signal(1.0f32);
    let (hematocrit, set_hematocrit) = create_signal(40.0f32);
    let (white_blood_count, set_white_blood_count) = create_signal(10.0f32);
    let (glasgow_coma_score, set_glasgow_coma_score) = create_signal(15u8);
    let (age, set_age) = create_signal(50u8);
    let (chronic_health, set_chronic_health) = create_signal("none".to_string());

    // Signal to store the result
    let (result, set_result) = create_signal(Option::<ApacheIIResponse>::None);
    let (loading, set_loading) = create_signal(false);

    // Smart Pre-fill Logic
    create_effect(move |_| {
        if let Some(id) = patient_id() {
            let id_clone = id.clone();
            spawn_local(async move {
                let token: Option<String> = window()
                    .local_storage()
                    .ok()
                    .flatten()
                    .and_then(|s| s.get_item("uci_token").ok().flatten());

                // 1. Fetch Patient Data for Age Pre-fill
                let pat_url = format!("/api/patients/{}", id_clone);
                let mut pat_req = Request::get(&pat_url);
                if let Some(t) = &token {
                    pat_req = pat_req.header("Authorization", &format!("Bearer {}", t));
                }

                if let Ok(res) = pat_req.send().await {
                    if let Ok(patient) = res.json::<crate::models::patient::Patient>().await {
                        if let Ok(dob) =
                            chrono::NaiveDate::parse_from_str(&patient.date_of_birth, "%Y-%m-%d")
                        {
                            let now = chrono::Local::now().naive_local().date();
                            let age_val = now.years_since(dob).unwrap_or(50) as u8;
                            set_age.set(age_val.clamp(18, 100));
                        }
                    }
                }

                // 2. Fetch History for Glasgow Pre-fill
                let hist_url = format!("/api/patients/{}/history", id_clone);
                let mut hist_req = Request::get(&hist_url);
                if let Some(t) = &token {
                    hist_req = hist_req.header("Authorization", &format!("Bearer {}", t));
                }

                if let Ok(res) = hist_req.send().await {
                    #[derive(serde::Deserialize)]
                    struct PartialHistory {
                        glasgow: Vec<crate::models::glasgow::GlasgowAssessment>,
                    }

                    if let Ok(history) = res.json::<PartialHistory>().await {
                        if let Some(latest) = history.glasgow.first() {
                            set_glasgow_coma_score.set(latest.score as u8);
                        }
                    }
                }
            });
        }
    });

    // Calculate function
    let calculate = move |_| {
        set_loading.set(true);

        let request = ApacheIIRequest {
            temperature: temperature.get(),
            mean_arterial_pressure: mean_arterial_pressure.get(),
            heart_rate: heart_rate.get(),
            respiratory_rate: respiratory_rate.get(),
            oxygenation_type: oxygenation_type.get(),
            oxygenation_value: oxygenation_value.get(),
            arterial_ph: arterial_ph.get(),
            serum_sodium: serum_sodium.get(),
            serum_potassium: serum_potassium.get(),
            serum_creatinine: serum_creatinine.get(),
            hematocrit: hematocrit.get(),
            white_blood_count: white_blood_count.get(),
            glasgow_coma_score: glasgow_coma_score.get(),
            age: age.get(),
            chronic_health: chronic_health.get(),
            patient_id: patient_id(),
        };

        spawn_local(async move {
            let token: Option<String> = window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("uci_token").ok().flatten());

            let mut req = Request::post("/api/apache").header("Content-Type", "application/json");

            if let Some(t) = token {
                req = req.header("Authorization", &format!("Bearer {}", t));
            }

            let response = req
                .body(serde_json::to_string(&request).unwrap())
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<ApacheIIResponse>().await {
                            set_result.set(Some(data));
                        }
                    }
                }
                Err(_) => {}
            }
            set_loading.set(false);
        });
    };

    view! {
            <div class="w-full max-w-7xl mx-auto px-4">
                // Header
                <div class="text-center mb-6">
                    <h2 class="text-2xl md:text-3xl font-bold bg-gradient-to-r from-red-600 to-orange-500 bg-clip-text text-transparent">
                        <i class="fas fa-heartbeat mr-2"></i>
                        {move || t(lang.get(), "apache_title")}
                    </h2>
                    <p class="text-sm text-gray-600 mt-1">{move || t(lang.get(), "apache_subtitle")}</p>
                </div>

                // Calculate Button (Top)
                <div class="flex justify-center mb-6">
                    <button
                        on:click=calculate
                        class="w-full md:w-auto px-8 py-4 bg-gradient-to-r from-red-600 to-orange-500 text-white font-bold text-lg rounded-xl shadow-lg hover:from-red-700 hover:to-orange-600 transition-all duration-200 transform hover:scale-105 flex items-center justify-center">
                        {move || if loading.get() {
                            view! { <i class="fas fa-spinner fa-spin mr-2"></i> }.into_view()
                        } else {
                            view! { <i class="fas fa-calculator mr-2"></i> }.into_view()
                        }}
                        {move || if loading.get() { t(lang.get(), "calculating") } else { t(lang.get(), "calculate_apache") }}
                    </button> // End of buttons div
                </div>

                // qSOFA Alert
                {move || {
                    let gcs = glasgow_coma_score.get();
                    let rr = respiratory_rate.get();
                    // qSOFA: RR >= 22, GCS < 15, SBP <= 100.
                    // We use MAP <= 75 as proxy for SBP <= 100 if SBP is missing, or just rely on RR and GCS.
                    // For safety, let's only warn if RR and GCS are definitely abnormal.

                    let high_risk = (if rr >= 22 {1} else {0}) +
                                    (if gcs < 15 {1} else {0}) +
                                    (if mean_arterial_pressure.get() <= 70 {1} else {0}) >= 2;

                    if high_risk {
                        view! {
                             <div class="mb-6 p-4 bg-red-50 border-l-4 border-red-500 rounded-r-lg shadow-sm animate-pulse-slow">
                                <div class="flex items-start">
                                    <div class="flex-shrink-0">
                                        <i class="fas fa-exclamation-triangle text-red-500 text-xl mt-1"></i>
                                    </div>
                                    <div class="ml-3">
                                        <h3 class="text-sm font-bold text-red-800 uppercase tracking-wide">
                                            {move || t(lang.get(), "sepsis_alert")}
                                        </h3>
                                        <p class="text-sm text-red-700 mt-1">
                                            {move || t(lang.get(), "qsofa_high_risk")}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div/> }.into_view()
                    }
                }}

                // Results (Top)
                <div class="min-h-[100px] mb-8">
                    {move || {
                        if let Some(data) = result.get() {
                            let (bg_color, text_color) = if data.score < 10 {
                                ("bg-green-500", "text-green-50")
                            } else if data.score < 25 {
                                ("bg-yellow-500", "text-yellow-50")
                            } else if data.score < 35 {
                                ("bg-orange-500", "text-orange-50")
                            } else {
                                ("bg-red-600", "text-red-50")
                            };

                            view! {
                                <div class=format!("{} {} rounded-xl shadow-lg transition-colors duration-700 animate-fade-in", bg_color, text_color)>
                                    <div class="p-4 grid grid-cols-1 md:grid-cols-4 gap-4 items-center">
                                        <div class="text-center md:border-r border-white/20">
                                            <div class="text-xs uppercase opacity-80 mb-1">
                                                <i class="fas fa-calculator mr-1"></i>{t(lang.get(), "score")}
                                            </div>
                                            <div class="text-4xl font-bold">
                                                {data.score}<span class="text-2xl opacity-80">"/71"</span>
                                            </div>
                                        </div>
                                        <div class="text-center md:border-r border-white/20">
                                            <div class="text-xs uppercase opacity-80 mb-1">
                                                <i class="fas fa-skull-crossbones mr-1"></i>{t(lang.get(), "mortality")}
                                            </div>
                                            <div class="text-2xl font-bold">
                                                {format!("{:.1}%", data.predicted_mortality)}
                                            </div>
                                        </div>
                                        <div class="text-center md:text-left">
                                            <div class="text-xs uppercase opacity-80 mb-1">
                                                <i class="fas fa-exclamation-triangle mr-1"></i>{t(lang.get(), "severity")}
                                            </div>
                                            <div class="font-semibold text-sm">{data.severity}</div>
                                        </div>
                                        <div class="text-center md:text-left">
                                            <div class="text-xs uppercase opacity-80 mb-1">
                                                <i class="fas fa-notes-medical mr-1"></i>{t(lang.get(), "recommendation")}
                                            </div>
                                            <div class="font-semibold text-sm">{data.recommendation}</div>
                                        </div>
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="text-center p-6 bg-blue-50 border border-blue-100 rounded-xl text-blue-800">
                                    <i class="fas fa-info-circle text-2xl mb-2"></i>
                                    <p>{move || t(lang.get(), "complete_form_hint")}</p>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>

                // Chaining
                {move || {
                    if result.get().is_some() {
                        if let Some(pid) = patient_id() {
                            view! {
                                <div class="mb-8 p-4 bg-gradient-to-r from-red-50 to-orange-50 border border-red-100 rounded-xl animate-fade-in shadow-sm">
                                    <h4 class="text-sm font-bold text-red-800 mb-3 flex items-center">
                                        <i class="fas fa-forward mr-2"></i>{t(lang.get(), "continue_assessment")}
                                    </h4>
                                    <div class="flex flex-wrap gap-3">
                                        <a href=format!("/glasgow?patient_id={}", pid)
                                           class="flex items-center px-4 py-2 bg-white text-purple-600 border border-purple-200 rounded-lg hover:bg-purple-600 hover:text-white hover:border-purple-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                           <i class="fas fa-brain mr-2"></i>{move || t(lang.get(), "glasgow_scale")}
                                        </a>
                                        <a href=format!("/sofa?patient_id={}", pid)
                                           class="flex items-center px-4 py-2 bg-white text-teal-600 border border-teal-200 rounded-lg hover:bg-teal-600 hover:text-white hover:border-teal-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                           <i class="fas fa-procedures mr-2"></i>{move || t(lang.get(), "sofa_score")}
                                        </a>
                                        <a href=format!("/saps?patient_id={}", pid)
                                           class="flex items-center px-4 py-2 bg-white text-orange-600 border border-orange-200 rounded-lg hover:bg-orange-600 hover:text-white hover:border-orange-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                           <i class="fas fa-notes-medical mr-2"></i>{move || t(lang.get(), "saps_ii")}
                                        </a>
                                        <button
                                            class="flex items-center px-4 py-2 bg-white text-gray-600 border border-gray-200 rounded-lg hover:bg-gray-600 hover:text-white hover:border-gray-600 transition-colors duration-200 shadow-sm text-sm font-bold no-print"
                                            on:click=move |_| {
                                                let _ = web_sys::window().unwrap().print();
                                            }
                                        >
                                            <i class="fas fa-print mr-2"></i>{move || t(lang.get(), "print")}
                                        </button>
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                             view! { <div/> }.into_view()
                        }
                    } else {
                         view! { <div/> }.into_view()
                    }
                }}

                // Form Sections
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    // Vital Signs
                    <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-red-500">
                        <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                            <i class="fas fa-temperature-high text-red-600 mr-2"></i>{move || t(lang.get(), "vital_signs")}
                        </h3>
                        <div class="space-y-6">
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-thermometer-half text-red-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "temperature")}</span>
                                    <span class="text-red-600 font-bold">{move || format!("{:.1}", temperature.get())}</span>
                                </label>
                                <input type="range" min="20" max="45" step="0.1"
                                    prop:value=move || temperature.get()
                                    on:input=move |ev| set_temperature.set(event_target_value(&ev).parse().unwrap_or(37.0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-red-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-tachometer-alt text-red-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "map")}</span>
                                    <span class="text-red-600 font-bold">{move || mean_arterial_pressure.get()}</span>
                                </label>
                                <input type="range" min="0" max="200" step="1"
                                    prop:value=move || mean_arterial_pressure.get()
                                    on:input=move |ev| set_mean_arterial_pressure.set(event_target_value(&ev).parse().unwrap_or(80))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-red-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-heart-pulse text-red-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "heart_rate")}</span>
                                    <span class="text-red-600 font-bold">{move || heart_rate.get()}</span>
                                </label>
                                <input type="range" min="0" max="250" step="1"
                                    prop:value=move || heart_rate.get()
                                    on:input=move |ev| set_heart_rate.set(event_target_value(&ev).parse().unwrap_or(80))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-red-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-wind text-red-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "respiratory_rate")}</span>
                                    <span class="text-red-600 font-bold">{move || respiratory_rate.get()}</span>
                                </label>
                                <input type="range" min="0" max="60" step="1"
                                    prop:value=move || respiratory_rate.get()
                                    on:input=move |ev| set_respiratory_rate.set(event_target_value(&ev).parse().unwrap_or(16))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-red-600"/>
                            </div>
                        </div>
                    </div>

                    // Oxygenation
                    <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-blue-500">
                        <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                            <i class="fas fa-lungs text-blue-600 mr-2"></i>{move || t(lang.get(), "oxygenation_ph")}
                        </h3>
                        <div class="space-y-6">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1 flex items-center"><i class="fas fa-sliders-h text-blue-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "oxygenation_type")}</label>
                                <select
                                    on:change=move |ev| set_oxygenation_type.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500">
                                    <option value="pao2" selected>{move || t(lang.get(), "pao2")}</option>
                                    <option value="aa_gradient">{move || t(lang.get(), "a_a_gradient")}</option>
                                </select>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-lungs text-blue-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "value_mmhg")}</span>
                                    <span class="text-blue-600 font-bold">{move || oxygenation_value.get()}</span>
                                </label>
                                <input type="range" min="0" max="800" step="1"
                                    prop:value=move || oxygenation_value.get()
                                    on:input=move |ev| set_oxygenation_value.set(event_target_value(&ev).parse().unwrap_or(80))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-vial text-blue-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "arterial_ph")}</span>
                                    <span class="text-blue-600 font-bold">{move || format!("{:.2}", arterial_ph.get())}</span>
                                </label>
                                <input type="range" min="6.8" max="7.8" step="0.01"
                                    prop:value=move || arterial_ph.get()
                                    on:input=move |ev| set_arterial_ph.set(event_target_value(&ev).parse().unwrap_or(7.4))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"/>
                            </div>
                        </div>
                    </div>

                    // Laboratory Values
                    <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-purple-500">
                        <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                            <i class="fas fa-flask text-purple-600 mr-2"></i>{move || t(lang.get(), "laboratory_values")}
                        </h3>
                        <div class="space-y-6">
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-cube text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "sodium")}</span>
                                    <span class="text-purple-600 font-bold">{move || serum_sodium.get()}</span>
                                </label>
                                <input type="range" min="100" max="180" step="1"
                                    prop:value=move || serum_sodium.get()
                                    on:input=move |ev| set_serum_sodium.set(event_target_value(&ev).parse().unwrap_or(140))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-bolt text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "potassium")}</span>
                                    <span class="text-purple-600 font-bold">{move || format!("{:.1}", serum_potassium.get())}</span>
                                </label>
                                <input type="range" min="1.0" max="8.0" step="0.1"
                                    prop:value=move || serum_potassium.get()
                                    on:input=move |ev| set_serum_potassium.set(event_target_value(&ev).parse().unwrap_or(4.0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-filter text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "creatinine")}</span>
                                    <span class="text-purple-600 font-bold">{move || format!("{:.1}", serum_creatinine.get())}</span>
                                </label>
                                <input type="range" min="0.0" max="15.0" step="0.1"
                                    prop:value=move || serum_creatinine.get()
                                    on:input=move |ev| set_serum_creatinine.set(event_target_value(&ev).parse().unwrap_or(1.0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-tint text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "hematocrit")}</span>
                                    <span class="text-purple-600 font-bold">{move || format!("{:.1}", hematocrit.get())}</span>
                                </label>
                                <input type="range" min="10.0" max="60.0" step="0.1"
                                    prop:value=move || hematocrit.get()
                                    on:input=move |ev| set_hematocrit.set(event_target_value(&ev).parse().unwrap_or(40.0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-shield-virus text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "wbc")}</span>
                                    <span class="text-purple-600 font-bold">{move || format!("{:.1}", white_blood_count.get())}</span>
                                </label>
                                <input type="range" min="0.0" max="50.0" step="0.1"
                                    prop:value=move || white_blood_count.get()
                                    on:input=move |ev| set_white_blood_count.set(event_target_value(&ev).parse().unwrap_or(10.0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                            </div>
                        </div>
                    </div>

                    // Neurological & Demographics
                    <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-indigo-500">
                        <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                            <i class="fas fa-user-md text-indigo-600 mr-2"></i>{move || t(lang.get(), "patient_data")}
                        </h3>
                        <div class="space-y-6">
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-brain text-indigo-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "gcs")}</span>
                                    <span class="text-indigo-600 font-bold">{move || glasgow_coma_score.get()}</span>
                                </label>
                                <input type="range" min="3" max="15" step="1"
                                    prop:value=move || glasgow_coma_score.get()
                                    on:input=move |ev| set_glasgow_coma_score.set(event_target_value(&ev).parse().unwrap_or(15))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"/>
                            </div>
                            <div>
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span class="flex items-center"><i class="fas fa-hourglass-half text-indigo-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "age")}</span>
                                    <span class="text-indigo-600 font-bold">{move || age.get()}</span>
                                </label>
                                <input type="range" min="18" max="100" step="1"
                                    prop:value=move || age.get()
                                    on:input=move |ev| set_age.set(event_target_value(&ev).parse().unwrap_or(50))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"/>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1 flex items-center"><i class="fas fa-procedures text-indigo-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "chronic_health")}</label>
                                <select
                                    on:change=move |ev| set_chronic_health.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500">
                                    <option value="none" selected>{move || t(lang.get(), "none")}</option>
                                    <option value="elective">{move || t(lang.get(), "elective_surgery")}</option>
                                    <option value="non_elective">{move || t(lang.get(), "non_elective_surgery")}</option>
                                    <option value="non_operative">{move || t(lang.get(), "non_operative")}</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
    }
}
