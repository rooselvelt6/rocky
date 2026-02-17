use leptos::prelude::*;
use olympus_client::App;

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
