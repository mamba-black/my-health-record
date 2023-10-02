use app::App;
use leptos::*;
use log::{info, Level};

mod app;

fn main() {
    console_log::init_with_level(Level::Debug).expect("TODO: panic message");

    info!("Cambiando a leptos");
    mount_to_body(|| view! { <App />})
}
