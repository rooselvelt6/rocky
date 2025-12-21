use crate::frontend::apache_form::ApacheForm;
use crate::frontend::glasgow_form::GlasgowForm;
use crate::frontend::i18n::{t, Language};
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
    // ... existing code ...

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

    view! {
        <Router>
            <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 font-sans">
                <nav class="bg-indigo-900 text-white shadow-lg">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div class="flex items-center justify-between h-16">
                            <div class="flex items-center">
                                <A href="/" class="text-xl font-bold tracking-wider">"UCI System"</A>
                                <div class="ml-10 flex items-baseline space-x-4">
                                    <A href="/patients" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        {move || t(lang.get(), "patient_list")}
                                    </A>
                                    <A href="/register" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        {move || t(lang.get(), "register_patient")}
                                    </A>
                                    <A href="/glasgow" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        {move || t(lang.get(), "glasgow_scale")}
                                    </A>
                                    <A href="/apache" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        {move || t(lang.get(), "apache_ii")}
                                    </A>
                                    <A href="/sofa" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        {move || t(lang.get(), "sofa_score")}
                                    </A>
                                    <A href="/saps" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">
                                        "SAPS II"
                                    </A>
                                    <A href="/ward" class="px-3 py-2 rounded-md text-sm font-medium bg-indigo-800 text-green-300 hover:bg-indigo-700 hover:text-green-200 transition-colors border border-green-500/30 flex items-center gap-2">
                                        <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                                        "Ward Monitor"
                                    </A>
                                </div>
                            </div>
                            <div>
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

                <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                    <Routes>
                        <Route path="/" view=move || view! {
                            <div class="text-center mt-20">
                                <h1 class="text-4xl font-bold text-indigo-900 mb-4">{move || t(lang.get(), "welcome_title")}</h1>
                                <p class="text-xl text-indigo-600 mb-8">{move || t(lang.get(), "welcome_subtitle")}</p>
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 max-w-5xl mx-auto px-4">
                                    <A href="/register" class="bg-indigo-600 text-white px-6 py-4 rounded-lg hover:bg-indigo-700 transition shadow-md">
                                        <i class="fas fa-user-plus text-2xl mb-2"></i>
                                        <div class="font-bold">{move || t(lang.get(), "register_patient")}</div>
                                    </A>
                                    <A href="/glasgow" class="bg-purple-600 text-white px-6 py-4 rounded-lg hover:bg-purple-700 transition shadow-md">
                                        <i class="fas fa-brain text-2xl mb-2"></i>
                                        <div class="font-bold">{move || t(lang.get(), "glasgow_scale")}</div>
                                    </A>
                                    <A href="/apache" class="bg-red-600 text-white px-6 py-4 rounded-lg hover:bg-red-700 transition shadow-md">
                                        <i class="fas fa-heartbeat text-2xl mb-2"></i>
                                        <div class="font-bold">{move || t(lang.get(), "apache_ii")}</div>
                                    </A>
                                    <A href="/sofa" class="bg-teal-600 text-white px-6 py-4 rounded-lg hover:bg-teal-700 transition shadow-md">
                                        <i class="fas fa-procedures text-2xl mb-2"></i>
                                        <div class="font-bold">{move || t(lang.get(), "sofa_score")}</div>
                                    </A>
                                     <A href="/saps" class="bg-orange-600 text-white px-6 py-4 rounded-lg hover:bg-orange-700 transition shadow-md">
                                        <i class="fas fa-notes-medical text-2xl mb-2"></i>
                                        <div class="font-bold">"SAPS II"</div>
                                    </A>
                                </div>
                            </div>
                        }/>
                        <Route path="/register" view=PatientForm/>
                        <Route path="/patients" view=PatientList/>
                        <Route path="/patients/:id" view=PatientDetail/>
                        <Route path="/glasgow" view=GlasgowForm/>
                        <Route path="/apache" view=ApacheForm/>
                        <Route path="/sofa" view=SofaForm/>
                        <Route path="/saps" view=SapsForm/>
                        <Route path="/ward" view=WardView/>
                    </Routes>
                </main>

                <footer class="text-center py-4 text-gray-500 text-sm">
                    <p>{move || t(lang.get(), "made_with_love")}</p>
                </footer>
            </div>
        </Router>
    }
}
