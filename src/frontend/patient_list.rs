use crate::frontend::i18n::{t, use_i18n};
use crate::models::patient::Patient;
use leptos::*;

#[component]
pub fn PatientList() -> impl IntoView {
    let lang = use_i18n();
    let (patients, set_patients) = create_signal(Vec::<Patient>::new());
    let (search_query, set_search_query) = create_signal(String::new());
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);

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

    create_effect(move |_| {
        refresh_trigger.get();
        spawn_local(async move {
            let mut req = reqwasm::http::Request::get("/api/patients");

            if let Some(storage) = window().local_storage().ok().flatten() {
                if let Some(token) = storage.get_item("uci_token").unwrap_or(None) {
                    req = req.header("Authorization", &format!("Bearer {}", token));
                }
            }

            let res = req.send().await;

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
                    <p class="text-gray-500 mt-1">{move || t(lang.get(), "patient_list_overview")}</p>
                </div>
                <div class="flex gap-4">
                    {move || {
                        let current_list = filtered_patients();
                        view! {
                            <crate::frontend::components::export_button::ExportButton
                                data=current_list
                                filename="patients_export".to_string()
                            />
                        }
                    }}
                    <a href="/register" class="bg-indigo-600 text-white px-6 py-3 rounded-xl hover:bg-indigo-700 shadow-md flex items-center transition-all hover:scale-105">
                        <i class="fas fa-user-plus mr-2"></i> {move || t(lang.get(), "add_patient")}
                    </a>
                </div>
            </div>

            // Search Box
            <div class="mb-6">
                <div class="relative max-w-md">
                    <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                        <i class="fas fa-search text-gray-400"></i>
                    </div>
                    <input
                        type="text"
                        placeholder=move || t(lang.get(), "search_placeholder")
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
                                {move || t(lang.get(), "showing_n_patients").replace("{}", &total.to_string())}
                            </p>
                        }.into_view()
                    } else {
                        view! {
                            <p class="text-sm text-gray-600 mt-2">
                                <span class="font-semibold">{count}</span>
                                {move || t(lang.get(), "of_n_patients").replace("{}", &total.to_string())}
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
                        view! {
                            <crate::frontend::components::patient_card::PatientCard
                                patient=patient
                                on_delete=Callback::new(move |_| set_refresh_trigger.update(|n| *n += 1))
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}
