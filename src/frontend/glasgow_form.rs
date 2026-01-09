use crate::frontend::i18n::{t, use_i18n};
use crate::uci::scale::glasgow::{GlasgowRequest, GlasgowResponse};
use leptos::*;
use leptos_router::use_query_map;
use reqwasm::http::Request;

/// Glasgow Coma Scale form component - Compact, Responsive & Smooth
#[component]
pub fn GlasgowForm() -> impl IntoView {
    let lang = use_i18n();
    let query = use_query_map();
    let patient_id = move || query.get().get("patient_id").cloned();

    // Reactive signals for form inputs
    let (eye_value, set_eye_value) = create_signal(4u8);
    let (verbal_value, set_verbal_value) = create_signal(5u8);
    let (motor_value, set_motor_value) = create_signal(6u8);

    // Resource that triggers when any input changes
    let glasgow_resource = create_resource(
        move || (eye_value.get(), verbal_value.get(), motor_value.get()),
        move |(eye, verbal, motor)| async move {
            let token: Option<String> = window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("uci_token").ok().flatten());

            let request = GlasgowRequest {
                eye,
                verbal,
                motor,
                patient_id: patient_id(),
            };

            // Call the API
            let mut req = Request::post("/api/glasgow").header("Content-Type", "application/json");

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
                        resp.json::<GlasgowResponse>().await.ok()
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        },
    );

    view! {
        <div class="w-full max-w-6xl mx-auto px-4">
            // Header with Load Last
            <div class="flex flex-col md:flex-row justify-between items-center mb-6 gap-4">
                <div class="text-center md:text-left">
                    <h2 class="text-2xl md:text-3xl font-bold bg-gradient-to-r from-purple-600 to-blue-500 bg-clip-text text-transparent">
                        <i class="fas fa-brain mr-2"></i>
                        {move || t(lang.get(), "glasgow_scale")}
                    </h2>
                </div>
                 <button
                    on:click=move |_| {
                        if let Some(id) = patient_id() {
                            spawn_local(async move {
                                let token: Option<String> = window().local_storage().ok().flatten()
                                    .and_then(|s| s.get_item("uci_token").ok().flatten());

                                let url = format!("/api/patients/{}/history", id);
                                let mut req = Request::get(&url);
                                if let Some(t) = token {
                                    req = req.header("Authorization", &format!("Bearer {}", t));
                                }

                                if let Ok(res) = req.send().await {
                                     #[derive(serde::Deserialize)]
                                     struct PartialHistory {
                                         glasgow: Vec<crate::models::glasgow::GlasgowAssessment>,
                                     }

                                     if let Ok(history) = res.json::<PartialHistory>().await {
                                         if let Some(last) = history.glasgow.first() {
                                             set_eye_value.set(last.eye_response);
                                             set_verbal_value.set(last.verbal_response);
                                             set_motor_value.set(last.motor_response);
                                         }
                                     }
                                }
                            });
                        }
                    }
                    class="px-4 py-2 bg-indigo-50 text-indigo-700 hover:bg-indigo-100 rounded-lg text-sm font-semibold transition-colors flex items-center shadow-sm border border-indigo-100"
                    title="Load previous assessment">
                    <i class="fas fa-clock-rotate-left mr-2"></i>
                    {move || t(lang.get(), "load_last")}
                </button>
            </div>

            // Results - Fixed height with smooth transitions
            <div class="min-h-[80px] mb-4">
                <Transition fallback=move || view! {
                    <div class="flex justify-center items-center h-[80px]">
                        <div class="animate-spin rounded-full h-8 w-8 border-4 border-purple-500 border-t-transparent"></div>
                    </div>
                }>
                    {move || {
                        glasgow_resource.get().flatten().map(|data| {
                            let (bg_color, text_color) = if data.score >= 13 {
                                ("bg-green-500", "text-green-50")
                            } else if data.score >= 9 {
                                ("bg-yellow-500", "text-yellow-50")
                            } else {
                                ("bg-red-500", "text-red-50")
                            };

                            view! {
                                <div class=format!("{} {} rounded-xl shadow-lg transition-colors duration-700 ease-in-out", bg_color, text_color)>
                                    <div class="p-4 grid grid-cols-1 md:grid-cols-4 gap-4 items-center">
                                        // Score Box
                                        <div class="text-center md:border-r border-white/20">
                                            <div class="text-xs uppercase opacity-80 mb-1 transition-opacity duration-500">
                                                <i class="fas fa-calculator mr-1"></i>{t(lang.get(), "score")}
                                            </div>
                                            <div class="text-4xl font-bold transition-all duration-700 ease-in-out transform">
                                                {data.score}<span class="text-2xl opacity-80 transition-opacity duration-700">"/15"</span>
                                            </div>
                                        </div>

                                        // Diagnosis
                                        <div class="md:col-span-2 text-center md:text-left">
                                            <div class="text-xs uppercase opacity-80 mb-1 transition-opacity duration-500">
                                                <i class="fas fa-stethoscope mr-1"></i>{t(lang.get(), "diagnosis")}
                                            </div>
                                            <div class="font-semibold text-sm transition-all duration-700 ease-in-out">{data.diagnosis}</div>
                                        </div>

                                        // Recommendation
                                        <div class="text-center md:text-left">
                                            <div class="text-xs uppercase opacity-80 mb-1 transition-opacity duration-500">
                                                <i class="fas fa-lightbulb mr-1"></i>{t(lang.get(), "action")}
                                            </div>
                                            <div class="font-semibold text-sm transition-all duration-700 ease-in-out">{data.recommendation}</div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                    }}
                </Transition>
            </div>

            // Chaining: Suggest next assessments
            {move || {
                if glasgow_resource.get().flatten().is_some() {
                    if let Some(pid) = patient_id() {
                        view! {
                            <div class="mb-8 p-4 bg-gradient-to-r from-indigo-50 to-blue-50 border border-indigo-100 rounded-xl animate-fade-in shadow-sm">
                                <h4 class="text-sm font-bold text-indigo-800 mb-3 flex items-center">
                                    <i class="fas fa-forward mr-2"></i>{t(lang.get(), "continue_assessment")}
                                </h4>
                                <div class="flex flex-wrap gap-3">
                                     <a href=format!("/apache?patient_id={}", pid)
                                       class="flex items-center px-4 py-2 bg-white text-indigo-600 border border-indigo-200 rounded-lg hover:bg-indigo-600 hover:text-white hover:border-indigo-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                       <i class="fas fa-chart-bar mr-2"></i>"APACHE II"
                                    </a>
                                    <a href=format!("/sofa?patient_id={}", pid)
                                       class="flex items-center px-4 py-2 bg-white text-teal-600 border border-teal-200 rounded-lg hover:bg-teal-600 hover:text-white hover:border-teal-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                       <i class="fas fa-procedures mr-2"></i>"SOFA"
                                    </a>
                                    <a href=format!("/saps?patient_id={}", pid)
                                       class="flex items-center px-4 py-2 bg-white text-orange-600 border border-orange-200 rounded-lg hover:bg-orange-600 hover:text-white hover:border-orange-600 transition-colors duration-200 shadow-sm text-sm font-bold">
                                       <i class="fas fa-notes-medical mr-2"></i>"SAPS II"
                                    </a>
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

            // Save Confirmation Message


            // Compact Selection Grid - Smooth transitions
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                // Eye Response
                <div class="bg-white rounded-xl shadow p-4 transition-all duration-300 hover:shadow-lg">
                    <h3 class="text-sm font-bold text-gray-800 mb-3 flex items-center">
                        <i class="fas fa-eye text-purple-600 mr-2"></i>{move || t(lang.get(), "eye_response")}
                    </h3>
                    <div class="grid grid-cols-2 gap-2">
                        <button
                            on:click=move |_| set_eye_value.set(4)
                            class=move || if eye_value.get() == 4 {
                                "p-3 text-xs rounded-lg border-2 border-green-500 bg-green-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-green-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-green-600 mb-1"><i class="fas fa-check-circle mr-1"></i>"4"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "spontaneous")}</div>
                        </button>

                        <button
                            on:click=move |_| set_eye_value.set(3)
                            class=move || if eye_value.get() == 3 {
                                "p-3 text-xs rounded-lg border-2 border-blue-500 bg-blue-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-blue-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-blue-600 mb-1"><i class="fas fa-volume-up mr-1"></i>"3"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "to_voice")}</div>
                        </button>

                        <button
                            on:click=move |_| set_eye_value.set(2)
                            class=move || if eye_value.get() == 2 {
                                "p-3 text-xs rounded-lg border-2 border-orange-500 bg-orange-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-orange-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-orange-600 mb-1"><i class="fas fa-hand-point-up mr-1"></i>"2"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "to_pain")}</div>
                        </button>

                        <button
                            on:click=move |_| set_eye_value.set(1)
                            class=move || if eye_value.get() == 1 {
                                "p-3 text-xs rounded-lg border-2 border-red-500 bg-red-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-red-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-red-600 mb-1"><i class="fas fa-times-circle mr-1"></i>"1"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "none")}</div>
                        </button>
                    </div>
                </div>

                // Verbal Response
                <div class="bg-white rounded-xl shadow p-4 transition-all duration-300 hover:shadow-lg">
                    <h3 class="text-sm font-bold text-gray-800 mb-3 flex items-center">
                        <i class="fas fa-comments text-purple-600 mr-2"></i>{move || t(lang.get(), "verbal_response")}
                    </h3>
                    <div class="grid grid-cols-2 gap-2">
                        <button
                            on:click=move |_| set_verbal_value.set(5)
                            class=move || if verbal_value.get() == 5 {
                                "p-3 text-xs rounded-lg border-2 border-green-500 bg-green-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-green-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-green-600 mb-1"><i class="fas fa-star mr-1"></i>"5"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "oriented")}</div>
                        </button>

                        <button
                            on:click=move |_| set_verbal_value.set(4)
                            class=move || if verbal_value.get() == 4 {
                                "p-3 text-xs rounded-lg border-2 border-blue-500 bg-blue-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-blue-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-blue-600 mb-1"><i class="fas fa-question-circle mr-1"></i>"4"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "confused")}</div>
                        </button>

                        <button
                            on:click=move |_| set_verbal_value.set(3)
                            class=move || if verbal_value.get() == 3 {
                                "p-3 text-xs rounded-lg border-2 border-yellow-500 bg-yellow-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-yellow-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-yellow-600 mb-1"><i class="fas fa-font mr-1"></i>"3"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "words")}</div>
                        </button>

                        <button
                            on:click=move |_| set_verbal_value.set(2)
                            class=move || if verbal_value.get() == 2 {
                                "p-3 text-xs rounded-lg border-2 border-orange-500 bg-orange-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-orange-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-orange-600 mb-1"><i class="fas fa-music mr-1"></i>"2"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "sounds")}</div>
                        </button>

                        <button
                            on:click=move |_| set_verbal_value.set(1)
                            class=move || if verbal_value.get() == 1 {
                                "p-3 text-xs rounded-lg border-2 border-red-500 bg-red-50 font-semibold col-span-2 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-red-400 col-span-2 transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-red-600 mb-1"><i class="fas fa-volume-mute mr-1"></i>"1"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "none")}</div>
                        </button>
                    </div>
                </div>

                // Motor Response
                <div class="bg-white rounded-xl shadow p-4 transition-all duration-300 hover:shadow-lg">
                    <h3 class="text-sm font-bold text-gray-800 mb-3 flex items-center">
                        <i class="fas fa-hand-rock text-purple-600 mr-2"></i>{move || t(lang.get(), "motor_response")}
                    </h3>
                    <div class="grid grid-cols-2 gap-2">
                        <button
                            on:click=move |_| set_motor_value.set(6)
                            class=move || if motor_value.get() == 6 {
                                "p-3 text-xs rounded-lg border-2 border-green-500 bg-green-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-green-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-green-600 mb-1"><i class="fas fa-thumbs-up mr-1"></i>"6"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "obeys")}</div>
                        </button>

                        <button
                            on:click=move |_| set_motor_value.set(5)
                            class=move || if motor_value.get() == 5 {
                                "p-3 text-xs rounded-lg border-2 border-blue-500 bg-blue-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-blue-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-blue-600 mb-1"><i class="fas fa-crosshairs mr-1"></i>"5"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "localizes")}</div>
                        </button>

                        <button
                            on:click=move |_| set_motor_value.set(4)
                            class=move || if motor_value.get() == 4 {
                                "p-3 text-xs rounded-lg border-2 border-cyan-500 bg-cyan-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-cyan-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-cyan-600 mb-1"><i class="fas fa-hand-paper mr-1"></i>"4"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "withdraws")}</div>
                        </button>

                        <button
                            on:click=move |_| set_motor_value.set(3)
                            class=move || if motor_value.get() == 3 {
                                "p-3 text-xs rounded-lg border-2 border-yellow-500 bg-yellow-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-yellow-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-yellow-600 mb-1"><i class="fas fa-compress-arrows-alt mr-1"></i>"3"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "flexion")}</div>
                        </button>

                        <button
                            on:click=move |_| set_motor_value.set(2)
                            class=move || if motor_value.get() == 2 {
                                "p-3 text-xs rounded-lg border-2 border-orange-500 bg-orange-50 font-semibold transform scale-100 transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-orange-400 hover:shadow transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-orange-600 mb-1"><i class="fas fa-expand-arrows-alt mr-1"></i>"2"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "extension")}</div>
                        </button>

                        <button
                            on:click=move |_| set_motor_value.set(1)
                            class=move || if motor_value.get() == 1 {
                                "p-3 text-xs rounded-lg border-2 border-red-500 bg-red-50 font-semibold transition-all duration-200 shadow-md"
                            } else {
                                "p-3 text-xs rounded-lg border border-gray-200 hover:border-red-400 transition-all duration-200"
                            }
                        >
                            <div class="font-bold text-red-600 mb-1"><i class="fas fa-ban mr-1"></i>"1"</div>
                            <div class="text-gray-700">{move || t(lang.get(), "none")}</div>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
