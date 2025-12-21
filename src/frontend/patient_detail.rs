use crate::frontend::i18n::{t, use_i18n};
use crate::models::apache::ApacheAssessment;
use crate::models::glasgow::GlasgowAssessment;
use crate::models::patient::Patient;
use crate::models::saps::SapsAssessment;
use crate::models::sofa::SofaAssessment;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HistoryResponse {
    pub glasgow: Vec<GlasgowAssessment>,
    pub apache: Vec<ApacheAssessment>,
    pub sofa: Vec<SofaAssessment>,
    pub saps: Vec<SapsAssessment>,
}

#[component]
pub fn PatientDetail() -> impl IntoView {
    let params = use_params_map();
    let lang = use_i18n();
    let id = move || params.get().get("id").cloned().unwrap_or_default();

    let (patient, set_patient) = create_signal(Option::<Patient>::None);
    let (history, set_history) = create_signal(HistoryResponse::default());
    let (loading, set_loading) = create_signal(true);

    // Fetch data
    create_effect(move |_| {
        let p_id = id();
        if !p_id.is_empty() {
            spawn_local(async move {
                // Fetch Patient
                let p_res = reqwasm::http::Request::get(&format!("/api/patients/{}", p_id))
                    .send()
                    .await;
                if let Ok(resp) = p_res {
                    if resp.ok() {
                        if let Ok(p) = resp.json::<Option<Patient>>().await {
                            set_patient.set(p);
                        }
                    }
                }

                // Fetch History
                let h_res = reqwasm::http::Request::get(&format!("/api/patients/{}/history", p_id))
                    .send()
                    .await;
                if let Ok(resp) = h_res {
                    if resp.ok() {
                        if let Ok(h) = resp.json::<HistoryResponse>().await {
                            set_history.set(h);
                        }
                    }
                }
                set_loading.set(false);
            });
        }
    });

    view! {
        <div class="max-w-7xl mx-auto p-6 space-y-8">
            <Show when=move || !loading.get() fallback=|| view! { <div class="text-center p-10"><i class="fas fa-spinner fa-spin text-4xl text-indigo-600"></i></div> }>
                {move || patient.get().map(|p| view! {
                    // Patient Header / Bio
                    <div class="bg-white rounded-2xl shadow-md p-6 border-l-8 border-indigo-600 flex justify-between items-start">
                        <div>
                            <div class="flex items-center gap-4 mb-2">
                                <h1 class="text-4xl font-bold text-gray-900">{p.first_name.clone()} {p.last_name.clone()}</h1>
                                <span class="bg-indigo-100 text-indigo-800 text-sm px-3 py-1 rounded-full font-semibold">
                                    "ID: " {id()}
                                </span>
                            </div>
                            <div class="flex gap-6 text-gray-600 mt-2">
                                <span class="flex items-center"><i class="fas fa-birthday-cake mr-2 text-indigo-400"></i> {p.date_of_birth.clone()}</span>
                                <span class="flex items-center"><i class="fas fa-venus-mars mr-2 text-indigo-400"></i> {p.gender.clone()}</span>
                                <span class="flex items-center"><i class="fas fa-bed mr-2 text-indigo-400"></i> "Bed 1"</span>
                            </div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm text-gray-500">{move || t(lang.get(), "days_in_hospital")}</div>
                             <div class="text-3xl font-black text-indigo-600">{p.days_in_hospital()}</div>
                        </div>
                    </div>

                    // Innovations: Visual Trends (Placeholder for now)
                    // We will implement actual charts here in the Innovation Phase
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                         // SOFA Trend
                        <div class="bg-white rounded-xl shadow-sm p-6">
                            <h3 class="font-bold text-gray-800 mb-4 flex items-center">
                                <i class="fas fa-chart-line mr-2 text-teal-500"></i> "SOFA Trend (Last 24h)"
                            </h3>
                            <div class="h-32 bg-gray-50 rounded-lg flex items-center justify-center text-gray-400">
                                "Visual Trends Chart (Coming Soon)"
                            </div>
                        </div>
                         // APACHE Trend
                        <div class="bg-white rounded-xl shadow-sm p-6">
                            <h3 class="font-bold text-gray-800 mb-4 flex items-center">
                                <i class="fas fa-chart-area mr-2 text-red-500"></i> "APACHE II Trend"
                            </h3>
                            <div class="h-32 bg-gray-50 rounded-lg flex items-center justify-center text-gray-400">
                                "Visual Trends Chart (Coming Soon)"
                            </div>
                        </div>
                    </div>

                    // Assessments History
                    <div class="bg-white rounded-xl shadow-md p-6">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold text-gray-800">{move || t(lang.get(), "history_assessments")}</h2>

                            // Buttons to Add New Assessments (Linked to Patient)
                            <div class="flex gap-2">
                                <a href=format!("/sofa?patient_id={}", id()) class="bg-teal-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-teal-700 transition">
                                    "+ SOFA"
                                </a>
                                <a href=format!("/apache?patient_id={}", id()) class="bg-red-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-red-700 transition">
                                    "+ APACHE"
                                </a>
                                <a href=format!("/glasgow?patient_id={}", id()) class="bg-purple-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-purple-700 transition">
                                    "+ Glasgow"
                                </a>
                                <a href=format!("/saps?patient_id={}", id()) class="bg-orange-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-orange-700 transition">
                                    "+ SAPS II"
                                </a>
                            </div>
                        </div>

                        // Tabs or List? Let's do a unified timeline for logic simplicity first, or sections.
                        // Sections are easier.
                        <div class="space-y-8">
                            // SOFA History
                            <section>
                                <h3 class="text-lg font-bold text-teal-800 border-b border-teal-100 pb-2 mb-4">"SOFA"</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Score</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Severity</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().sofa key=|a| a.assessed_at.clone() children=move |item| {
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm font-bold text-gray-900">{item.score}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.severity}</td>
                                                    </tr>
                                                }
                                            }/>
                                            <Show when=move || history.get().sofa.is_empty() fallback=|| view!{}>
                                                <tr><td colspan="3" class="px-6 py-4 text-center text-gray-400 italic">{move || t(lang.get(), "no_history")}</td></tr>
                                            </Show>
                                        </tbody>
                                    </table>
                                </div>
                            </section>

                            // APACHE History
                             <section>
                                <h3 class="text-lg font-bold text-red-800 border-b border-red-100 pb-2 mb-4">"APACHE II"</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Score</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Mortality</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().apache key=|a| a.assessed_at.clone() children=move |item| {
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm font-bold text-gray-900">{item.score}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{format!("{:.1}%", item.predicted_mortality)}</td>
                                                    </tr>
                                                }
                                            }/>
                                        </tbody>
                                    </table>
                                </div>
                            </section>
                        </div>
                    </div>
                })}
            </Show>
        </div>
    }
}
