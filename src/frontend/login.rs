use crate::frontend::i18n::{t, use_i18n};
use crate::server_functions::auth::login;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Login() -> impl IntoView {
    let lang = use_i18n();
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_error_msg.set(None);
        set_loading.set(true);

        let user = username.get();
        let pass = password.get();
        let navigate = navigate.clone();

        spawn_local(async move {
            let result = login(user.clone(), pass.clone()).await;

            match result {
                Ok(response) => {
                    set_loading.set(false);
                    if response.success {
                        let token_str = response.token.unwrap_or_default();
                        let user_id_str = response.user_id.unwrap_or_default();
                        let role_str = response.role.unwrap_or_default();
                        
                        if let Some(storage) = window().local_storage().ok().flatten() {
                            let _ = storage.set_item("uci_token", &token_str);
                            let _ = storage.set_item("uci_user_id", &user_id_str);
                            let _ = storage.set_item("uci_role", &role_str);
                        }

                        if let Some(set_trigger) = use_context::<WriteSignal<i32>>() {
                            set_trigger.update(|v| *v += 1);
                        }

                        navigate("/dashboard", Default::default());
                    } else {
                        set_error_msg.set(Some(response.message));
                    }
                }
                Err(e) => {
                    set_loading.set(false);
                    set_error_msg.set(Some(format!("Error: {}", e)));
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
                        disabled=loading
                        class="w-full bg-indigo-600 text-white py-3.5 sm:py-4 rounded-xl sm:rounded-2xl font-black text-base sm:text-lg hover:bg-indigo-700 transition-all shadow-xl shadow-indigo-200 active:scale-[0.98] flex items-center justify-center gap-2 sm:gap-3 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        <i class={move || if loading.get() { "fas fa-spinner fa-spin" } else { "fas fa-sign-in-alt" }}></i>
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
