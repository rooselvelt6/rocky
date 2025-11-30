use leptos::*;
use uci::frontend::app::App;
use wasm_bindgen::JsCast;

fn main() {
    // Initialize panic hook for better error messages in browser console
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Hello from Rust! Frontend is running.".into());

    // Mount the Leptos app to the #app div
    leptos::mount_to(
        leptos::document()
            .get_element_by_id("app")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap(),
        || view! { <App /> },
    )
}
