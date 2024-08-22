use leptos::*;
use leptos_router::*;
use log::{debug, error, info};

use crate::di::DI;
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::SubmitButton;
use crate::ui::components::organisms::patient_consultation_list::PatientConsultationList;
use crate::ui::components::organisms::patient_detail::PatientDetail;

#[derive(Debug, Params, PartialEq)]
pub struct HistoryDetailParams {
    id: String,
}

#[component]
pub fn HistoryDetail() -> impl IntoView {
    debug!("HistoryDetail");

    // let app_state = DI.patient_service.get_app_status().clone();
    let id_params = use_params::<HistoryDetailParams>();
    debug!("Memo<id>: {:?}", id_params);
    let id = move || {
        id_params.with(move |id_param| {
            debug!("id_param: {:?}", id_param);
            match id_param {
                Ok(_id_param) => _id_param.id.to_string(),
                _ => {
                    error!("El id no se pudo parsear a string o es nulo");
                    "".to_string()
                }
            }
        })
    };

    view! {
        <>
        {move || {
            let id = id();
            let patient_opt = DI.patient_service.get_app_status().get();
            match patient_opt {
                Some(patient) if patient.id == id => {
                    let patient2 = patient.clone();
                    let full_name = format!("{} {} {}", patient.first_name, patient.last_name, patient.clone().second_name.unwrap_or("".to_string()));
                    view! {
                        <div>
                            <header class="bg-white shadow">
                                <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
                                    <h1 class="text-3xl font-bold tracking-tight text-gray-900">Ficha del Paciente: {full_name}</h1>
                                    <SubmitButton label="Historial".to_string() />
                                </div>
                            </header>
                            <main>
                                <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
                                    <div class="px-10 py-10 bg-white rounded-2xl">
                                        <PatientDetail patient/>
                                    </div>
                                </div>
                                <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
                                    <div class="px-10 py-10 bg-white rounded-2xl">
                                        <PatientConsultationList patient=patient2/>
                                    </div>
                                </div>
                            </main>
                        </div>
                    }
                },
                None if !id.is_empty() => {
                    // FIXME: Ver porque no esta funcianando
                    info!("patient is None, id: {} !!!", id);
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
                _ => {
                    let patient_opt_2 = patient_opt.clone();
                    let patient_opt_id = if patient_opt.is_some() { patient_opt.unwrap().id } else { "No existe id".to_string() };
                    info!("id: {}, patient_opt_id: {}", id, patient_opt_id);
                    view! {<div>Error al mostrar informacion del paciente: {format!("patient_opt: {:?}", patient_opt_2)}</div> }
                },
            }
        }}
        </>
    }
}
