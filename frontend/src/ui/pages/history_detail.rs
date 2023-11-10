use leptos::*;
use leptos_router::*;
use log::{debug, info};

use crate::di::DI;
use crate::services::patient_service::PatientService;
use crate::ui::components::organisms::patient_detail::PatientDetail;

#[derive(Params, PartialEq)]
pub struct HistoryDetailParams {
    id: usize,
}

#[component]
pub fn HistoryDetail() -> impl IntoView {
    debug!("HistoryDetail");

    let app_state = DI.patient_service.get_app_status().clone();
    let id_params = use_params::<HistoryDetailParams>();
    debug!("id: {:?}", id_params);
    let id = move || {
        id_params.with(move |id_param| match id_param {
            Ok(_id_param) => _id_param.id.to_string(),
            _ => "".to_string(),
        })
    };

    view! {
        <>
        {move || {
            let id = id();
            match app_state.get() {
                Some(patient) if patient.id == id => view! {
                    <div>
                        <header class="bg-white shadow">
                            <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
                                <h1 class="text-3xl font-bold tracking-tight text-gray-900">Historial del Paciente</h1>
                            </div>
                        </header>
                        <main>
                            <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
                                <div class="px-10 py-10 bg-white rounded-2xl">
                                    <PatientDetail patient/>
                                </div>
                            </div>
                        </main>
                    </div>
                },
                None if !id.is_empty() => {
                    info!("patient is None !!!");
                    // let id = id.clone();
                    spawn_local(async move {
                        DI.patient_service
                            .find_and_update_app_status(id)
                            .await;
                    });
                    // find_and_update_app_status();
                    view! {<div><h1>Cargar desde servicio</h1></div> }
                },
                None => view! {<div><h1>Seleccione un paciente</h1></div> },
                _ => view! {<div>Error al mostrar informacion del paciente</div> },
            }
        }}
        </>
    }
}
