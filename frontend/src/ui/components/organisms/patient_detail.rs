use leptos::component;
use leptos::*;
use log::info;
use web_sys::{Event, SubmitEvent};

use crate::di::DI;
use crate::domain::patient::Patient;
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::{ResetButton, SubmitButton};
use crate::ui::components::molecules::input::Input;

#[derive(Clone)]
pub(crate) enum EditState {
    ReadOnly(Patient),
    Edit(Patient),
}

#[component]
pub fn PatientDetail(patient: Patient) -> impl IntoView {
    let patient_signal = create_rw_signal(patient.clone());

    let (first_name, set_first_name) = create_slice(
        patient_signal,
        |patient| patient.full_name.clone(),
        |patient, value| patient.full_name = value,
    );
    let (last_name, set_last_name) = create_slice(
        patient_signal,
        |patient| patient.last_name.clone(),
        |patient, value| patient.last_name = value,
    );
    let (second_name, set_second_name) = create_slice(
        patient_signal,
        |patient| patient.second_name.clone(),
        |patient, value| patient.second_name = value,
    );
    let (email, set_email) = create_slice(
        patient_signal,
        |patient| patient.email.clone(),
        |patient, value| patient.email = value,
    );
    let (address, set_address) = create_slice(
        patient_signal,
        |patient| patient.other.clone(),
        |patient, value| patient.other = value,
    );

    let edit_status = create_rw_signal(EditState::ReadOnly(patient.clone()));

    let read_only = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly(_) => true,
            EditState::Edit(_) => false,
        })
    };

    let edit_button = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly(_) => view! {
                <div><SubmitButton label="Editar".to_string() /></div>
            },
            EditState::Edit(_) => view! {
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
                on:reset=move |e| reset_handle(e, edit_status, patient_signal)
                on:submit=move |e| submit_handle(e, edit_status, patient_signal)>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mr-2 md:mb-0 md:w-1/3">
                        <Input id="firstName".to_string()
                               name="Nombre".to_string()
                               value={first_name}
                               set_value={set_first_name}
                               readonly={read_only} />
                    </div>
                    <div class="mb-4 md:mr-2 md:ml-2 md:w-1/3">
                        <Input id="lastName".to_string()
                               name="Apellido Paterno".to_string()
                               value={last_name}
                               set_value={set_last_name}
                               readonly={read_only} />
                    </div>
                    <div class="md:ml-2 md:w-1/3">
                        <Input id="secondLastName".to_string()
                               name="Apellido Materno".to_string()
                               value={second_name}
                               set_value={set_second_name}
                               readonly={read_only} />
                    </div>
                </div>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mb-0 md:w-2/3">
                        <Input id="secondLastName".to_string()
                               name="Dirección".to_string()
                               value={address}
                               set_value={set_address}
                               readonly={read_only} />
                    </div>
                    <div class="md:ml-5 md:w-1/3">
                        <Input id="email".to_string()
                               name="Correo electronico".to_string()
                               readonly={read_only}
                               value={email}
                               set_value={set_email}
                               _type="email".to_string() />
                    </div>
                </div>
                <div class="mb-4 mx-auto md:flex">
                </div>
                <div class="mb-6 text-center md:flex md:flex-row-reverse relative h-10">
                    <div class="absolute right-2.5 ">{edit_button}</div>
                </div>
                <hr class="mb-6 border-t" />
            </form>
        </div>
    }
}

fn reset_handle(event: Event, edit_status: RwSignal<EditState>, patient_signal: RwSignal<Patient>) {
    event.prevent_default();
    let patient = match edit_status.get() {
        EditState::ReadOnly(patient) => patient,
        EditState::Edit(patient) => patient,
    };
    info!("reset_handle: {:?}", event);
    edit_status.set(EditState::ReadOnly(patient.clone()));
    patient_signal.set(patient);
}

fn submit_handle(event: SubmitEvent, edit_status: RwSignal<EditState>, patient: RwSignal<Patient>) {
    event.prevent_default();
    info!("submit_handle: {}", event.type_());

    match edit_status.get() {
        EditState::ReadOnly(patient) => edit_status.set(EditState::Edit(patient)),
        EditState::Edit(_) => {
            spawn_local(async move {
                let patient = patient.get();
                DI.patient_service.save(patient.clone()).await;
                edit_status.set(EditState::ReadOnly(patient));
            });
        }
    };
}
