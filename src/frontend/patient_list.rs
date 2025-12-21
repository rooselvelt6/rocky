use crate::frontend::i18n::{t, use_i18n};
use crate::models::patient::Patient;
use leptos::*;

#[component]
pub fn PatientList() -> impl IntoView {
    let lang = use_i18n();
    let (patients, set_patients) = create_signal(Vec::<Patient>::new());
    let (search_query, set_search_query) = create_signal(String::new());

    // Filtered patients based on search query
    let filtered_patients = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            patients.get()
        } else {
            patients
                .get()
                .into_iter()
                .filter(|p| {
                    p.first_name.to_lowercase().contains(&query)
                        || p.last_name.to_lowercase().contains(&query)
                        || p.principal_diagnosis.to_lowercase().contains(&query)
                        || p.id
                            .as_ref()
                            .and_then(|id| Some(id.to_string().to_lowercase().contains(&query)))
                            .unwrap_or(false)
                })
                .collect()
        }
    };

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

            // Search Box
            <div class="mb-6">
                <div class="relative max-w-md">
                    <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                        <i class="fas fa-search text-gray-400"></i>
                    </div>
                    <input
                        type="text"
                        placeholder="Buscar por nombre, apellido, diagnóstico o ID..."
                        class="w-full pl-11 pr-4 py-3 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                        on:input=move |ev| {
                            set_search_query.set(event_target_value(&ev));
                        }
                        prop:value=move || search_query.get()
                    />
                </div>
                {move || {
                    let count = filtered_patients().len();
                    let total = patients.get().len();
                    if search_query.get().is_empty() {
                        view! {
                            <p class="text-sm text-gray-500 mt-2">
                                {format!("Mostrando {} pacientes", total)}
                            </p>
                        }.into_view()
                    } else {
                        view! {
                            <p class="text-sm text-gray-600 mt-2">
                                <span class="font-semibold">{count}</span>
                                {format!(" de {} pacientes", total)}
                            </p>
                        }.into_view()
                    }
                }}
            </div>

            // Ward View / Grid View
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || filtered_patients()
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

                                    <div class="space-y-2">
                                        <div class="grid grid-cols-2 gap-2">
                                            <a
                                                href=format!("/glasgow?patient_id={}", id_str)
                                                class="bg-purple-50 text-purple-700 text-center py-2 px-2 rounded-lg hover:bg-purple-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                                            >
                                                <i class="fas fa-brain text-xs"></i>
                                                "Glasgow"
                                            </a>
                                            <a
                                                href=format!("/apache?patient_id={}", id_str)
                                                class="bg-red-50 text-red-700 text-center py-2 px-2 rounded-lg hover:bg-red-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                                            >
                                                <i class="fas fa-heartbeat text-xs"></i>
                                                "APACHE"
                                            </a>
                                        </div>
                                        <div class="grid grid-cols-2 gap-2">
                                            <a
                                                href=format!("/sofa?patient_id={}", id_str)
                                                class="bg-teal-50 text-teal-700 text-center py-2 px-2 rounded-lg hover:bg-teal-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                                            >
                                                <i class="fas fa-heart text-xs"></i>
                                                "SOFA"
                                            </a>
                                            <a
                                                href=format!("/saps?patient_id={}", id_str)
                                                class="bg-orange-50 text-orange-700 text-center py-2 px-2 rounded-lg hover:bg-orange-600 hover:text-white text-xs font-semibold transition-all flex items-center justify-center gap-1"
                                            >
                                                <i class="fas fa-procedures text-xs"></i>
                                                "SAPS II"
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
