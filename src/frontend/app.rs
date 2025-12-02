use crate::frontend::glasgow_form::GlasgowForm;
use crate::frontend::patient_form::PatientForm;
use leptos::*;
use leptos_router::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 font-sans">
                <nav class="bg-indigo-900 text-white shadow-lg">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div class="flex items-center justify-between h-16">
                            <div class="flex items-center">
                                <a href="/" class="text-xl font-bold tracking-wider">"UCI System"</a>
                                <div class="ml-10 flex items-baseline space-x-4">
                                    <A href="/" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">"Home"</A>
                                    <A href="/register" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">"Register Patient"</A>
                                    <A href="/glasgow" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-indigo-700 transition-colors">"Glasgow Scale"</A>
                                </div>
                            </div>
                        </div>
                    </div>
                </nav>

                <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                    <Routes>
                        <Route path="/" view=|| view! {
                            <div class="text-center mt-20">
                                <h1 class="text-4xl font-bold text-indigo-900 mb-4">"Welcome to UCI System"</h1>
                                <p class="text-xl text-indigo-600 mb-8">"Select an option from the menu to begin."</p>
                                <div class="flex justify-center space-x-4">
                                    <a href="/register" class="bg-indigo-600 text-white px-6 py-3 rounded-lg hover:bg-indigo-700 transition shadow-md">"Register New Patient"</a>
                                    <a href="/glasgow" class="bg-white text-indigo-600 px-6 py-3 rounded-lg hover:bg-gray-50 transition shadow-md border border-indigo-200">"Glasgow Assessment"</a>
                                </div>
                            </div>
                        }/>
                        <Route path="/register" view=PatientForm/>
                        <Route path="/glasgow" view=GlasgowForm/>
                    </Routes>
                </main>

                <footer class="text-center py-4 text-gray-500 text-sm">
                    <p>"Made with ❤️ for improving ICU care"</p>
                </footer>
            </div>
        </Router>
    }
}
