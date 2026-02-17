use leptos::mount_to_body;
use leptos::prelude::*;
use olympus_lib::App;

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
