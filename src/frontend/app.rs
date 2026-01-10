use crate::frontend::apache_form::ApacheForm;
use crate::frontend::dashboard::Dashboard;
use crate::frontend::glasgow_form::GlasgowForm;
use crate::frontend::i18n::{t, Language};
use crate::frontend::login::Login;
use crate::frontend::patient_detail::PatientDetail;
use crate::frontend::patient_form::PatientForm;
use crate::frontend::patient_list::PatientList;
use crate::frontend::saps_form::SapsForm;
use crate::frontend::sofa_form::SofaForm;
use crate::frontend::ward_view::WardView;
use leptos::*;
use leptos_router::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Initialize language signal
    let (lang, set_lang) = create_signal(Language::default());

    // Provide language signal to all children
    provide_context(Signal::from(lang));

    let toggle_lang = move |_| {
        set_lang.update(|l| {
            *l = if *l == Language::En {
                Language::Es
            } else {
                Language::En
            }
        });
    };

    let (user_id, set_user_id) = create_signal(None::<String>);
    let (auth_trigger, set_auth_trigger) = create_signal(0);

    // Check auth whenever trigger changes or on mount
    create_effect(move |_| {
        auth_trigger.get(); // Subscribe to trigger
        if let Some(storage) = window().local_storage().ok().flatten() {
            set_user_id.set(storage.get_item("uci_user_id").ok().flatten());
        }
    });

    // Provide auth trigger to children so Login can update it
    provide_context(set_auth_trigger);

    let on_logout = move |_| {
        if let Some(storage) = window().local_storage().ok().flatten() {
            let _ = storage.remove_item("uci_token");
            let _ = storage.remove_item("uci_user_id");
            let _ = storage.remove_item("uci_role");
            set_user_id.set(None);
            let _ = window().location().set_href("/");
        }
    };

    view! {
        <Router>
            <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 font-sans flex flex-col">
                <nav class="bg-indigo-900 text-white shadow-lg sticky top-0 z-50">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div class="flex items-center justify-between h-16">
                            <div class="flex items-center">
                                <A href="/" class="text-xl font-bold tracking-wider hover:text-indigo-200 transition-colors">"UCI System"</A>
                                <div class="ml-10 flex items-baseline space-x-2">
                                    {move || user_id.get().is_some().then(|| view! {
                                        <>
                                            <A href="/patients" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                                <i class="fas fa-users mr-2"></i>"Pacientes"
                                            </A>
                                            <A href="/dashboard" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                                <i class="fas fa-chart-line mr-2"></i>"Escalas"
                                            </A>
                                            <A href="/ward" class="px-3 py-2 rounded-md text-sm font-medium bg-indigo-800 text-green-300 hover:bg-indigo-700 hover:text-green-200 transition-colors border border-green-500/30 flex items-center gap-2">
                                                <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                                                <i class="fas fa-hospital mr-1"></i>"Monitor"
                                            </A>
                                        </>
                                    })}
                                </div>
                            </div>
                            <div class="flex items-center space-x-4">
                                {move || match user_id.get() {
                                    Some(id) => view! {
                                        <div class="flex items-center gap-4">
                                            <span class="text-xs text-indigo-300 font-mono hidden md:inline">{id}</span>
                                            <button
                                                on:click=on_logout
                                                class="px-3 py-1 rounded-lg bg-red-800/50 hover:bg-red-700 transition-colors border border-red-500/30 text-xs flex items-center gap-2"
                                            >
                                                <i class="fas fa-sign-out-alt"></i>
                                                <span class="hidden sm:inline">"Cerrar Sesión"</span>
                                            </button>
                                        </div>
                                    }.into_view(),
                                    None => view! {
                                        <A href="/login" class="px-4 py-2 rounded-lg bg-green-600 hover:bg-green-700 transition-colors text-sm font-bold shadow-md shadow-green-900/20">
                                            <i class="fas fa-sign-in-alt mr-2"></i> "Acceder"
                                        </A>
                                    }.into_view()
                                }}
                                <button
                                    on:click=toggle_lang
                                    class="flex items-center space-x-2 px-3 py-1 rounded-full bg-indigo-800 hover:bg-indigo-700 transition-colors border border-indigo-600"
                                >
                                    <span class="text-lg">{move || if lang.get() == Language::En { "🇺🇸" } else { "🇪🇸" }}</span>
                                    <span class="text-xs font-bold">{move || if lang.get() == Language::En { "EN" } else { "ES" }}</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </nav>

                <main class="flex-grow">
                    <Routes>
                        <Route path="/" view=move || view! {
                            <div class="max-w-5xl mx-auto px-4 py-16">
                                <div class="text-center mb-16">
                                    <div class="inline-block p-4 bg-indigo-100 rounded-3xl mb-8">
                                        <i class="fas fa-hospital text-6xl text-indigo-600"></i>
                                    </div>
                                    <h1 class="text-5xl font-extrabold text-indigo-950 mb-6 tracking-tight">
                                        {move || t(lang.get(), "welcome_title")}
                                    </h1>
                                    <p class="text-2xl text-indigo-700 mb-4 font-light max-w-3xl mx-auto">
                                        {move || t(lang.get(), "icu_info_desc")}
                                    </p>
                                </div>

                                <div class="grid md:grid-cols-3 gap-8 max-w-6xl mx-auto">
                                    <div class="bg-white/70 backdrop-blur-sm rounded-2xl p-8 border border-indigo-100 shadow-lg hover:shadow-xl transition-shadow">
                                        <div class="w-12 h-12 bg-indigo-100 rounded-full flex items-center justify-center mb-4">
                                            <i class="fas fa-heartbeat text-indigo-600 text-xl"></i>
                                        </div>
                                        <h3 class="text-xl font-bold text-gray-800 mb-3">"Monitoreo Continuo"</h3>
                                        <p class="text-gray-600 leading-relaxed">"Vigilancia 24/7 de signos vitales y parámetros críticos en pacientes con condiciones potencialmente mortales."</p>
                                    </div>

                                    <div class="bg-white/70 backdrop-blur-sm rounded-2xl p-8 border border-indigo-100 shadow-lg hover:shadow-xl transition-shadow">
                                        <div class="w-12 h-12 bg-teal-100 rounded-full flex items-center justify-center mb-4">
                                            <i class="fas fa-user-md text-teal-600 text-xl"></i>
                                        </div>
                                        <h3 class="text-xl font-bold text-gray-800 mb-3">"Equipo Especializado"</h3>
                                        <p class="text-gray-600 leading-relaxed">"Personal médico altamente capacitado en cuidados críticos y medicina intensiva."</p>
                                    </div>

                                    <div class="bg-white/70 backdrop-blur-sm rounded-2xl p-8 border border-indigo-100 shadow-lg hover:shadow-xl transition-shadow">
                                        <div class="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center mb-4">
                                            <i class="fas fa-chart-line text-purple-600 text-xl"></i>
                                        </div>
                                        <h3 class="text-xl font-bold text-gray-800 mb-3">"Escalas Clínicas"</h3>
                                        <p class="text-gray-600 leading-relaxed">"Evaluación objetiva mediante APACHE II, SOFA, SAPS II y Glasgow para optimizar tratamientos."</p>
                                    </div>
                                </div>
                            </div>
                        }/>
                        <Route path="/login" view=Login/>

                        // Protected Routes
                        <Route path="/dashboard" view=|| view! { <Protected><Dashboard/></Protected> }/>
                        <Route path="/patients" view=|| view! { <Protected><PatientList/></Protected> }/>
                        <Route path="/register" view=|| view! { <Protected><PatientForm/></Protected> }/>
                        <Route path="/patients/:id" view=|| view! { <Protected><PatientDetail/></Protected> }/>
                        <Route path="/glasgow" view=|| view! { <Protected><GlasgowForm/></Protected> }/>
                        <Route path="/apache" view=|| view! { <Protected><ApacheForm/></Protected> }/>
                        <Route path="/sofa" view=|| view! { <Protected><SofaForm/></Protected> }/>
                        <Route path="/saps" view=|| view! { <Protected><SapsForm/></Protected> }/>
                        <Route path="/ward" view=|| view! { <Protected><WardView/></Protected> }/>
                    </Routes>
                </main>

                <footer class="bg-indigo-950 text-indigo-300 py-12 border-t border-indigo-900">
                    <div class="max-w-4xl mx-auto px-4 text-center">
                        <div class="mb-6 pb-6 border-b border-indigo-900/50">
                            <h4 class="text-xs font-bold uppercase tracking-wider text-indigo-100 mb-2">
                                <i class="fas fa-exclamation-triangle mr-2 text-amber-500"></i>
                                {move || t(lang.get(), "medical_disclaimer_title")}
                            </h4>
                            <p class="text-xs leading-relaxed opacity-70">
                                {move || t(lang.get(), "medical_disclaimer_text")}
                            </p>
                        </div>
                        <p class="text-sm opacity-70 mb-2">{move || t(lang.get(), "made_with_love")}</p>
                        <div class="flex items-center justify-center gap-4 text-xs opacity-50">
                            <span>"© 2026 UCI System"</span>
                            <span>"•"</span>
                            <span>{move || t(lang.get(), "footer_license")}</span>
                        </div>
                    </div>
                </footer>
            </div>
        </Router>
    }
}

#[component]
fn Protected(children: Children) -> impl IntoView {
    let navigate = use_navigate();

    create_effect(move |_| {
        if let Some(storage) = window().local_storage().ok().flatten() {
            if storage.get_item("uci_token").unwrap_or(None).is_none() {
                navigate("/login", Default::default());
            }
        }
    });

    children()
}
