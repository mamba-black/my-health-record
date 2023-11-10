use leptos::component;
use leptos::tracing::instrument::WithSubscriber;
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
    ReadOnly,
    Edit,
}

#[component]
pub fn PatientDetail(patient: Patient) -> impl IntoView {
    let patient2 = patient.clone();
    // let patient_signal = create_rw_signal(patient.clone());

    let name = create_rw_signal(patient2.name.clone());
    let email = create_rw_signal(patient2.email.clone());
    let address = create_rw_signal("".to_string());

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
                on:submit=move |e| submit_handle(e, edit_status, set_edit_status, patient2.clone())>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mr-2 md:mb-0 md:w-1/3">
                        <Input id="firstName".to_string()
                               name="Nombre".to_string()
                               value={name}
                               readonly={read_only} />
                    </div>
                    <div class="mb-4 md:mr-2 md:ml-2 md:w-1/3">
                        <Input id="lastName".to_string()
                               name="Apellido Paterno".to_string()
                               value={email}
                               readonly={read_only} />
                    </div>
                    <div class="md:ml-2 md:w-1/3">
                        <Input id="secondLastName".to_string()
                               name="Apellido Materno".to_string()
                               value={name}
                               readonly={read_only} />
                    </div>
                </div>
                <div class="mb-4 mx-auto md:flex">
                    <div class="mb-4 md:mb-0 md:w-2/3">
                        <Input id="secondLastName".to_string()
                               name="Dirección".to_string()
                               value={address}
                               readonly={read_only} />
                    </div>
                    <div class="md:ml-5 md:w-1/3">
                        <Input id="email".to_string()
                               name="Correo electronico".to_string()
                               readonly={read_only}
                               value={email}
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
