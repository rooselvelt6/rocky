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
        let navigate = navigate.clone();

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

                        // Try to trigger auth update in parent
                        if let Some(set_trigger) = use_context::<WriteSignal<i32>>() {
                            set_trigger.update(|v| *v += 1);
                        }

                        // Redirect to Dashboard
                        navigate("/dashboard", Default::default());
                    }
                }
                _ => {
                    set_error_msg.set(Some(t(lang.get(), "invalid_credentials")));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col items-center justify-center min-h-[70vh] sm:min-h-[75vh] py-8 sm:py-12 px-4 sm:px-6 lg:px-8">
            <div class="bg-white/80 backdrop-blur-md p-6 sm:p-10 rounded-2xl sm:rounded-3xl shadow-2xl w-full max-w-md border border-white/20">
                <div class="text-center mb-8 sm:mb-10">
                    <div class="inline-flex items-center justify-center w-16 h-16 sm:w-20 sm:h-20 bg-indigo-50 text-indigo-600 rounded-xl sm:rounded-2xl mb-4 sm:mb-6 shadow-sm border border-indigo-100/50">
                        <i class="fas fa-user-shield text-2xl sm:text-3xl"></i>
                    </div>
                    <h2 class="text-2xl sm:text-3xl font-extrabold text-indigo-950 tracking-tight">{move || t(lang.get(), "secure_access")}</h2>
                    <p class="text-indigo-600/60 mt-2 sm:mt-3 font-medium text-sm sm:text-base">{move || t(lang.get(), "login_subtitle")}</p>
                </div>

                <form on:submit=on_submit class="space-y-4 sm:space-y-6">
                    <div class="space-y-1.5 sm:space-y-2">
                        <label class="block text-[10px] sm:text-xs font-bold uppercase tracking-wider text-indigo-900/50 ml-1">{move || t(lang.get(), "username")}</label>
                        <div class="relative group">
                            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                <i class="fas fa-at text-indigo-300 group-focus-within:text-indigo-500 transition-colors"></i>
                            </div>
                            <input
                                type="text"
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                prop:value=username
                                class="w-full pl-11 sm:pl-12 pr-4 py-3 sm:py-3.5 bg-indigo-50/50 border border-indigo-100 rounded-xl sm:rounded-2xl focus:ring-4 focus:ring-indigo-500/10 focus:border-indigo-500 focus:bg-white transition-all outline-none"
                                placeholder=move || t(lang.get(), "username_placeholder")
                                required
                            />
                        </div>
                    </div>

                    <div class="space-y-1.5 sm:space-y-2">
                        <label class="block text-[10px] sm:text-xs font-bold uppercase tracking-wider text-indigo-900/50 ml-1">{move || t(lang.get(), "password")}</label>
                        <div class="relative group">
                            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                <i class="fas fa-fingerprint text-indigo-300 group-focus-within:text-indigo-500 transition-colors"></i>
                            </div>
                            <input
                                type="password"
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                prop:value=password
                                class="w-full pl-11 sm:pl-12 pr-4 py-3 sm:py-3.5 bg-indigo-50/50 border border-indigo-100 rounded-xl sm:rounded-2xl focus:ring-4 focus:ring-indigo-500/10 focus:border-indigo-500 focus:bg-white transition-all outline-none"
                                placeholder=move || t(lang.get(), "password_placeholder")
                                required
                            />
                        </div>
                    </div>

                    {move || error_msg.get().map(|err| view! {
                        <div class="p-3 sm:p-4 rounded-xl sm:rounded-2xl bg-red-50 border border-red-100 text-red-600 text-xs sm:text-sm flex items-center gap-2 sm:gap-3 animate-pulse">
                            <i class="fas fa-shield-virus text-base sm:text-lg"></i>
                            <span class="font-medium">{err}</span>
                        </div>
                    })}

                    <button
                        type="submit"
                        class="w-full bg-indigo-600 text-white py-3.5 sm:py-4 rounded-xl sm:rounded-2xl font-black text-base sm:text-lg hover:bg-indigo-700 transition-all shadow-xl shadow-indigo-200 active:scale-[0.98] flex items-center justify-center gap-2 sm:gap-3"
                    >
                        <i class="fas fa-sign-in-alt"></i>
                        {move || t(lang.get(), "login_btn")}
                    </button>
                </form>

                <div class="mt-8 sm:mt-10 pt-6 sm:pt-8 border-t border-indigo-50 text-center">
                    <p class="text-[10px] sm:text-xs font-bold text-indigo-900/40 uppercase tracking-widest mb-1">"UCI Management Engine"</p>
                    <p class="text-[9px] sm:text-[10px] text-indigo-300">"v1.0.0 - Professional Clinical Bio-Security"</p>
                </div>
            </div>
        </div>
    }
}
