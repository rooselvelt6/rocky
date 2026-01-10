use crate::frontend::i18n::t;
use crate::frontend::i18n::Language;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let lang = use_context::<Signal<Language>>().expect("Language context not found");

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 pb-24">
            <div class="text-center mb-12">
                <h1 class="text-3xl font-bold text-indigo-900 mb-2">
                    {move || t(lang.get(), "clinical_dashboard")}
                </h1>
                <p class="text-indigo-600">
                    {move || t(lang.get(), "dashboard_subtitle")}
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-5xl mx-auto">
                <A href="/register" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-indigo-600 transition-colors">
                        <i class="fas fa-user-plus text-indigo-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "register_patient")}</span>
                </A>

                <A href="/patients" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-blue-600 transition-colors">
                        <i class="fas fa-list text-blue-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "patient_list")}</span>
                </A>

                <A href="/ward" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-green-600 transition-colors">
                        <i class="fas fa-chart-line text-green-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "nav_monitor")}</span>
                </A>

                <A href="/glasgow" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-purple-600 transition-colors">
                        <i class="fas fa-brain text-purple-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "glasgow_scale")}</span>
                </A>

                <A href="/apache" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-red-600 transition-colors">
                        <i class="fas fa-heartbeat text-red-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "apache_ii")}</span>
                </A>

                <A href="/sofa" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-teal-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-teal-600 transition-colors">
                        <i class="fas fa-procedures text-teal-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "sofa_score")}</span>
                </A>

                <A href="/saps" class="flex flex-col items-center justify-center bg-white p-8 rounded-2xl shadow-xl border border-indigo-100 hover:border-indigo-300 hover:shadow-2xl transition-all group">
                    <div class="w-16 h-16 bg-orange-100 rounded-full flex items-center justify-center mb-4 group-hover:bg-orange-600 transition-colors">
                        <i class="fas fa-notes-medical text-orange-600 text-2xl group-hover:text-white"></i>
                    </div>
                    <span class="text-lg font-bold text-gray-800">{move || t(lang.get(), "saps_ii")}</span>
                </A>
            </div>
        </div>
    }
}
