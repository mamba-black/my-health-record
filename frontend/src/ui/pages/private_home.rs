use crate::ui::components::organisms::header::MedicalHeader;

use leptos::prelude::*;
use leptos_router::components::Outlet;
use log::{debug, info};

#[component]
pub fn PrivateHome() -> impl IntoView {
    view! {
        <div class="min-h-full">
            <MedicalHeader />
            <Outlet />
        </div>
    }
}
