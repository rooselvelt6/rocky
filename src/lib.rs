use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;

#[component]
pub fn App() -> impl IntoView {
    let (page, set_page) = create_signal(String::from("/"));
    let (logged_in, set_logged_in) = create_signal(false);
    let (user_role, set_user_role) = create_signal(String::new());

    let check_auth = move || {
        if let Some(storage) = window().local_storage().ok().flatten() {
            let token = storage.get_item("uci_token").ok().flatten();
            if token.is_some() {
                set_logged_in.set(true);
                if let Ok(role) = storage.get_item("uci_role") {
                    set_user_role.set(role.unwrap_or_default());
                }
            }
        }
    };

    check_auth();

    view! {
        <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            <nav class="bg-indigo-900 text-white p-4">
                <div class="container mx-auto flex justify-between items-center">
                    <button on:click=move |_| set_page.set("/".to_string()) class="text-xl font-bold text-white bg-transparent border-0 cursor-pointer">"UCI System"</button>
                    <div class="flex gap-4">
                        <button on:click=move |_| set_page.set("/patients".to_string()) class="text-white bg-transparent border-0 cursor-pointer">"Pacientes"</button>
                        <button on:click=move |_| set_page.set("/ward".to_string()) class="text-white bg-transparent border-0 cursor-pointer">"Monitoreo"</button>
                        <Show when=move || user_role.get() == "Admin">
                            <button on:click=move |_| set_page.set("/admin".to_string()) class="text-white bg-transparent border-0 cursor-pointer">"Admin"</button>
                        </Show>
                        <Show when=move || logged_in.get()>
                            <button on:click=move |_| {
                                if let Some(s) = window().local_storage().ok().flatten() {
                                    let _ = s.remove_item("uci_token");
                                    let _ = s.remove_item("uci_role");
                                    set_logged_in.set(false);
                                    set_page.set("/".to_string());
                                }
                            } class="bg-transparent text-white border-0 cursor-pointer">"Logout"</button>
                        </Show>
                        <Show when=move || !logged_in.get()>
                            <button on:click=move |_| set_page.set("/login".to_string()) class="text-white bg-transparent border-0 cursor-pointer">"Login"</button>
                        </Show>
                    </div>
                </div>
            </nav>
            <main class="container mx-auto p-4">
                {move || {
                    let p = page.get();
                    match p.as_str() {
                        "/patients" => view! { <PatientList/> }.into_any(),
                        "/ward" => view! { <WardView/> }.into_any(),
                        "/admin" => view! { <AdminPanel/> }.into_any(),
                        "/login" => view! { <Login/> }.into_any(),
                        _ => view! { <Home/> }.into_any(),
                    }
                }}
            </main>
        </div>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="text-center py-16">
            <h1 class="text-4xl font-bold text-indigo-900 mb-4">"Sistema UCI"</h1>
            <p class="text-xl text-gray-600">"Gestión de pacientes y escalas médicas"</p>
            <div class="mt-8 flex justify-center gap-4">
                <button class="px-6 py-3 bg-indigo-600 text-white rounded-lg">"Ver Pacientes"</button>
                <button class="px-6 py-3 bg-green-600 text-white rounded-lg">"Monitoreo"</button>
            </div>
        </div>
    }
}

#[component]
fn Login() -> impl IntoView {
    let (error, set_error) = create_signal(String::new());

    let do_login = move |_| {
        spawn_local(async move {
            let res = reqwasm::http::Request::get("/api/login")
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if data.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                            if let Some(storage) = window().local_storage().ok().flatten() {
                                let _ = storage.set_item("uci_token", data.get("token").and_then(|v| v.as_str()).unwrap_or(""));
                                let _ = storage.set_item("uci_role", data.get("role").and_then(|v| v.as_str()).unwrap_or(""));
                            }
                        } else {
                            set_error.set("Login fallido".to_string());
                        }
                    }
                }
                Err(e) => set_error.set(format!("Error: {}", e))
            }
        });
    };

    view! {
        <div class="max-w-md mx-auto mt-16 p-8 bg-white rounded-lg shadow-lg">
            <h2 class="text-2xl font-bold mb-6 text-center">"Login"</h2>
            <Show when=move || !error.get().is_empty()>
                <p class="text-red-600 mb-4">{error.get()}</p>
            </Show>
            <div class="space-y-4">
                <input type="text" placeholder="Usuario" class="w-full p-3 border rounded" />
                <input type="password" placeholder="Contraseña" class="w-full p-3 border rounded" />
                <button on:click=do_login class="w-full py-3 bg-indigo-600 text-white rounded hover:bg-indigo-700">
                    "Iniciar Sesión"
                </button>
            </div>
        </div>
    }
}

