use leptos::*;
use leptos_router::*;
use log::{debug, info};
use web_sys::{Event, SubmitEvent};

use crate::di::DI;
use crate::domain::patient::Patient;
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::{ResetButton, SubmitButton};

#[derive(Params, PartialEq)]
pub struct HistoryDetailParams {
    id: usize,
}

#[component]
pub fn HistoryDetail() -> impl IntoView {
    debug!("HistoryDetail");

    let app_state = DI.patient_service.get_app_status().clone();
    let id_params = use_params::<HistoryDetailParams>();
    // let id = move || id_params.with(|id| id.clone());
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

#[derive(Clone)]
pub(crate) enum EditState {
    ReadOnly,
    Edit,
}

#[component]
fn PatientDetail(patient: Patient) -> impl IntoView {
    let name = patient.name.clone();

    let (edit_status, set_edit_status) = create_signal(EditState::ReadOnly);
    let read_only = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly => true,
            EditState::Edit => false,
        })
    };
    let edit_button = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly => view! {
                <div><SubmitButton label="Editar".to_string() /></div>
            },
            EditState::Edit => view! {
                <div class="space-x-3">
                    <SubmitButton label="Guardar".to_string() />
                    <ResetButton label="Cancelar".to_string() />
                </div>
            },
        })
    };

    view! {
        <div class="lg:wa-7/12 lg:justify-around">
            <form
                on:reset=move |e| reset_handle(e, edit_status, set_edit_status)
                on:submit=move |e| submit_handle(e, edit_status, set_edit_status, patient.clone())>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mr-2 md:mb-0 md:w-1/3">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="firstName">
                            Nombre
                        </label>
                        <input
                            class="w-full px-3 py-2 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="firstName"
                            type="text"
                            placeholder="Nombre"
                            readonly={read_only}
                            value={name}/>
                    </div>
                    <div class="mb-4 md:mr-2 md:ml-2 md:w-1/3">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="lastName">
                          Apellido Paterno
                        </label>
                        <input
                            class="w-full px-3 py-2 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="lastName"
                            type="text"
                            readonly={read_only}
                            value={""}
                            placeholder="Apellido Paterno"/>
                    </div>
                    <div class="md:ml-2 md:w-1/3">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="secondLastName">
                            Apellido Materno
                        </label>
                        <input
                            class="w-full px-3 py-2 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="secondLastName"
                            type="text"
                            readonly={read_only}
                            value={""}
                            placeholder="Apellido Materno"/>
                    </div>
                </div>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mb-0 md:w-2/3">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="address">
                            Dirección
                        </label>
                        <input
                            class="w-full px-3 py-2 mb-3 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="address"
                            type="text"
                            readonly={read_only}
                            value={""}
                            placeholder="Dirección"/>
                    </div>
                    <div class="md:ml-5 md:w-1/3">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="email">
                            Email
                        </label>
                        <input
                            class="w-full px-3 py-2 mb-3 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="email"
                            type="email"
                            readonly={read_only}
                            value={""}
                            placeholder="Email"/>
                    </div>
                </div>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mr-2 md:mb-0">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="password">
                            Password
                        </label>
                        <input
                            class="w-full px-3 py-2 mb-3 text-sm leading-tight text-gray-700 border border-red-500 rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="password"
                            type="password"
                            readonly={read_only}
                            value={""}
                            placeholder="******************"/>
                        <p class="text-xs italic text-red-500">Please choose a password.</p>
                    </div>
                    <div class="md:ml-2">
                        <label class="block mb-2 text-sm font-bold text-gray-700" for="c_password">
                            Confirm Password
                        </label>
                        <input
                            class="w-full px-3 py-2 mb-3 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                            id="c_password"
                            type="password"
                            readonly={read_only}
                            value={""}
                            placeholder="******************"/>
                    </div>
                </div>
                <div class="mb-6 text-center md:flex md:flex-row-reverse relative h-10">
                    // {
                    //   match edit_state3 {
                    //     EditState::Read => view! {cx, <>},
                    //     EditState::Edit => view! {cx,
                    //       <button
                    //           class="w-full px-4 py-2 font-bold text-white bg-blue-500 rounded-full hover:bg-blue-700 focus:outline-none focus:shadow-outline mb-2 md:mb-0 md:ml-2 md:w-32"
                    //           // on:click={move |_| {edit_state3.set(EditState::Read);}}
                    //           type="button">
                    //         "Cancelar"
                    //       </button>
                    //     },
                    //     _ => view! {cx, <>},
                    //   }
                    // }
                    // <EditButton />
                    <div class="absolute right-2.5 ">{edit_button}</div>
                </div>
                <hr class="mb-6 border-t" />
            </form>
        </div>
    }
}

fn reset_handle(
    event: Event,
    edit_status: ReadSignal<EditState>,
    set_edit_status: WriteSignal<EditState>,
) {
    // event.prevent_default();
    info!("reset_handle: {:?}", event);
    set_edit_status(EditState::ReadOnly);

    // match edit_status.get() {
    //     EditState::ReadOnly => {set_edit_status(EditState::Edit)},
    //     EditState::Edit => {set_edit_status(EditState::ReadOnly)},
    // };
}

fn submit_handle(
    event: SubmitEvent,
    edit_status: ReadSignal<EditState>,
    set_edit_status: WriteSignal<EditState>,
    patient: Patient,
) {
    event.prevent_default();
    info!("submit_handle: {}", event.type_());

    match edit_status.get() {
        EditState::ReadOnly => set_edit_status(EditState::Edit),
        EditState::Edit => {
            spawn_local(async move {
                DI.patient_service.save(patient).await;
                set_edit_status(EditState::ReadOnly);
            });
        }
    };
}

#[component]
fn EditButton() -> impl IntoView {
    view! {
        <button
            // class="w-full px-4 py-2 font-bold text-white bg-red-500 rounded-full hover:bg-red-700 focus:outline-none focus:shadow-outline md:w-32"
            // bg-red-500 hover:bg-red-700
            class="w-full px-4 py-2 font-bold text-white rounded-full focus:outline-none focus:shadow-outline md:w-32"
            on:click={move |_| {
                  info!("EditState::Edit");
            //   // edit_state2.set(EditState::Edit);
            //   match edit_state1 {
            //     // EditState::Read => edit_state1.set(EditState::Edit),
            //     // EditState::Edit => edit_state1.set(EditState::Read),
            //   }
            }}
            type="button">
            // {edit_button}
            Editar
        </button>
    }
}
