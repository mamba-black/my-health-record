#[cfg(debug_assertions)]
use console_error_panic_hook;
use leptos::*;
use medical_front::ui::app::App;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    #[cfg(debug_assertions)]
    init_log();

    info!("Cambiando a leptos");
    mount_to_body(|| view! { <App />})
}

use log::{debug, error, info, trace, warn, Level};
use serde::Deserialize;
use serde_qs as qs;
use std::str::FromStr;

pub fn init_log() {
    let query_string = window()
        .location()
        .search()
        .unwrap()
        .chars()
        .skip(1)
        .collect::<String>();
    let config = qs::from_str::<Config>(&query_string);

    println!("query string: {:?}", query_string);
    match config {
        Ok(Config { level }) => {
            let level = Level::from_str(&level).unwrap();
            let _ = console_log::init_with_level(level);
            info!("================================================>>>");
            info!("log level: [{:?}]", level);
        }
        _ => {
            let _ = console_log::init_with_level(Level::Warn);
        }
    }

    let min_level = [Level::Trace, Level::Debug, Level::Info];
    let level = log::max_level().to_level();

    if level.is_some_and(|level| min_level.contains(&level)) {
        trace!("== TEST: log trace");
        debug!("== TEST: log debug");
        info!("== TEST: log info");
        warn!("== TEST: log warn");
        error!("== TEST: log error");
        info!("== Log Iniciado");
        info!("================================================<<<");
    };
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
struct Config {
    level: String,
}
