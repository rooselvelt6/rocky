use crate::frontend::i18n::{t, use_i18n};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthState {
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub role: Option<String>,
}

#[component]
pub fn Login() -> impl IntoView {
    let lang = use_i18n();
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error_msg, set_error_msg) = create_signal(None::<String>);

    let navigate = use_navigate();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_error_msg.set(None);

        let user = username.get();
        let pass = password.get();

        spawn_local(async move {
            let res = reqwasm::http::Request::post("/api/login")
                .header("Content-Type", "application/json")
                .body(
                    serde_json::to_string(&serde_json::json!({
                        "username": user,
                        "password": pass
                    }))
                    .unwrap(),
                )
                .send()
                .await;

            match res {
                Ok(resp) if resp.ok() => {
                    let text = resp.text().await.unwrap_or_default();
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        let token = data["token"].as_str().unwrap_or_default();
                        let user_id = data["user"]["id"].as_str().unwrap_or_default();
                        let role = data["user"]["role"].as_str().unwrap_or_default();

                        // Store in storage
                        if let Some(storage) = window().local_storage().ok().flatten() {
                            let _ = storage.set_item("uci_token", token);
                            let _ = storage.set_item("uci_user_id", user_id);
                            let _ = storage.set_item("uci_role", role);
                        }

                        // Redirect to home
                        navigate("/", Default::default());
                    }
                }
                _ => {
                    set_error_msg.set(Some("Invalid username or password".to_string()));
                }
            }
        });
    };

    view! {
        <div class="flex items-center justify-center min-h-[60vh]">
            <div class="bg-white p-8 rounded-2xl shadow-xl w-full max-w-md border border-indigo-50">
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-16 h-16 bg-indigo-100 rounded-full mb-4">
                        <i class="fas fa-lock text-indigo-600 text-2xl"></i>
                    </div>
                    <h2 class="text-2xl font-bold text-gray-800">"Acceso al Sistema"</h2>
                    <p class="text-gray-500 mt-2">"Ingrese sus credenciales para continuar"</p>
                </div>

                <form on:submit=on_submit class="space-y-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Usuario"</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <i class="fas fa-user text-gray-400"></i>
                            </div>
                            <input
                                type="text"
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                prop:value=username
                                class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                                placeholder="admin"
                                required
                            />
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Contraseña"</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <i class="fas fa-key text-gray-400"></i>
                            </div>
                            <input
                                type="password"
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                prop:value=password
                                class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                                placeholder="••••••••"
                                required
                            />
                        </div>
                    </div>

                    {move || error_msg.get().map(|err| view! {
                        <div class="p-3 rounded-lg bg-red-50 text-red-600 text-sm flex items-center gap-2">
                            <i class="fas fa-exclamation-circle"></i>
                            {err}
                        </div>
                    })}

                    <button
                        type="submit"
                        class="w-full bg-indigo-600 text-white py-3 rounded-xl font-bold hover:bg-indigo-700 transition-all shadow-lg hover:shadow-indigo-200"
                    >
                        "Iniciar Sesión"
                    </button>
                </form>

                <div class="mt-6 text-center text-xs text-gray-400">
                    <p>"UCI Medical Scales Management System"</p>
                    <p>"v0.1.0 - Modo Desarrollo"</p>
                </div>
            </div>
        </div>
    }
}
