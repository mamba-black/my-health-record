use app::App;
use leptos::*;
use log::{info, Level};

mod app;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console_log::init_with_level(Level::Debug).expect("TODO: panic message");

    info!("Cambiando a leptos");
    mount_to_body(|| view! { <App />})
}
