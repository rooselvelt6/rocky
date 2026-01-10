// src/frontend/admin.rs
use crate::frontend::i18n::{t, use_i18n};
use crate::models::config::SystemConfig;
use crate::models::user::User;
use leptos::*;

#[component]
pub fn AdminPanel() -> impl IntoView {
    let lang = use_i18n();
    let (active_tab, set_active_tab) = create_signal("users");

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
            <div class="mb-8">
                <h1 class="text-4xl font-black text-indigo-950 tracking-tight mb-2">{move || t(lang.get(), "system_config_title")}</h1>
                <p class="text-indigo-600/60 font-medium">{move || t(lang.get(), "admin_panel_subtitle")}</p>
            </div>

            // Navigation Tabs
            <div class="flex gap-4 mb-8 border-b border-indigo-100 pb-1">
                <button
                    on:click=move |_| set_active_tab.set("users")
                    class=move || format!("px-6 py-3 font-bold text-sm uppercase tracking-widest transition-all {}",
                        if active_tab.get() == "users" { "text-indigo-600 border-b-2 border-indigo-600 bg-indigo-50/50" }
                        else { "text-indigo-900/40 hover:text-indigo-600" })
                >
                    <i class="fas fa-users-cog mr-2"></i> {move || t(lang.get(), "tab_personal")}
                </button>
                <button
                    on:click=move |_| set_active_tab.set("theme")
                    class=move || format!("px-6 py-3 font-bold text-sm uppercase tracking-widest transition-all {}",
                        if active_tab.get() == "theme" { "text-indigo-600 border-b-2 border-indigo-600 bg-indigo-50/50" }
                        else { "text-indigo-900/40 hover:text-indigo-600" })
                >
                    <i class="fas fa-palette mr-2"></i> {move || t(lang.get(), "tab_interface")}
                </button>
            </div>

            <div class="grid grid-cols-1 gap-8">
                {move || match active_tab.get() {
                    "users" => view! { <UserManagement /> }.into_view(),
                    "theme" => view! { <ThemeSettings /> }.into_view(),
                    _ => view! { <div>"Seleccione una opción"</div> }.into_view(),
                }}
            </div>
        </div>
    }
}

