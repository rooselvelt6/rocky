use crate::frontend::i18n::{t, use_i18n};
use crate::models::patient::Patient;
use crate::server_functions::patients::get_patients;
use leptos::*;
use leptos_router::*;

#[component]
pub fn PatientList() -> impl IntoView {
    let lang = use_i18n();
    let (patients, set_patients) = create_signal(Vec::<Patient>::new());
    let (search_query, set_search_query) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);

    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match get_patients().await {
                Ok(list) => {
                    set_patients.set(list);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to fetch patients: {}", e);
                }
            }
            set_loading.set(false);
        });
    });

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
                })
                .collect()
        }
    };

    view! {
        <div class="p-6 max-w-7xl mx-auto">
            <div class="flex justify-between items-center mb-8">
                <div>
                    <h2 class="text-3xl font-bold text-gray-800">{move || t(lang.get(), "patient_list")}</h2>
                    <p class="text-gray-500 mt-1">{move || t(lang.get(), "patient_list_overview")}</p>
                </div>
                <A href="/register" class="bg-indigo-600 text-white px-6 py-3 rounded-xl font-bold hover:bg-indigo-700 transition-colors shadow-lg shadow-indigo-200">
                    <i class="fas fa-plus mr-2"></i>
                    {move || t(lang.get(), "new_patient")}
                </A>
            </div>

            <div class="bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">
                <div class="p-4 border-b border-gray-100">
                    <input
                        type="text"
                        placeholder={move || t(lang.get(), "search_patients")}
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-all"
                    />
                </div>

                {move || if loading.get() {
                    view! {
                        <div class="p-12 text-center">
                            <i class="fas fa-spinner fa-spin text-4xl text-indigo-600"></i>
                            <p class="mt-4 text-gray-500">Cargando pacientes...</p>
                        </div>
                    }
                } else {
                    view! {
                        <div class="overflow-x-auto">
                            <table class="w-full">
                                <thead class="bg-gray-50 border-b border-gray-100">
                                    <tr>
                                        <th class="px-6 py-4 text-left text-xs font-bold text-gray-500 uppercase tracking-wider">Paciente</th>
                                        <th class="px-6 py-4 text-left text-xs font-bold text-gray-500 uppercase tracking-wider">Diagnóstico</th>
                                        <th class="px-6 py-4 text-left text-xs font-bold text-gray-500 uppercase tracking-wider">Ingreso UCI</th>
                                        <th class="px-6 py-4 text-left text-xs font-bold text-gray-500 uppercase tracking-wider">VM</th>
                                        <th class="px-6 py-4 text-left text-xs font-bold text-gray-500 uppercase tracking-wider">Acciones</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-100">
                                    {move || filtered_patients().into_iter().map(|patient| {
                                        let patient_id = patient.id.as_ref().map(|i| i.to_string()).unwrap_or_else(|| "0".to_string());
                                        view! {
                                            <tr class="hover:bg-indigo-50/50 transition-colors">
                                                <td class="px-6 py-4">
                                                    <div class="flex items-center">
                                                        <div class="w-10 h-10 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 font-bold mr-3">
                                                            {patient.first_name.chars().next().unwrap_or('P')}
                                                        </div>
                                                        <div>
                                                            <p class="font-bold text-gray-800">{patient.first_name.clone()} {patient.last_name.clone()}</p>
                                                            <p class="text-sm text-gray-500">{patient.date_of_birth.clone()}</p>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4">
                                                    <span class="px-3 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                                        {patient.principal_diagnosis.chars().take(30).collect::<String>()}
                                                    </span>
                                                </td>
                                                <td class="px-6 py-4 text-gray-600">
                                                    {patient.days_in_hospital()} días
                                                </td>
                                                <td class="px-6 py-4">
                                                    {if patient.mechanical_ventilation {
                                                        view! { <span class="text-red-500"><i class="fas fa-lungs"></i> Sí</span> }
                                                    } else {
                                                        view! { <span class="text-green-500">No</span> }
                                                    }}
                                                </td>
                                                <td class="px-6 py-4">
                                                    <A href={format!("/patients/{}", patient_id)} class="text-indigo-600 hover:text-indigo-800 mr-3">
                                                        <i class="fas fa-eye"></i>
                                                    </A>
                                                    <A href={format!("/edit-patient/{}", patient_id)} class="text-blue-600 hover:text-blue-800 mr-3">
                                                        <i class="fas fa-edit"></i>
                                                    </A>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}
