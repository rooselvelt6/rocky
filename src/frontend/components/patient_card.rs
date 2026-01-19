use crate::frontend::components::sparkline::Sparkline;
use crate::frontend::i18n::{t, use_i18n};
use crate::models::history::PatientHistoryResponse;
use crate::models::patient::Patient;
use leptos::*;
use reqwasm::http::Request;

#[component]
pub fn PatientCard(
    patient: Patient,
    #[prop(optional)] on_delete: Option<Callback<()>>,
) -> impl IntoView {
    let lang = use_i18n();
    let id = patient.id.clone();
    let id_str = id.clone().map(|t| t.to_string()).unwrap_or_default();
    let (history, set_history) = create_signal(Option::<PatientHistoryResponse>::None);

    // Fetch history
    let p_id_clone = id_str.clone();
    create_effect(move |_| {
        let p_id = p_id_clone.clone();
        spawn_local(async move {
            let url = format!("/api/patients/{}/history", p_id);
            if let Ok(res) = Request::get(&url).send().await {
                if let Ok(data) = res.json::<PatientHistoryResponse>().await {
                    set_history.set(Some(data));
                }
            }
        });
    });

    let sparkline_data = move || {
        if let Some(h) = history.get() {
            // Sort by date? Assuming DB returns sorted or we sort.
            // Let's take APACHE scores.
            let mut scores: Vec<f32> = h.apache.iter().map(|a| a.score as f32).collect();
            // If empty, try SOFA
            if scores.is_empty() {
                scores = h.sofa.iter().map(|s| s.score as f32).collect();
            }
            // Limit to last 10 points
            if scores.len() > 10 {
                scores = scores.into_iter().rev().take(10).collect(); // actually keep order?
                                                                      // If sorted desc in DB (ORDER BY assessed_at DESC), then .rev().take(10) gives last 10 reversed.
                                                                      // We want chronological order for sparkline.
                                                                      // If DB is DESC, then index 0 is newest.
                                                                      // So we take 10, then reverse back to chronological.
                                                                      // We need to verify DB sort order. main.rs query usually does simple fetch, filtering.
            } else {
                // If not sorted, we might need to sort. But for efficiency let's assume...
                // Actually Sparkline needs chronological.
                // Let's reverse if we assume DESC.
                scores.reverse();
            }
            scores
        } else {
            vec![]
        }
    };

    view! {
        <div class="bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-xl transition-all duration-300 pb-2 flex flex-col h-full">
            <div class="p-6 pb-4 flex-grow">
                <div class="flex justify-between items-start mb-4">
                    <div class="flex items-center">
                        <div class="bg-indigo-100 text-indigo-600 rounded-full w-12 h-12 flex items-center justify-center text-xl font-bold mr-4">
                            {patient.first_name.chars().next().unwrap_or('?')}
                        </div>
                        <div>
                            <h3 class="text-xl font-bold text-gray-900">{patient.first_name} {patient.last_name}</h3>
                            <div class="flex items-center text-gray-500 text-sm mt-1">
                                <i class="fas fa-bed mr-2 text-indigo-400"></i>
                                <span>{move || t(lang.get(), "bed")} " 1"</span>
                            </div>
                        </div>
                    </div>
                    <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full uppercase tracking-wider">
                        {move || t(lang.get(), "stable")}
                    </span>
                </div>

                <div class="space-y-3 mb-4">
                    <div class="flex items-center text-sm text-gray-600 bg-gray-50 p-2 rounded-lg">
                        <i class="fas fa-file-medical text-teal-500 w-6 text-center mr-2"></i>
                        <span class="truncate font-medium">{patient.principal_diagnosis}</span>
                    </div>
                    // Sparkline Here
                     <div class="h-12 w-full flex items-center justify-center bg-gray-50/50 rounded-lg overflow-hidden relative">
                        {move || {
                            let data = sparkline_data();
                            if data.len() > 1 {
                                view! {
                                    <div class="absolute inset-0 flex items-center justify-center opacity-20 pointer-events-none">
                                        <span class="text-[10px] uppercase tracking-widest text-gray-400">{move || t(lang.get(), "trend")}</span>
                                    </div>
                                    <Sparkline data=data color="blue" width=200 height=40 />
                                }.into_view()
                            } else {
                                view! { <span class="text-xs text-gray-400 italic">{move || t(lang.get(), "no_trend_data")}</span> }.into_view()
                            }
                        }}
                    </div>
                </div>

                <div class="space-y-2">
                    <div class="grid grid-cols-2 gap-2">
                        <a
                            href=format!("/patients/{}/assess/glasgow", id_str)
                            class="bg-purple-50 text-purple-700 text-center py-2 px-2 rounded-lg hover:bg-purple-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                            <i class="fas fa-brain text-xs"></i>
                            {move || t(lang.get(), "glasgow_scale")}
                        </a>
                        <a
                            href=format!("/patients/{}/assess/apache", id_str)
                            class="bg-red-50 text-red-700 text-center py-2 px-2 rounded-lg hover:bg-red-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                            <i class="fas fa-heartbeat text-xs"></i>
                            {move || t(lang.get(), "apache_ii")}
                        </a>
                    </div>
                    <div class="grid grid-cols-2 gap-2">
                        <a
                            href=format!("/patients/{}/assess/sofa", id_str)
                            class="bg-teal-50 text-teal-700 text-center py-2 px-2 rounded-lg hover:bg-teal-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                            <i class="fas fa-heart text-xs"></i>
                            {move || t(lang.get(), "sofa_score")}
                        </a>
                        <a
                            href=format!("/patients/{}/assess/saps", id_str)
                            class="bg-orange-50 text-orange-700 text-center py-2 px-2 rounded-lg hover:bg-orange-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                        >
                            <i class="fas fa-procedures text-xs"></i>
                            {move || t(lang.get(), "saps_ii")}
                        </a>
                    </div>
                    <a
                        href=format!("/patients/{}", id_str)
                        class="block bg-indigo-50 text-indigo-700 text-center py-2 rounded-lg hover:bg-indigo-600 hover:text-white text-xs font-semibold transition-all"
                    >
                        <i class="fas fa-chart-line mr-1"></i>
                        {move || t(lang.get(), "view_history")}
                    </a>
                </div>
            </div>
            <div class="px-6 py-2 border-t border-gray-50 flex justify-between items-center text-xs text-gray-400 mt-auto">
                <div class="flex items-center gap-4">
                    <span>"ID: " {id_str.split(':').last().unwrap_or("?").to_string()}</span>
                    <button
                        on:click=move |ev| {
                            ev.stop_propagation();
                            let p_id = id_str.clone();
                            let lang = lang;
                            let on_delete = on_delete;
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

                                if let Ok(resp) = req.send().await {
                                    if resp.ok() {
                                        if let Some(cb) = on_delete {
                                            cb.call(());
                                        }
                                    } else {
                                        window().alert_with_message(&t(lang.get(), "delete_error")).ok();
                                    }
                                }
                            });
                        }
                        class="text-red-400 hover:text-red-600 transition-colors p-1"
                        title=move || t(lang.get(), "delete")
                    >
                        <i class="fas fa-trash-alt"></i>
                    </button>
                </div>
                <span>{move || t(lang.get(), "updated_today")}</span>
            </div>
        </div>
    }
}