#[component]
fn UserManagement() -> impl IntoView {
    let lang = use_i18n();
    let users = create_resource(
        || (),
        |_| async move {
            let res = reqwasm::http::Request::get("/api/admin/users")
                .header(
                    "Authorization",
                    &format!(
                        "Bearer {}",
                        window()
                            .local_storage()
                            .ok()
                            .flatten()
                            .and_then(|s| s.get_item("uci_token").ok().flatten())
                            .unwrap_or_default()
                    ),
                )
                .send()
                .await;

            match res {
                Ok(resp) => resp.json::<Vec<User>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        },
    );

    view! {
        <div class="bg-white rounded-3xl shadow-xl shadow-indigo-100/50 border border-indigo-50 overflow-hidden">
            <div class="p-6 border-b border-indigo-50 flex justify-between items-center bg-indigo-50/10">
                <h3 class="text-lg font-bold text-indigo-950">{move || t(lang.get(), "authorized_personnel")}</h3>
                <button class="bg-indigo-600 text-white px-4 py-2 rounded-xl text-xs font-black shadow-lg shadow-indigo-200 hover:bg-indigo-700 transition-all flex items-center gap-2">
                    <i class="fas fa-plus"></i> {move || t(lang.get(), "add_staff")}
                </button>
            </div>
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-indigo-50/30">
                        <tr>
                            <th class="px-6 py-4 text-[10px] font-black text-indigo-900/40 uppercase tracking-widest">{move || t(lang.get(), "table_user")}</th>
                            <th class="px-6 py-4 text-[10px] font-black text-indigo-900/40 uppercase tracking-widest">{move || t(lang.get(), "table_name")}</th>
                            <th class="px-6 py-4 text-[10px] font-black text-indigo-900/40 uppercase tracking-widest">{move || t(lang.get(), "table_role")}</th>
                            <th class="px-6 py-4 text-[10px] font-black text-indigo-900/40 uppercase tracking-widest">{move || t(lang.get(), "table_status")}</th>
                            <th class="px-6 py-4 text-[10px] font-black text-indigo-900/40 uppercase tracking-widest text-right">{move || t(lang.get(), "table_actions")}</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-indigo-50">
                        {move || users.get().map(|list| {
                            list.into_iter().map(|user| view! {
                                <tr class="hover:bg-indigo-50/20 transition-colors">
                                    <td class="px-6 py-4 font-bold text-indigo-900">{user.username}</td>
                                    <td class="px-6 py-4 text-indigo-600/80 text-sm">{user.full_name}</td>
                                    <td class="px-6 py-4">
                                        <span class="px-2 py-1 bg-indigo-100 text-indigo-700 text-[10px] font-black rounded-lg uppercase">
                                            {format!("{:?}", user.role)}
                                        </span>
                                    </td>
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-2">
                                            <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                                            <span class="text-xs font-bold text-green-600">{move || t(lang.get(), "user_active")}</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 text-right space-x-2">
                                        <button class="p-2 text-indigo-400 hover:text-indigo-600 transition-colors">
                                            <i class="fas fa-edit"></i>
                                        </button>
                                        <button class="p-2 text-red-300 hover:text-red-500 transition-colors">
                                            <i class="fas fa-trash"></i>
                                        </button>
                                    </td>
                                </tr>
                            }).collect::<Vec<_>>()
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn ThemeSettings() -> impl IntoView {
    let lang = use_i18n();
    let config = create_resource(
        || (),
        |_| async move {
            let res = reqwasm::http::Request::get("/api/admin/config")
                .header(
                    "Authorization",
                    &format!(
                        "Bearer {}",
                        window()
                            .local_storage()
                            .ok()
                            .flatten()
                            .and_then(|s| s.get_item("uci_token").ok().flatten())
                            .unwrap_or_default()
                    ),
                )
                .send()
                .await;

            match res {
                Ok(resp) => resp.json::<SystemConfig>().await.ok(),
                Err(_) => None,
            }
        },
    );

    let (primary, set_primary) = create_signal("#4F46E5".to_string());

    let config_val = config.clone();
    let set_primary_val = set_primary;
    create_effect(move |_| {
        if let Some(Some(conf)) = config_val.get() {
            set_primary_val.set(conf.primary_color);
        }
    });

    let primary_val = primary;
    let on_save = move |_| {
        let color = primary_val.get();
        spawn_local(async move {
            let _ = reqwasm::http::Request::put("/api/admin/config")
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    &format!(
                        "Bearer {}",
                        window()
                            .local_storage()
                            .ok()
                            .flatten()
                            .and_then(|s| s.get_item("uci_token").ok().flatten())
                            .unwrap_or_default()
                    ),
                )
                .body(
                    serde_json::to_string(&SystemConfig {
                        id: None,
                        hospital_name: "UCI".into(),
                        primary_color: color.clone(),
                        secondary_color: "#1E1B4B".into(),
                        accent_color: "#FACC15".into(),
                        logo_url: None,
                        updated_at: chrono::Utc::now(),
                    })
                    .unwrap(),
                )
                .send()
                .await;

            // Re-apply property to document root for instant feedback
            if let Some(doc) = window().document() {
                if let Some(el) = doc.document_element() {
                    use wasm_bindgen::JsCast;
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        let _ = html_el.style().set_property("--primary-color", &color);
                    }
                }
            }
        });
    };

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div class="bg-white p-8 rounded-3xl shadow-xl shadow-indigo-100/50 border border-indigo-50">
                <h3 class="text-lg font-bold text-indigo-950 mb-6">{move || t(lang.get(), "brand_colors")}</h3>
                <div class="space-y-6">
                    <div class="space-y-2">
                        <label class="block text-xs font-bold text-indigo-900/50 uppercase tracking-wider ml-1">{move || t(lang.get(), "primary_color")}</label>
                        <div class="flex gap-4 items-center">
                            <input
                                type="color"
                                on:input=move |ev| set_primary.set(event_target_value(&ev))
                                prop:value=primary
                                class="w-12 h-12 rounded-xl border-none cursor-pointer"
                            />
                            <input
                                type="text"
                                on:input=move |ev| set_primary.set(event_target_value(&ev))
                                prop:value=primary
                                class="flex-1 px-4 py-3 bg-indigo-50/50 border border-indigo-100 rounded-xl focus:ring-2 focus:ring-indigo-500 transition-all font-mono"
                            />
                        </div>
                    </div>
                </div>

                <button
                    on:click=on_save
                    class="w-full mt-10 bg-indigo-900 text-white py-4 rounded-2xl font-black text-sm hover:bg-indigo-950 transition-all shadow-xl shadow-indigo-200 uppercase tracking-widest"
                >
                    {move || t(lang.get(), "save_visual_changes")}
                </button>
            </div>

            <div class="bg-indigo-900 p-8 rounded-3xl shadow-2xl text-white flex flex-col justify-center relative overflow-hidden">
                <div class="relative z-10">
                    <h3 class="text-xl font-black mb-4">{move || t(lang.get(), "preview_view")}</h3>
                    <div
                        class="p-6 rounded-2xl shadow-lg border border-white/10 backdrop-blur-sm"
                        style:background-color=move || primary.get()
                    >
                        <p class="font-bold opacity-80 mb-2 text-xs">{move || t(lang.get(), "preview_button_label")}</p>
                        <div class="bg-white text-indigo-950 py-3 px-6 rounded-xl font-black text-center text-sm">
                            {move || t(lang.get(), "preview_button_text")}
                        </div>
                    </div>
                    <p class="mt-6 text-sm opacity-60 leading-relaxed">
                        {move || t(lang.get(), "preview_description")}
                    </p>
                </div>
                // Subtle decorative icon
                <i class="fas fa-brush absolute -right-4 -bottom-4 text-white/5 text-9xl"></i>
            </div>
        </div>
    }
}
