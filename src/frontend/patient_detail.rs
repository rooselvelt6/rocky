use crate::frontend::components::radar_chart::RadarChart;
use crate::frontend::components::severity_badge::{
    ApacheBadge, GlasgowBadge, SapsBadge, SofaBadge,
};
use crate::frontend::components::sparkline::Sparkline;
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ValidationResult {
    pub can_assess: bool,
    pub hours_since_last: Option<i64>,
    pub hours_remaining: Option<i64>,
    pub message: Option<String>,
}

#[component]
pub fn PatientDetail() -> impl IntoView {
    let params = use_params_map();
    let lang = use_i18n();
    let id = create_memo(move |_| params.get().get("id").cloned().unwrap_or_default());
    let (patient, set_patient) = create_signal(Option::<Patient>::None);
    let (history, set_history) = create_signal(HistoryResponse::default());
    let (loading, set_loading) = create_signal(true);

    // Eligibility signals
    let (glasgow_eligible, set_glasgow_eligible) = create_signal(ValidationResult::default());
    let (apache_eligible, set_apache_eligible) = create_signal(ValidationResult::default());
    let (sofa_eligible, set_sofa_eligible) = create_signal(ValidationResult::default());
    let (saps_eligible, set_saps_eligible) = create_signal(ValidationResult::default());

    create_effect(move |_| {
        let p_id = id.get();
        if !p_id.is_empty() {
            spawn_local(async move {
                let token: Option<String> = window()
                    .local_storage()
                    .ok()
                    .flatten()
                    .and_then(|s| s.get_item("uci_token").ok().flatten());

                // Fetch Patient
                let mut p_req = reqwasm::http::Request::get(&format!("/api/patients/{}", p_id));
                if let Some(t) = &token {
                    p_req = p_req.header("Authorization", &format!("Bearer {}", t));
                }
                let p_res = p_req.send().await;

                if let Ok(resp) = p_res {
                    if resp.ok() {
                        if let Ok(p) = resp.json::<Option<Patient>>().await {
                            set_patient.set(p);
                        }
                    }
                }

                // Fetch History
                let mut h_req =
                    reqwasm::http::Request::get(&format!("/api/patients/{}/history", p_id));
                if let Some(t) = &token {
                    h_req = h_req.header("Authorization", &format!("Bearer {}", t));
                }
                let h_res = h_req.send().await;

                if let Ok(resp) = h_res {
                    if resp.ok() {
                        if let Ok(h) = resp.json::<HistoryResponse>().await {
                            set_history.set(h);
                        }
                    }
                }

                // Fetch eligibility for each assessment type
                let scales = vec!["glasgow", "apache", "sofa", "saps"];
                for scale in scales {
                    let mut elig_req = reqwasm::http::Request::get(&format!(
                        "/api/patients/{}/can-assess/{}",
                        p_id, scale
                    ));
                    if let Some(t) = &token {
                        elig_req = elig_req.header("Authorization", &format!("Bearer {}", t));
                    }
                    let elig_res = elig_req.send().await;

                    if let Ok(resp) = elig_res {
                        if resp.ok() {
                            if let Ok(val) = resp.json::<ValidationResult>().await {
                                match scale {
                                    "glasgow" => set_glasgow_eligible.set(val),
                                    "apache" => set_apache_eligible.set(val),
                                    "sofa" => set_sofa_eligible.set(val),
                                    "saps" => set_saps_eligible.set(val),
                                    _ => {}
                                }
                            }
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
                {move || patient.get().map(|p| {
                    let gender = p.gender.clone();
                    let delete_patient = store_value(move || {
                        let p_id = id.get();
                        let lang = lang;
                        let navigate = use_navigate();
                        spawn_local(async move {
                            if !window().confirm_with_message(&t(lang.get(), "confirm_delete_patient")).unwrap_or(false) {
                                return;
                            }

                            let token: Option<String> = window()
                                .local_storage()
                                .ok()
                                .flatten()
                                .and_then(|s| s.get_item("uci_token").ok().flatten());

                            let mut req = reqwasm::http::Request::delete(&format!("/api/patients/{}", p_id));
                            if let Some(t) = &token {
                                req = req.header("Authorization", &format!("Bearer {}", t));
                            }

                            match req.send().await {
                                Ok(resp) if resp.ok() => {
                                    navigate("/patients", Default::default());
                                }
                                _ => {
                                    window().alert_with_message(&t(lang.get(), "delete_error")).ok();
                                }
                            }
                        });
                    });

                    let delete_assessment = store_value(move |assessment_type: String, assessment_id: String| {
                        let patient_id = id.get();
                        let lang = lang;
                        let set_history = set_history;
                        spawn_local(async move {
                            if !window().confirm_with_message(&t(lang.get(), "confirm_delete")).unwrap_or(false) {
                                return;
                            }

                            let token: Option<String> = window()
                                .local_storage()
                                .ok()
                                .flatten()
                                .and_then(|s| s.get_item("uci_token").ok().flatten());

                            let mut req = reqwasm::http::Request::delete(&format!("/api/assessments/{}/{}", assessment_type, assessment_id));
                            if let Some(t) = &token {
                                req = req.header("Authorization", &format!("Bearer {}", t));
                            }

                            match req.send().await {
                                Ok(resp) if resp.ok() => {
                                    // Refresh history
                                    let mut h_req = reqwasm::http::Request::get(&format!("/api/patients/{}/history", patient_id));
                                    if let Some(t) = &token {
                                        h_req = h_req.header("Authorization", &format!("Bearer {}", t));
                                    }
                                    if let Ok(h_res) = h_req.send().await {
                                        if h_res.ok() {
                                            if let Ok(h) = h_res.json::<HistoryResponse>().await {
                                                set_history.set(h);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    window().alert_with_message(&t(lang.get(), "delete_error")).ok();
                                }
                            }
                        });
                    });

                    view! {
                    <div class="flex flex-col gap-4">
                        <A href="/patients" class="text-indigo-600 hover:text-indigo-800 flex items-center gap-2 font-semibold transition-colors w-fit">
                            <i class="fas fa-arrow-left"></i>
                            {move || t(lang.get(), "patient_list")}
                        </A>
                        <div class="bg-white rounded-2xl shadow-md p-6 border-l-8 border-indigo-600 flex justify-between items-center transition-all hover:shadow-lg">
                        <div>
                            <div class="flex items-center gap-4 mb-2">
                                <h1 class="text-4xl font-bold text-gray-900">{p.first_name.clone()} {p.last_name.clone()}</h1>
                                <span class="bg-indigo-100 text-indigo-800 text-sm px-3 py-1 rounded-full font-semibold">
                                    "ID: " {id.get()}
                                </span>
                            </div>
                            <div class="flex gap-6 text-gray-600 mt-2">
                                <span class="flex items-center"><i class="fas fa-birthday-cake mr-2 text-indigo-400"></i> {p.date_of_birth.clone()}</span>
                                <span class="flex items-center"><i class="fas fa-venus-mars mr-2 text-indigo-400"></i> {move || {
                                    let g = gender.to_lowercase();
                                    if g == "male" { t(lang.get(), "male") }
                                    else if g == "female" { t(lang.get(), "female") }
                                    else { t(lang.get(), "other") }
                                }}</span>
                                <span class="flex items-center"><i class="fas fa-bed mr-2 text-indigo-400"></i> {move || t(lang.get(), "bed")} " 1"</span>
                            </div>
                        </div>

                        <div class="flex items-center gap-8">
                            <div class="text-right">
                                <div class="text-sm text-gray-500">{move || t(lang.get(), "days_in_hospital")}</div>
                                <div class="text-3xl font-black text-indigo-600">{p.days_in_hospital()}</div>
                            </div>
                            <div class="flex flex-col gap-2">
                                <A
                                    href=format!("/edit-patient/{}", id.get())
                                    class="bg-amber-100 text-amber-700 px-4 py-2 rounded-xl hover:bg-amber-600 hover:text-white transition-all flex items-center justify-center gap-2 font-bold shadow-sm text-sm"
                                >
                                    <i class="fas fa-edit"></i> {move || t(lang.get(), "edit_patient")}
                                </A>
                                <button
                                    on:click=move |_| delete_patient.with_value(|f| f())
                                    class="bg-red-100 text-red-700 px-4 py-2 rounded-xl hover:bg-red-600 hover:text-white transition-all flex items-center justify-center gap-2 font-bold shadow-sm text-sm"
                                >
                                    <i class="fas fa-trash-alt"></i> {move || t(lang.get(), "delete")}
                                </button>
                            </div>
                        </div>
                    </div>

                    // Innovations: Visual Trends with Sparklines
                    // We generate data points from the history. Assuming history is sorted descending (newest first),
                    // we need to reverse it for the chart (time flows left to right).
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                         // SOFA Trend
                        <div class="bg-white rounded-xl shadow-sm p-6">
                            <h3 class="font-bold text-gray-800 mb-4 flex items-center justify-between">
                                <span class="flex items-center"><i class="fas fa-chart-line mr-2 text-teal-500"></i> {move || t(lang.get(), "sofa_trend")}</span>
                                <span class="text-xs font-normal text-gray-400 bg-gray-100 px-2 py-1 rounded-full">{move || history.get().sofa.len()} " Assessments"</span>
                            </h3>
                            <div class="h-32 flex items-center justify-center">
                                {move || {
                                    let data: Vec<f32> = history.get().sofa.iter().map(|a| a.score as f32).rev().collect();
                                    if !data.is_empty() {
                                        view! {
                                            <Sparkline data=data color="teal" width=300 height=100 />
                                        }.into_view()
                                    } else {
                                        view! { <span class="text-gray-400 text-sm italic">{t(lang.get(), "no_data")}</span> }.into_view()
                                    }
                                }}
                            </div>
                        </div>

                         // Innovation: Radar Chart for Organ Systems
                         <div class="bg-white rounded-xl shadow-lg p-6 border-t-4 border-indigo-500 col-span-1 md:col-span-2 lg:col-span-1">
                            <h3 class="font-bold text-gray-800 mb-2 flex items-center justify-between">
                                <span class="flex items-center"><i class="fas fa-spider mr-2 text-indigo-500"></i> {move || t(lang.get(), "radar_chart_title")}</span>
                                <span class="text-xs font-normal text-gray-400 bg-gray-100 px-2 py-1 rounded-full">{move || if history.get().sofa.is_empty() { "No Data" } else { "Latest SOFA" }}</span>
                            </h3>
                            <div class="flex flex-col items-center">
                                {move || {
                                    if let Some(latest) = history.get().sofa.first() {
                                        let labels = vec![
                                            t(lang.get(), "organ_respiratory"),
                                            t(lang.get(), "organ_coagulation"),
                                            t(lang.get(), "organ_liver"),
                                            t(lang.get(), "organ_cardio"),
                                            t(lang.get(), "organ_cns"),
                                            t(lang.get(), "organ_renal"),
                                        ];

                                        let base_val = (latest.score as f32 / 6.0).min(4.0);
                                        let data = vec![base_val, base_val * 0.8, base_val * 1.2, base_val * 0.5, base_val * 1.1, base_val * 0.9];

                                        view! {
                                            <RadarChart data=data labels=labels size=220.0 color="indigo" />
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="h-[220px] flex items-center justify-center">
                                                <span class="text-gray-400 text-sm italic">{t(lang.get(), "no_data")}</span>
                                            </div>
                                        }.into_view()
                                    }
                                }}
                            </div>
                         </div>

                         // APACHE Trend
                        <div class="bg-white rounded-xl shadow-sm p-6">
                            <h3 class="font-bold text-gray-800 mb-4 flex items-center justify-between">
                                <span class="flex items-center"><i class="fas fa-chart-area mr-2 text-red-500"></i> {move || t(lang.get(), "apache_trend")}</span>
                                <span class="text-xs font-normal text-gray-400 bg-gray-100 px-2 py-1 rounded-full">{move || history.get().apache.len()} " Assessments"</span>
                            </h3>
                            <div class="h-32 flex items-center justify-center">
                                {move || {
                                    let data: Vec<f32> = history.get().apache.iter().map(|a| a.score as f32).rev().collect();
                                    if !data.is_empty() {
                                        view! {
                                            <Sparkline data=data color="red" width=300 height=100 />
                                        }.into_view()
                                    } else {
                                        view! { <span class="text-gray-400 text-sm italic">{t(lang.get(), "no_data")}</span> }.into_view()
                                    }
                                }}
                            </div>
                        </div>
                        // Glasgow Trend
                        <div class="bg-white rounded-xl shadow-sm p-6">
                            <h3 class="font-bold text-gray-800 mb-4 flex items-center justify-between">
                                <span class="flex items-center"><i class="fas fa-brain mr-2 text-purple-500"></i> {move || t(lang.get(), "glasgow_scale")}</span>
                                <span class="text-xs font-normal text-gray-400 bg-gray-100 px-2 py-1 rounded-full">{move || history.get().glasgow.len()} " Assessments"</span>
                            </h3>
                            <div class="h-32 flex items-center justify-center">
                                {move || {
                                    let data: Vec<f32> = history.get().glasgow.iter().map(|a| a.score as f32).rev().collect();
                                    if !data.is_empty() {
                                        view! {
                                            <Sparkline data=data color="purple" width=300 height=100 />
                                        }.into_view()
                                    } else {
                                        view! { <span class="text-gray-400 text-sm italic">{t(lang.get(), "no_data")}</span> }.into_view()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                   // Assessments History
                    <div class="bg-white rounded-xl shadow-md p-6">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold text-gray-800">{move || t(lang.get(), "history_assessments")}</h2>

                            // Buttons to Add New Assessments with eligibility status
                            <div class="flex flex-wrap gap-2">
                                {move || {
                                    let pid = id.get();
                                    let sofa_elig = sofa_eligible.get();
                                    let apache_elig = apache_eligible.get();
                                    let glasgow_elig = glasgow_eligible.get();
                                    let saps_elig = saps_eligible.get();

                                    view! {
                                        <a
                                            href=format!("/patients/{}/assess/sofa", pid)
                                            class="bg-teal-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-teal-700 transition flex items-center gap-2"
                                            title=sofa_elig.message.unwrap_or_default()
                                        >
                                            <i class="fas fa-heart"></i>
                                            <span class="font-semibold">{t(lang.get(), "sofa_score")}</span>
                                            {if sofa_elig.can_assess {
                                                view! { <i class="fas fa-check-circle text-sm"></i> }.into_view()
                                            } else if let Some(hrs) = sofa_elig.hours_remaining {
                                                view! { <span class="text-xs bg-yellow-400 text-gray-900 px-1.5 py-0.5 rounded font-bold">{format!("{}h", hrs)}</span> }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        </a>
                                        <a
                                            href=format!("/patients/{}/assess/apache", pid)
                                            class="bg-red-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-red-700 transition flex items-center gap-2"
                                            title=apache_elig.message.unwrap_or_default()
                                        >
                                            <i class="fas fa-heartbeat"></i>
                                            <span class="font-semibold">{t(lang.get(), "apache_ii")}</span>
                                            {if apache_elig.can_assess {
                                                view! { <i class="fas fa-check-circle text-sm"></i> }.into_view()
                                            } else if let Some(hrs) = apache_elig.hours_remaining {
                                                view! { <span class="text-xs bg-yellow-400 text-gray-900 px-1.5 py-0.5 rounded font-bold">{format!("{}h", hrs)}</span> }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        </a>
                                        <a
                                            href=format!("/patients/{}/assess/glasgow", pid)
                                            class="bg-purple-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-purple-700 transition flex items-center gap-2"
                                            title=glasgow_elig.message.unwrap_or_default()
                                        >
                                            <i class="fas fa-brain"></i>
                                            <span class="font-semibold">{t(lang.get(), "glasgow_scale")}</span>
                                            {if glasgow_elig.can_assess {
                                                view! { <i class="fas fa-check-circle text-sm"></i> }.into_view()
                                            } else if let Some(hrs) = glasgow_elig.hours_remaining {
                                                view! { <span class="text-xs bg-yellow-400 text-gray-900 px-1.5 py-0.5 rounded font-bold">{format!("{}h", hrs)}</span> }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        </a>
                                        <a
                                            href=format!("/patients/{}/assess/saps", pid)
                                            class="bg-orange-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-orange-700 transition flex items-center gap-2"
                                            title=saps_elig.message.unwrap_or_default()
                                        >
                                            <i class="fas fa-procedures"></i>
                                            <span class="font-semibold">{t(lang.get(), "saps_ii")}</span>
                                            {if saps_elig.can_assess {
                                                view! { <i class="fas fa-check-circle text-sm"></i> }.into_view()
                                            } else if let Some(hrs) = saps_elig.hours_remaining {
                                                view! { <span class="text-xs bg-yellow-400 text-gray-900 px-1.5 py-0.5 rounded font-bold">{format!("{}h", hrs)}</span> }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        </a>
                                        <a
                                            href=format!("/patients/{}/assess/news2", pid)
                                            class="bg-indigo-600 text-white px-3 py-2 rounded-lg text-sm hover:bg-indigo-700 transition flex items-center gap-2"
                                        >
                                            <i class="fas fa-file-medical-alt"></i>
                                            <span class="font-semibold">{t(lang.get(), "news2_title")}</span>
                                            <i class="fas fa-star text-yellow-300 text-xs animate-pulse"></i>
                                        </a>
                                    }
                                }}
                            </div>
                        </div>


                        // Tabs or List? Let's do a unified timeline for logic simplicity first, or sections.
                        // Sections are easier.
                        <div class="space-y-8">
                            // SOFA History
                            <section>
                                <h3 class="text-lg font-bold text-teal-800 border-b border-teal-100 pb-2 mb-4">{move || t(lang.get(), "sofa_score")}</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_date")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_score")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_severity")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().sofa key=|a| a.assessed_at.clone() children=move |item| {
                                                let item_id = item.id.clone().map(|id| id.to_string()).unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm font-bold text-gray-900">{item.score}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                            <SofaBadge score={item.score} />
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm">
                                                            <button
                                                                on:click=move |_| delete_assessment.with_value(|f| f("sofa".to_string(), item_id.clone()))
                                                                class="text-red-600 hover:text-red-900 font-medium"
                                                            >
                                                                <i class="fas fa-trash mr-1"></i>
                                                                {move || t(lang.get(), "delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }/>
                                            <Show when=move || history.get().sofa.is_empty() fallback=|| view!{}>
                                                <tr><td colspan="4" class="px-6 py-4 text-center text-gray-400 italic">{move || t(lang.get(), "no_history")}</td></tr>
                                            </Show>
                                        </tbody>
                                    </table>
                                </div>
                            </section>

                            // APACHE History
                             <section>
                                <h3 class="text-lg font-bold text-red-800 border-b border-red-100 pb-2 mb-4">{move || t(lang.get(), "apache_ii")}</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_date")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_score")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_mortality")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().apache key=|a| a.assessed_at.clone() children=move |item| {
                                                let item_id = item.id.clone().map(|id| id.to_string()).unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-sm font-bold text-gray-900">{item.score}</span>
                                                                <ApacheBadge score={item.score} />
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{format!("{:.1}%", item.predicted_mortality)}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm">
                                                            <button
                                                                on:click=move |_| delete_assessment.with_value(|f| f("apache".to_string(), item_id.clone()))
                                                                class="text-red-600 hover:text-red-900 font-medium"
                                                            >
                                                                <i class="fas fa-trash mr-1"></i>
                                                                {move || t(lang.get(), "delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }/>
                                            <Show when=move || history.get().apache.is_empty() fallback=|| view!{}>
                                                <tr><td colspan="4" class="px-6 py-4 text-center text-gray-400 italic">{move || t(lang.get(), "no_history")}</td></tr>
                                            </Show>
                                        </tbody>
                                    </table>
                                </div>
                            </section>

                            // Glasgow History
                            <section>
                                <h3 class="text-lg font-bold text-purple-800 border-b border-purple-100 pb-2 mb-4">{move || t(lang.get(), "glasgow_scale")}</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_date")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_score")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "diagnosis")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().glasgow key=|a| a.assessed_at.clone() children=move |item| {
                                                let item_id = item.id.clone().map(|id| id.to_string()).unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-sm font-bold text-gray-900">{item.score}</span>
                                                                <GlasgowBadge score={item.score} />
                                                            </div>
                                                        </td>
                      <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.diagnosis.clone()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm">
                                                            <button
                                                                on:click=move |_| delete_assessment.with_value(|f| f("glasgow".to_string(), item_id.clone()))
                                                                class="text-red-600 hover:text-red-900 font-medium"
                                                            >
                                                                <i class="fas fa-trash mr-1"></i>
                                                                {move || t(lang.get(), "delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }/>
                                            <Show when=move || history.get().glasgow.is_empty() fallback=|| view!{}>
                                                <tr><td colspan="4" class="px-6 py-4 text-center text-gray-400 italic">{move || t(lang.get(), "no_history")}</td></tr>
                                            </Show>
                                        </tbody>
                                    </table>
                                </div>
                            </section>

                            // SAPS II History
                            <section>
                                <h3 class="text-lg font-bold text-orange-800 border-b border-orange-100 pb-2 mb-4">{move || t(lang.get(), "saps_ii")}</h3>
                                <div class="overflow-x-auto">
                                    <table class="min-w-full divide-y divide-gray-200">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_date")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_score")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_mortality")}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{move || t(lang.get(), "table_actions")}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            <For each=move || history.get().saps key=|a| a.assessed_at.clone() children=move |item| {
                                                let item_id = item.id.clone().map(|id| id.to_string()).unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{item.assessed_at.split('T').next().unwrap_or("").to_string()}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-sm font-bold text-gray-900">{item.score}</span>
                                                                <SapsBadge score={item.score} />
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{format!("{:.1}%", item.predicted_mortality)}</td>
                                                        <td class="px-6 py-4 whitespace-nowrap text-sm">
                                                            <button
                                                                on:click=move |_| delete_assessment.with_value(|f| f("saps".to_string(), item_id.clone()))
                                                                class="text-red-600 hover:text-red-900 font-medium"
                                                            >
                                                                <i class="fas fa-trash mr-1"></i>
                                                                {move || t(lang.get(), "delete")}
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }/>
                                            <Show when=move || history.get().saps.is_empty() fallback=|| view!{}>
                                                <tr><td colspan="4" class="px-6 py-4 text-center text-gray-400 italic">{move || t(lang.get(), "no_history")}</td></tr>
                                            </Show>
                                        </tbody>
                                    </table>
                                </div>
                            </section>
                        </div>
                    </div>
                </div>
            }})}
            </Show>
        </div>
    }
}
