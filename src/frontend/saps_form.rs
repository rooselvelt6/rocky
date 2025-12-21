use crate::frontend::i18n::{t, use_i18n};
use crate::uci::scale::saps::{SAPSIIRequest, SAPSIIResponse};
use leptos::*;
use leptos_router::use_query_map;
use reqwasm::http::Request;

/// SAPS II Score form component
#[component]
pub fn SapsForm() -> impl IntoView {
    let lang = use_i18n();
    let query = use_query_map();
    let patient_id = move || query.get().get("patient_id").cloned();

    // Signals
    let (age, set_age) = create_signal(50u8);
    let (heart_rate, set_heart_rate) = create_signal(80i32);
    let (systolic_bp, set_systolic_bp) = create_signal(120i32);
    let (temperature, set_temperature) = create_signal(37.0f32);

    // PaO2/FiO2 is optional (only if specific ventilation support)
    let (ventilated, set_ventilated) = create_signal(false);
    let (pao2_fio2, set_pao2_fio2) = create_signal(300i32);

    let (urinary_output, set_urinary_output) = create_signal(1.5f32);
    let (serum_urea, set_serum_urea) = create_signal(10.0f32);
    let (white_blood_count, set_white_blood_count) = create_signal(8.0f32);
    let (serum_potassium, set_serum_potassium) = create_signal(4.0f32);
    let (serum_sodium, set_serum_sodium) = create_signal(140i32);
    let (serum_bicarbonate, set_serum_bicarbonate) = create_signal(24.0f32);
    let (bilirubin, set_bilirubin) = create_signal(0.8f32);
    let (glasgow, set_glasgow) = create_signal(15u8);

    let (chronic_disease, set_chronic_disease) = create_signal("none".to_string());
    let (admission_type, set_admission_type) = create_signal("medical".to_string());

    let (result, set_result) = create_signal(Option::<SAPSIIResponse>::None);
    let (loading, set_loading) = create_signal(false);

    // Smart Pre-fill Logic
    create_effect(move |_| {
        if let Some(id) = patient_id() {
            let id_clone = id.clone();
            spawn_local(async move {
                // 1. Fetch Patient Data for Age Pre-fill
                let pat_url = format!("/api/patients/{}", id_clone);
                if let Ok(res) = Request::get(&pat_url).send().await {
                    if let Ok(patient) = res.json::<crate::models::patient::Patient>().await {
                        if let Ok(dob) =
                            chrono::NaiveDate::parse_from_str(&patient.date_of_birth, "%Y-%m-%d")
                        {
                            let now = chrono::Local::now().naive_local().date();
                            let age_val = now.years_since(dob).unwrap_or(50) as u8;
                            set_age.set(age_val.clamp(18, 110));
                        }
                    }
                }

                // 2. Fetch History for Glasgow Pre-fill
                let hist_url = format!("/api/patients/{}/history", id_clone);
                if let Ok(res) = Request::get(&hist_url).send().await {
                    #[derive(serde::Deserialize)]
                    struct PartialHistory {
                        glasgow: Vec<crate::models::glasgow::GlasgowAssessment>,
                    }

                    if let Ok(history) = res.json::<PartialHistory>().await {
                        if let Some(latest) = history.glasgow.first() {
                            set_glasgow.set(latest.score as u8);
                        }
                    }
                }
            });
        }
    });

    let calculate = move |_| {
        set_loading.set(true);

        let request = SAPSIIRequest {
            age: age.get(),
            heart_rate: heart_rate.get(),
            systolic_bp: systolic_bp.get(),
            temperature: temperature.get(),
            pao2_fio2: if ventilated.get() {
                Some(pao2_fio2.get())
            } else {
                None
            },
            urinary_output: urinary_output.get(),
            serum_urea: serum_urea.get(),
            white_blood_count: white_blood_count.get(),
            serum_potassium: serum_potassium.get(),
            serum_sodium: serum_sodium.get(),
            serum_bicarbonate: serum_bicarbonate.get(),
            bilirubin: bilirubin.get(),
            glasgow: glasgow.get(),
            chronic_disease: chronic_disease.get(),
            admission_type: admission_type.get(),
            patient_id: patient_id(),
        };

        spawn_local(async move {
            let response = Request::post("/api/saps")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&request).unwrap())
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<SAPSIIResponse>().await {
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
        <div class="w-full max-w-6xl mx-auto px-4 pb-20">
            // Header
            <div class="text-center mb-6">
                <h2 class="text-2xl md:text-3xl font-bold bg-gradient-to-r from-orange-600 to-red-500 bg-clip-text text-transparent">
                    <i class="fas fa-notes-medical mr-2"></i>
                    {move || t(lang.get(), "saps_title")}
                </h2>
                <p class="text-sm text-gray-600 mt-1">{move || t(lang.get(), "saps_subtitle")}</p>
            </div>

            // Calculate Button
            <div class="flex justify-center mb-6">
                 <button
                    on:click=calculate
                    class="w-full md:w-auto px-8 py-4 bg-gradient-to-r from-orange-600 to-red-500 text-white font-bold text-lg rounded-xl shadow-lg hover:from-orange-700 hover:to-red-600 transition-all duration-200 transform hover:scale-105 flex items-center justify-center">
                    {move || if loading.get() {
                        view! { <i class="fas fa-spinner fa-spin mr-2"></i> }.into_view()
                    } else {
                        view! { <i class="fas fa-calculator mr-2"></i> }.into_view()
                    }}
                    {move || if loading.get() { t(lang.get(), "calculating") } else { t(lang.get(), "calculate_saps") }}
                </button>
            </div>

            // Result Display
            <div class="min-h-[100px] mb-8">
                {move || {
                    if let Some(data) = result.get() {
                        let (bg_color, text_color) = if data.score <= 30 {
                            ("bg-green-500", "text-white")
                        } else if data.score <= 50 {
                            ("bg-yellow-500", "text-white")
                        } else if data.score <= 70 {
                            ("bg-orange-500", "text-white")
                        } else {
                            ("bg-red-600", "text-white")
                        };

                        view! {
                            <div class=format!("{} {} rounded-xl shadow-lg transition-colors duration-700 animate-fade-in", bg_color, text_color)>
                                <div class="p-4 grid grid-cols-1 md:grid-cols-3 gap-4 items-center">
                                    <div class="text-center md:border-r border-white/20">
                                        <div class="text-xs uppercase opacity-80 mb-1">
                                            <i class="fas fa-calculator mr-1"></i>{t(lang.get(), "score")}
                                        </div>
                                        <div class="text-4xl font-bold">
                                            {data.score}
                                        </div>
                                    </div>
                                    <div class="text-center md:text-left">
                                         <div class="text-xs uppercase opacity-80 mb-1">
                                            <i class="fas fa-skull-crossbones mr-1"></i>{t(lang.get(), "mortality")}
                                        </div>
                                        <div class="font-semibold text-2xl">{format!("{:.1}%", data.predicted_mortality)}</div>
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
                        view! { <div/> }.into_view()
                    }
                }}
            </div>

            // Form Layout
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">

                // --- Vitals ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-orange-500">
                    <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-heartbeat mr-2 text-orange-600"></i>{move || t(lang.get(), "vitals")}</h3>

                    <div class="mb-4">
                         <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-heart text-red-500 mr-2"></i>{move || t(lang.get(), "heart_rate")}</span>
                            <span class="font-bold text-orange-600">{move || heart_rate.get()}</span>
                        </label>
                        <input type="range" min="20" max="250" step="1" prop:value=move || heart_rate.get()
                            on:input=move |ev| set_heart_rate.set(event_target_value(&ev).parse().unwrap_or(0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-orange-600"/>
                         <p class="text-xs text-gray-400 mt-1">"<40: 11pts | <70: 2pts | <120: 0pts | <160: 4pts | >160: 7pts"</p>
                    </div>
                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-compress-alt text-orange-500 mr-2"></i>{move || t(lang.get(), "systolic_bp")}</span>
                            <span class="font-bold text-orange-600">{move || systolic_bp.get()}</span>
                        </label>
                        <input type="range" min="40" max="250" step="2" prop:value=move || systolic_bp.get()
                            on:input=move |ev| set_systolic_bp.set(event_target_value(&ev).parse().unwrap_or(0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-orange-600"/>
                    </div>
                     <div class="mb-4">
                         <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-thermometer-half text-red-500 mr-2"></i>{move || t(lang.get(), "temperature")}</span>
                            <span class="font-bold text-orange-600">{move || temperature.get()}</span>
                        </label>
                        <input type="range" min="32.0" max="42.0" step="0.1" prop:value=move || temperature.get()
                            on:input=move |ev| set_temperature.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-orange-600"/>
                    </div>
                </div>

                // --- Oxygenation ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-blue-500">
                     <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-lungs mr-2 text-blue-600"></i>{move || t(lang.get(), "oxygenation")}</h3>

                     <div class="mb-4">
                        <label class="flex items-center space-x-2 cursor-pointer p-3 bg-blue-50 rounded-lg hover:bg-blue-100 transition">
                            <input type="checkbox" prop:checked=move || ventilated.get()
                                on:change=move |ev| set_ventilated.set(event_target_checked(&ev))
                                class="rounded text-blue-600 focus:ring-blue-500 h-5 w-5"/>
                            <span class="text-sm font-medium text-gray-700"><i class="fas fa-mask text-blue-500 mr-2"></i>{move || t(lang.get(), "ventilated_cpap")}</span>
                        </label>
                     </div>

                     {move || if ventilated.get() {
                         view! {
                             <div class="mb-4 animate-fade-in">
                                <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span><i class="fas fa-wind text-blue-400 mr-2"></i>{move || t(lang.get(), "pao2_fio2")}</span>
                                    <span class="font-bold text-blue-600">{move || pao2_fio2.get()}</span>
                                </label>
                                <input type="range" min="50" max="600" step="10" prop:value=move || pao2_fio2.get()
                                    on:input=move |ev| set_pao2_fio2.set(event_target_value(&ev).parse().unwrap_or(0))
                                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"/>
                             </div>
                         }.into_view()
                     } else {
                         view! { <div class="text-sm text-gray-500 italic p-3 bg-gray-50 rounded border border-gray-200 text-center">"Not applicable for non-ventilated patients"</div> }.into_view()
                     }}
                </div>

                // --- Renal ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-yellow-500">
                     <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-tint mr-2 text-yellow-600"></i>{move || t(lang.get(), "renal")}</h3>

                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-faucet text-yellow-500 mr-2"></i>{move || t(lang.get(), "urinary_output")}</span>
                            <span class="font-bold text-yellow-600">{move || urinary_output.get()}</span>
                        </label>
                        <input type="range" min="0.0" max="5.0" step="0.1" prop:value=move || urinary_output.get()
                            on:input=move |ev| set_urinary_output.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-yellow-600"/>
                     </div>
                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-flask text-yellow-500 mr-2"></i>{move || t(lang.get(), "serum_urea")}</span>
                            <span class="font-bold text-yellow-600">{move || serum_urea.get()}</span>
                        </label>
                        <input type="range" min="0" max="150" step="1" prop:value=move || serum_urea.get()
                            on:input=move |ev| set_serum_urea.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-yellow-600"/>
                     </div>
                </div>

                // --- Labs 1 ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-purple-500">
                     <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-microscope mr-2 text-purple-600"></i>{move || t(lang.get(), "labs")}</h3>

                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-bacterium text-purple-500 mr-2"></i>{move || t(lang.get(), "wbc")}</span>
                            <span class="font-bold text-purple-600">{move || white_blood_count.get()}</span>
                        </label>
                        <input type="range" min="0.0" max="50.0" step="0.5" prop:value=move || white_blood_count.get()
                            on:input=move |ev| set_white_blood_count.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                     </div>
                      <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-bolt text-yellow-400 mr-2"></i>{move || t(lang.get(), "potassium")}</span>
                            <span class="font-bold text-purple-600">{move || serum_potassium.get()}</span>
                        </label>
                        <input type="range" min="1.0" max="10.0" step="0.1" prop:value=move || serum_potassium.get()
                            on:input=move |ev| set_serum_potassium.set(event_target_value(&ev).parse().unwrap_or(0.0))
                             class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                     </div>
                      <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-cube text-blue-300 mr-2"></i>{move || t(lang.get(), "sodium")}</span>
                            <span class="font-bold text-purple-600">{move || serum_sodium.get()}</span>
                        </label>
                        <input type="range" min="100" max="180" step="1" prop:value=move || serum_sodium.get()
                            on:input=move |ev| set_serum_sodium.set(event_target_value(&ev).parse().unwrap_or(0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                     </div>
                </div>

                 // --- Labs 2 + Glasgow ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-indigo-500">
                     <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-vial mr-2 text-indigo-600"></i>{move || t(lang.get(), "more_labs_cns")}</h3>

                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-atom text-indigo-400 mr-2"></i>{move || t(lang.get(), "bicarbonate")}</span>
                            <span class="font-bold text-indigo-600">{move || serum_bicarbonate.get()}</span>
                        </label>
                        <input type="range" min="5" max="50" step="1" prop:value=move || serum_bicarbonate.get()
                            on:input=move |ev| set_serum_bicarbonate.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"/>
                     </div>
                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-poop text-yellow-700 mr-2"></i>{move || t(lang.get(), "bilirubin")}</span>
                            <span class="font-bold text-indigo-600">{move || bilirubin.get()}</span>
                        </label>
                        <input type="range" min="0.0" max="30.0" step="0.5" prop:value=move || bilirubin.get()
                            on:input=move |ev| set_bilirubin.set(event_target_value(&ev).parse().unwrap_or(0.0))
                             class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"/>
                     </div>
                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-brain text-pink-500 mr-2"></i>{move || t(lang.get(), "gcs_score")}</span>
                            <span class="font-bold text-indigo-600">{move || glasgow.get()}</span>
                        </label>
                        <input type="range" min="3" max="15" prop:value=move || glasgow.get()
                            on:input=move |ev| set_glasgow.set(event_target_value(&ev).parse().unwrap_or(15))
                             class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"/>
                        <p class="text-xs text-gray-400 mt-1">"<6: Severe | 6-8: Moderate | 9-10: Mild | 11-13: Normal"</p>
                     </div>
                </div>

                // --- Demographics ---
                <div class="bg-white p-6 rounded-xl shadow-md border-t-4 border-gray-500">
                     <h3 class="font-bold text-gray-800 mb-4 border-b pb-2"><i class="fas fa-id-card mr-2 text-gray-600"></i>{move || t(lang.get(), "demographics_type")}</h3>

                     <div class="mb-4">
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span><i class="fas fa-birthday-cake text-pink-400 mr-2"></i>{move || t(lang.get(), "age")}</span>
                            <span class="font-bold text-gray-600">{move || age.get()}</span>
                        </label>
                        <input type="range" min="18" max="110" step="1" prop:value=move || age.get()
                            on:input=move |ev| set_age.set(event_target_value(&ev).parse().unwrap_or(50))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-gray-600"/>
                     </div>

                     <div class="mb-4">
                         <label class="block text-sm font-medium text-gray-700 mb-1"><i class="fas fa-hospital-user mr-2 text-gray-500"></i>{move || t(lang.get(), "admission_type")}</label>
                        <select
                            on:change=move |ev| set_admission_type.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-gray-500">
                            <option value="scheduled" selected="selected">{move || t(lang.get(), "scheduled_surgical")}</option>
                            <option value="medical">{move || t(lang.get(), "medical")}</option>
                            <option value="unscheduled">{move || t(lang.get(), "unscheduled_surgical")}</option>
                        </select>
                     </div>

                     <div class="mb-4">
                        <label class="block text-sm font-medium text-gray-700 mb-1"><i class="fas fa-virus mr-2 text-green-700"></i>{move || t(lang.get(), "chronic_disease")}</label>
                        <select
                            on:change=move |ev| set_chronic_disease.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-gray-500">
                            <option value="none" selected="selected">{move || t(lang.get(), "none")}</option>
                            <option value="cancer">{move || t(lang.get(), "metastatic_cancer")}</option>
                            <option value="hematologic">{move || t(lang.get(), "hematologic_malignancy")}</option>
                            <option value="aids">{move || t(lang.get(), "aids")}</option>
                        </select>
                     </div>
                </div>
            </div>
        </div>
    }
}
