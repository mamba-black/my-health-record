use leptos::component;
use leptos::*;
use log::info;
use web_sys::{Event, SubmitEvent};

use crate::di::DI;
use crate::domain::patient::Patient;
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::{ResetButton, SubmitButton};
use crate::ui::components::molecules::checkbox::Checkbox;
use crate::ui::components::molecules::input::Input;

#[derive(Clone)]
pub(crate) enum EditState {
    ReadOnly(Patient),
    Edit(Patient),
}

#[component]
pub fn PatientDetail(patient: Patient) -> impl IntoView {
    let edit_status = create_rw_signal(EditState::ReadOnly(patient.clone()));
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
    let (phone, set_phone) = create_slice(
        patient_signal,
        |patient| patient.email.clone(),
        |patient, value| patient.email = value,
    );
    let (email, set_email) = create_slice(
        patient_signal,
        |patient| patient.email.clone(),
        |patient, value| patient.email = value,
    );
    let (birthday, set_birthday) = create_slice(
        patient_signal,
        |patient| patient.email.clone(),
        |patient, value| patient.email = value,
    );
    let (address, set_address) = create_slice(
        patient_signal,
        |patient| patient.other.clone(),
        |patient, value| patient.other = value,
    );
    let (district, set_district) = create_slice(
        patient_signal,
        |patient| patient.other.clone(),
        |patient, value| patient.other = value,
    );

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
                <div class="md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">

                    // ---------------------------------------------
                    <Input id="firstName".to_string()
                           name="Nombre".to_string()
                           value={first_name}
                           set_value={set_first_name}
                           readonly={read_only} />
                    <Input id="lastName".to_string()
                           name="Apellido Paterno".to_string()
                           value={last_name}
                           set_value={set_last_name}
                           readonly={read_only} />
                    <Input id="secondLastName".to_string()
                           name="Apellido Materno".to_string()
                           value={second_name}
                           set_value={set_second_name}
                           readonly={read_only} />

                    // ---------------------------------------------
                    <Input id="secondLastName".to_string()
                           name="Dirección".to_string()
                           value={address}
                           set_value={set_address}
                           class="col-span-2".to_string()
                           readonly={read_only} />
                    <Input id="district".to_string()
                           name="Distrito".to_string()
                           readonly={read_only}
                           value={email}
                           set_value={set_email}/>

                    // ---------------------------------------------
                    <Input id="phone".to_string()
                           name="Telefono".to_string()
                           readonly={read_only}
                           value={phone}
                           set_value={set_phone}
                           _type="tel".to_string() />
                    <Input id="email".to_string()
                           name="Correo electronico".to_string()
                           readonly={read_only}
                           value={email}
                           set_value={set_email}
                           _type="email".to_string() />
                    <Input id="birthday".to_string()
                           name="Fecha de cumpleaños".to_string()
                           readonly={read_only}
                           value={birthday}
                           set_value={set_birthday}
                           _type="date".to_string() />

                    // ---------------------------------------------
                    <div class="col-span-3">
                        <hr class="h-px mt-8" />
                    </div>
                    <h2 class="col-span-3">Antecedentes</h2>

                    // ---------------------------------------------
                    <Input id="alergy".to_string()
                           name="Alergias".to_string()
                           readonly={read_only}
                           value={birthday}
                           set_value={set_birthday}
                           class="col-span-3".to_string() />

                    // ---------------------------------------------
                    <Checkbox />
                    <Input id="".to_string() name="Hepatitis".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Diabetes".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Hemorragia".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Presion alta".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Presion baja".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Colesteros".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="Asma".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />
                    <Input id="".to_string() name="TBC".to_string() value={birthday} set_value={set_birthday} readonly={read_only} />

                    // ---------------------------------------------
                    //

                    // ---------------------------------------------
                    //

                    // ---------------------------------------------
                    <div class="col-span-2"></div>
                    <div class="justify-self-end">{edit_button}</div>
                </div>
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
