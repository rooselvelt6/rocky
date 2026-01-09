use crate::frontend::i18n::{t, use_i18n};
use crate::uci::scale::sofa::{SOFARequest, SOFAResponse};
use leptos::*;
use leptos_router::use_query_map;
use reqwasm::http::Request;

/// SOFA Score form component - Sequential Organ Failure Assessment
#[component]
pub fn SofaForm() -> impl IntoView {
    let lang = use_i18n();
    let query = use_query_map();
    let patient_id = move || query.get().get("patient_id").cloned();

    // Reactive signals for form inputs
    let (pao2_fio2, set_pao2_fio2) = create_signal(400i32);
    let (platelets, set_platelets) = create_signal(150i32);
    let (bilirubin, set_bilirubin) = create_signal(1.0f32);
    let (cardiovascular, set_cardiovascular) = create_signal("map_70_plus".to_string());
    let (glasgow, set_glasgow) = create_signal(15u8);
    let (renal, set_renal) = create_signal("cr_lt_1_2".to_string());

    // Signals to store the result
    let (result, set_result) = create_signal(Option::<SOFAResponse>::None);
    let (loading, set_loading) = create_signal(false);

    // Smart Pre-fill Logic
    create_effect(move |_| {
        if let Some(id) = patient_id() {
            let id_clone = id.clone();
            spawn_local(async move {
                let token = window()
                    .local_storage()
                    .ok()
                    .flatten()
                    .and_then(|s| s.get_item("uci_token").ok().flatten());

                // Fetch History for Glasgow Pre-fill
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
                            set_glasgow.set(latest.score as u8);
                        }
                    }
                }
            });
        }
    });

    // Calculate function
    let calculate = move |_| {
        set_loading.set(true);

        let request = SOFARequest {
            pao2_fio2: pao2_fio2.get(),
            platelets: platelets.get(),
            bilirubin: bilirubin.get(),
            cardiovascular: cardiovascular.get(),
            glasgow: glasgow.get(),
            renal: renal.get(),
            patient_id: patient_id(),
        };

        spawn_local(async move {
            let token = window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("uci_token").ok().flatten());

            let mut req = Request::post("/api/sofa").header("Content-Type", "application/json");

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
                        if let Ok(data) = resp.json::<SOFAResponse>().await {
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
        <div class="w-full max-w-6xl mx-auto px-4">
            // Header
            <div class="text-center mb-6">
                <h2 class="text-2xl md:text-3xl font-bold bg-gradient-to-r from-teal-600 to-cyan-500 bg-clip-text text-transparent">
                    <i class="fas fa-procedures mr-2"></i>
                    {move || t(lang.get(), "sofa_title")}
                </h2>
                <p class="text-sm text-gray-600 mt-1">{move || t(lang.get(), "sofa_subtitle")}</p>
            </div>

            // Calculate Button (Top)
            <div class="flex justify-center mb-6">
                <button
                    on:click=calculate
                    class="w-full md:w-auto px-8 py-4 bg-gradient-to-r from-teal-600 to-cyan-500 text-white font-bold text-lg rounded-xl shadow-lg hover:from-teal-700 hover:to-cyan-600 transition-all duration-200 transform hover:scale-105 flex items-center justify-center">
                    {move || if loading.get() {
                        view! { <i class="fas fa-spinner fa-spin mr-2"></i> }.into_view()
                    } else {
                        view! { <i class="fas fa-calculator mr-2"></i> }.into_view()
                    }}
                    {move || if loading.get() { t(lang.get(), "calculating") } else { t(lang.get(), "calculate_sofa") }}
                </button>
            </div>

            // Results (Top)
            <div class="min-h-[100px] mb-8">
                {move || {
                    if let Some(data) = result.get() {
                        let (bg_color, text_color) = if data.score <= 6 {
                            ("bg-green-500", "text-green-50")
                        } else if data.score <= 9 {
                            ("bg-yellow-500", "text-yellow-50")
                        } else if data.score <= 12 {
                            ("bg-orange-500", "text-orange-50")
                        } else {
                            ("bg-red-600", "text-red-50")
                        };

                        view! {
                            <div class=format!("{} {} rounded-xl shadow-lg transition-colors duration-700 animate-fade-in", bg_color, text_color)>
                                <div class="p-4 grid grid-cols-1 md:grid-cols-3 gap-4 items-center">
                                    <div class="text-center md:border-r border-white/20">
                                        <div class="text-xs uppercase opacity-80 mb-1">
                                            <i class="fas fa-calculator mr-1"></i>{t(lang.get(), "score")}
                                        </div>
                                        <div class="text-4xl font-bold">
                                            {data.score}<span class="text-2xl opacity-80">"/24"</span>
                                        </div>
                                    </div>
                                    <div class="text-center md:text-left">
                                        <div class="text-xs uppercase opacity-80 mb-1">
                                            <i class="fas fa-heartbeat mr-1"></i>{t(lang.get(), "severity")}
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

            // Form Sections - Organ Systems
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Respiration
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-blue-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-lungs text-blue-600 mr-2"></i>{move || t(lang.get(), "respiration")}
                    </h3>
                    <div>
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span class="flex items-center"><i class="fas fa-wind text-blue-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "pao2_fio2")}</span>
                            <span class="text-blue-600 font-bold">{move || pao2_fio2.get()}</span>
                        </label>
                        <input type="range" min="50" max="600" step="10"
                            prop:value=move || pao2_fio2.get()
                            on:input=move |ev| set_pao2_fio2.set(event_target_value(&ev).parse().unwrap_or(400))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"/>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "pao2_hint")}</p>
                    </div>
                </div>

                // Coagulation
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-red-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-tint text-red-600 mr-2"></i>{move || t(lang.get(), "coagulation")}
                    </h3>
                    <div>
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span class="flex items-center"><i class="fas fa-circle text-red-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "platelets")}</span>
                            <span class="text-red-600 font-bold">{move || platelets.get()}</span>
                        </label>
                        <input type="range" min="0" max="400" step="5"
                            prop:value=move || platelets.get()
                            on:input=move |ev| set_platelets.set(event_target_value(&ev).parse().unwrap_or(150))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-red-600"/>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "platelets_hint")}</p>
                    </div>
                </div>

                // Liver
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-amber-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-liver text-amber-600 mr-2"></i>{move || t(lang.get(), "liver")}
                    </h3>
                    <div>
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span class="flex items-center"><i class="fas fa-vial text-amber-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "bilirubin")}</span>
                            <span class="text-amber-600 font-bold">{move || format!("{:.1}", bilirubin.get())}</span>
                        </label>
                        <input type="range" min="0.1" max="20.0" step="0.1"
                            prop:value=move || bilirubin.get()
                            on:input=move |ev| set_bilirubin.set(event_target_value(&ev).parse().unwrap_or(1.0))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-amber-600"/>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "bilirubin_hint")}</p>
                    </div>
                </div>

                // Cardiovascular
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-pink-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-heart text-pink-600 mr-2"></i>{move || t(lang.get(), "cardiovascular")}
                    </h3>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1 flex items-center"><i class="fas fa-heart-pulse text-pink-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "hemodynamic_status")}</label>
                        <select
                            on:change=move |ev| set_cardiovascular.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-pink-500">
                            <option value="map_70_plus" selected>{move || t(lang.get(), "map_70_plus")}</option>
                            <option value="map_lt_70">{move || t(lang.get(), "map_lt_70")}</option>
                            <option value="dopa_lte5">{move || t(lang.get(), "dopa_lte5")}</option>
                            <option value="dopa_gt5">{move || t(lang.get(), "dopa_gt5")}</option>
                            <option value="dopa_gt15">{move || t(lang.get(), "dopa_gt15")}</option>
                        </select>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "vasopressor_hint")}</p>
                    </div>
                </div>

                // Central Nervous System
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-purple-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-brain text-purple-600 mr-2"></i>{move || t(lang.get(), "cns")}
                    </h3>
                    <div>
                        <label class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                            <span class="flex items-center"><i class="fas fa-brain text-purple-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "gcs_score")}</span>
                            <span class="text-purple-600 font-bold">{move || glasgow.get()}</span>
                        </label>
                        <input type="range" min="3" max="15" step="1"
                            prop:value=move || glasgow.get()
                            on:input=move |ev| set_glasgow.set(event_target_value(&ev).parse().unwrap_or(15))
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"/>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "gcs_hint")}</p>
                    </div>
                </div>

                // Renal
                <div class="bg-white rounded-xl shadow-md p-6 border-t-4 border-indigo-500">
                    <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center border-b pb-2">
                        <i class="fas fa-kidneys text-indigo-600 mr-2"></i>{move || t(lang.get(), "renal")}
                    </h3>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1 flex items-center"><i class="fas fa-filter text-indigo-500 mr-2 w-5 text-center"></i>{move || t(lang.get(), "creatinine_level")}</label>
                        <select
                            on:change=move |ev| set_renal.set(event_target_value(&ev))
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500">
                            <option value="cr_lt_1_2" selected>{move || t(lang.get(), "cr_lt_1_2")}</option>
                            <option value="cr_1_2_1_9">{move || t(lang.get(), "cr_1_2_1_9")}</option>
                            <option value="cr_2_0_3_4">{move || t(lang.get(), "cr_2_0_3_4")}</option>
                            <option value="cr_3_5_4_9">{move || t(lang.get(), "cr_3_5_4_9")}</option>
                            <option value="cr_gte_5">{move || t(lang.get(), "cr_gte_5")}</option>
                        </select>
                        <p class="text-xs text-gray-500 mt-2">{move || t(lang.get(), "renal_hint")}</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