#[component]
fn PatientList() -> impl IntoView {
    let (patients, set_patients) = create_signal(Vec::<Value>::new());

    spawn_local(async move {
        let res = reqwasm::http::Request::get("/api/patients")
            .send()
            .await;
        
        if let Ok(resp) = res {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(list) = data.get("patients").and_then(|v| v.as_array()) {
                    set_patients.set(list.clone());
                }
            }
        }
    });

    view! {
        <div>
            <h2 class="text-2xl font-bold mb-4">"Pacientes"</h2>
            <div class="space-y-4">
                {move || patients.get().into_iter().map(|patient| {
                    let first = patient.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let last = patient.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let diag = patient.get("principal_diagnosis").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let id = patient.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    view! {
                        <div data-id={id} class="block p-4 bg-white rounded-lg shadow hover:shadow-lg transition cursor-pointer">
                            <p class="font-bold">{first} {last}</p>
                            <p class="text-gray-600">{diag}</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn WardView() -> impl IntoView {
    let (patients, set_patients) = create_signal(Vec::<Value>::new());

    spawn_local(async move {
        let res = reqwasm::http::Request::get("/api/patients")
            .send()
            .await;
        
        if let Ok(resp) = res {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(list) = data.get("patients").and_then(|v| v.as_array()) {
                    set_patients.set(list.clone());
                }
            }
        }
    });

    view! {
        <div>
            <h2 class="text-2xl font-bold mb-4">"Unidad de Cuidados Intensivos"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {move || patients.get().into_iter().map(|patient| {
                    let first = patient.get("first_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let last = patient.get("last_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let diag = patient.get("principal_diagnosis").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    view! {
                        <div class="bg-white p-4 rounded-lg shadow border-l-4 border-green-500">
                            <p class="font-bold">{first} {last}</p>
                            <p class="text-sm text-gray-600">{diag}</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn AdminPanel() -> impl IntoView {
    let (stats, set_stats) = create_signal(serde_json::json!({
        "total_patients": 0,
        "active_patients": 0,
        "total_assessments": 0,
        "critical_patients": 0
    }));

    spawn_local(async move {
        let res = reqwasm::http::Request::get("/api/admin/stats")
            .send()
            .await;
        
        if let Ok(resp) = res {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                set_stats.set(data);
            }
        }
    });

    let total = move || stats.get().get("total_patients").and_then(|v| v.as_i64()).unwrap_or(0);
    let active = move || stats.get().get("active_patients").and_then(|v| v.as_i64()).unwrap_or(0);
    let assessments = move || stats.get().get("total_assessments").and_then(|v| v.as_i64()).unwrap_or(0);
    let critical = move || stats.get().get("critical_patients").and_then(|v| v.as_i64()).unwrap_or(0);

    view! {
        <div>
            <h2 class="text-2xl font-bold mb-4">"Panel de Administración"</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div class="bg-white p-6 rounded-lg shadow text-center">
                    <p class="text-3xl font-bold text-indigo-600">{total()}</p>
                    <p class="text-gray-600">"Total Pacientes"</p>
                </div>
                <div class="bg-white p-6 rounded-lg shadow text-center">
                    <p class="text-3xl font-bold text-green-600">{active()}</p>
                    <p class="text-gray-600">"Pacientes Activos"</p>
                </div>
                <div class="bg-white p-6 rounded-lg shadow text-center">
                    <p class="text-3xl font-bold text-blue-600">{assessments()}</p>
                    <p class="text-gray-600">"Evaluaciones"</p>
                </div>
                <div class="bg-white p-6 rounded-lg shadow text-center">
                    <p class="text-3xl font-bold text-red-600">{critical()}</p>
                    <p class="text-gray-600">"Críticos"</p>
                </div>
            </div>
        </div>
    }
}
