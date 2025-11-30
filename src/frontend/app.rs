use crate::frontend::glasgow_form::GlasgowForm;
use leptos::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-8">
            <div class="max-w-4xl mx-auto">
                <header class="text-center mb-8">
                    <h1 class="text-4xl font-bold text-indigo-900 mb-2">
                        "UCI - ICU Medical Scales"
                    </h1>
                    <p class="text-lg text-indigo-600">
                        "Automation & Optimization Platform"
                    </p>
                </header>

                <main class="bg-white rounded-lg shadow-lg p-6">
                    <GlasgowForm />
                </main>

                <footer class="text-center mt-8 text-gray-600">
                    <p>"Made with ❤️ for improving ICU care"</p>
                </footer>
            </div>
        </div>
    }
}
