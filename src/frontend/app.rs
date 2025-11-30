use crate::frontend::glasgow_form::GlasgowForm;
use leptos::*;

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="app-container">
            <header>
                <h1>"UCI - ICU Medical Scales"</h1>
                <p>"Automation & Optimization Platform"</p>
            </header>

            <main>
                <GlasgowForm />
            </main>

            <footer>
                <p>"Made with ❤️ for improving ICU care"</p>
            </footer>
        </div>
    }
}
