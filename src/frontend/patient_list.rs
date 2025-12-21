use crate::frontend::i18n::{t, use_i18n};
use crate::models::patient::Patient;
use leptos::*;

#[component]
pub fn PatientList() -> impl IntoView {
    let lang = use_i18n();
    let (patients, set_patients) = create_signal(Vec::<Patient>::new());

    // Fetch patients on mount
    create_effect(move |_| {
        spawn_local(async move {
            let res = reqwasm::http::Request::get("/api/patients").send().await;

            if let Ok(resp) = res {
                if resp.ok() {
                    let text = resp.text().await.unwrap_or_default();
                    leptos::logging::log!("Response: {}", text);
                    match serde_json::from_str::<Vec<Patient>>(&text) {
                        Ok(list) => set_patients.set(list),
                        Err(e) => leptos::logging::error!("Failed to parse patients: {}", e),
                    }
                } else {
                    leptos::logging::error!(
                        "Failed to fetch patients: {} {}",
                        resp.status(),
                        resp.status_text()
                    );
                }
            }
        });
    });

    view! {
        <div class="p-6 max-w-7xl mx-auto">
            <div class="flex justify-between items-center mb-8">
                <div>
                    <h2 class="text-3xl font-bold text-gray-800">{move || t(lang.get(), "patient_list")}</h2>
                    <p class="text-gray-500 mt-1">"Overview of all active patients"</p>
                </div>
                <a href="/register" class="bg-indigo-600 text-white px-6 py-3 rounded-xl hover:bg-indigo-700 shadow-md flex items-center transition-all hover:scale-105">
                    <i class="fas fa-user-plus mr-2"></i> {move || t(lang.get(), "add_patient")}
                </a>
            </div>

            // Ward View / Grid View
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || patients.get()
                    key=|p| p.id.clone()
                    children=move |patient| {
                        let id = patient.id.clone();
                        let id_str = id.clone().map(|t| t.to_string()).unwrap_or_default();

                        view! {
                            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-xl transition-all duration-300 pb-2">
                                <div class="p-6 pb-4">
                                    <div class="flex justify-between items-start mb-4">
                                        <div class="flex items-center">
                                            <div class="bg-indigo-100 text-indigo-600 rounded-full w-12 h-12 flex items-center justify-center text-xl font-bold mr-4">
                                                {patient.first_name.chars().next().unwrap_or('?')}
                                            </div>
                                            <div>
                                                <h3 class="text-xl font-bold text-gray-900">{patient.first_name} {patient.last_name}</h3>
                                                <div class="flex items-center text-gray-500 text-sm mt-1">
                                                    <i class="fas fa-bed mr-2 text-indigo-400"></i>
                                                    <span>{move || t(lang.get(), "bed")} " 1"</span> // Placeholder Bed
                                                </div>
                                            </div>
                                        </div>
                                        <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full uppercase tracking-wider">
                                            {move || t(lang.get(), "stable")}
                                        </span>
                                    </div>

                                    <div class="space-y-3 mb-6">
                                        <div class="flex items-center text-sm text-gray-600 bg-gray-50 p-2 rounded-lg">
                                            <i class="fas fa-file-medical text-teal-500 w-6 text-center mr-2"></i>
                                            <span class="truncate font-medium">{patient.principal_diagnosis}</span>
                                        </div>
                                        <div class="flex items-center text-sm text-gray-600 bg-gray-50 p-2 rounded-lg">
                                            <i class="fas fa-calendar-alt text-teal-500 w-6 text-center mr-2"></i>
                                            <span class="font-medium">{patient.uci_admission_date.split('T').next().unwrap_or("").to_string()}</span>
                                        </div>
                                    </div>

                                    <div class="flex gap-3">
                                        <a href=format!("/patients/{}", id_str) class="flex-1 bg-indigo-50 text-indigo-700 text-center py-3 rounded-xl hover:bg-indigo-600 hover:text-white font-semibold transition-colors flex items-center justify-center gap-2">
                                            <i class="fas fa-chart-line"></i>
                                            {move || t(lang.get(), "view_history")}
                                        </a>
                                    </div>
                                </div>
                                <div class="px-6 py-2 border-t border-gray-50 flex justify-between items-center text-xs text-gray-400">
                                    <span>"ID: " {id_str.split(':').last().unwrap_or("?").to_string()}</span>
                                    <span>"Updated: Today"</span>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
