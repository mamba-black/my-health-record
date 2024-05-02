use leptos::component;
use leptos::*;
use log::info;
use web_sys::{Event, SubmitEvent};

use crate::di::DI;
use crate::domain::patient::{Address, AddressBuilder, Patient, PatientBuilder};
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::{ResetButton, SubmitButton};
use crate::ui::components::molecules::checkbox::Checkbox;
use crate::ui::components::molecules::input::Input;

mod dto {
    use crate::domain::patient::{AddressBuilder, Patient, PatientBuilder};

    #[derive(Clone, Debug, Default)]
    pub struct PatientDetailDTO {
        pub id: String,
        pub first_name: String,
        pub last_name: String,
        pub second_name: String,
        pub birthdate: String,
        pub phone_number: String,
        pub email: String,
        pub other: String,
        pub online: bool,
        pub avatar: String,
        pub street: String,
        pub district: String,
        pub allergies: Vec<String>,
    }

    impl From<Patient> for PatientDetailDTO {
        fn from(patient: Patient) -> Self {
            let street = patient
                .address
                .as_ref()
                .map(|address| address.street.clone().unwrap_or_default())
                .unwrap_or_default();
            let district = patient
                .address
                .as_ref()
                .map(|address| address.district.clone().unwrap_or_default())
                .unwrap_or_default();
            PatientDetailDTO {
                id: patient.id,
                first_name: patient.first_name,
                last_name: patient.last_name,
                second_name: patient.second_name.unwrap_or_default(),
                birthdate: patient.birthdate,
                phone_number: patient.phone_number.unwrap_or_default(),
                email: patient.email.unwrap_or_default(),
                other: patient.other,
                online: patient.online,
                avatar: patient.avatar.unwrap_or_default(),
                street: street,
                district: district,
                allergies: patient.allergies,
            }
        }
    }

    impl From<PatientDetailDTO> for Patient {
        fn from(patient_detail_dto: PatientDetailDTO) -> Self {
            let address = AddressBuilder::default()
                .street(Some(patient_detail_dto.street))
                .district(Some(patient_detail_dto.district))
                .build()
                .unwrap();
            PatientBuilder::default()
                .id(patient_detail_dto.id)
                .first_name(patient_detail_dto.first_name)
                .last_name(patient_detail_dto.last_name)
                .second_name(Some(patient_detail_dto.second_name))
                .birthdate(patient_detail_dto.birthdate)
                .phone_number(Some(patient_detail_dto.phone_number))
                .email(Some(patient_detail_dto.email))
                .other(patient_detail_dto.other)
                .online(patient_detail_dto.online)
                .avatar(Some(patient_detail_dto.avatar))
                .address(Some(address))
                .allergies(patient_detail_dto.allergies)
                .build()
                .unwrap()
        }
    }
}

#[derive(Clone)]
enum EditState {
    ReadOnly(Patient),
    Edit(Patient),
}

#[component]
pub fn PatientDetail(patient: Patient) -> impl IntoView {
    let edit_status = create_rw_signal(EditState::ReadOnly(patient.clone()));
    let patient_signal = create_rw_signal(dto::PatientDetailDTO::from(patient));

    let (first_name, set_first_name) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.first_name.clone(),
        |patient_detail_dto, value| patient_detail_dto.first_name = value,
    );
    let (last_name, set_last_name) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.last_name.clone(),
        |patient_detail_dto, value| patient_detail_dto.last_name = value,
    );
    let (second_name, set_second_name) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.second_name.clone(),
        |patient_detail_dto, value: String| patient_detail_dto.second_name = value,
    );
    let (phone, set_phone) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.phone_number.clone(),
        |patient_detail_dto, value| patient_detail_dto.phone_number = value,
    );
    let (district, set_district) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.district.clone(),
        |patient_detail_dto, value| patient_detail_dto.district = value,
    );

    let (email, set_email) = create_slice(
        patient_signal,
        |patient| patient.email.clone(),
        |patient, value: String| patient.email = value,
    );
    let (birthday, set_birthday) = create_slice(
        patient_signal,
        |patient| patient.birthdate.clone(),
        |patient, value| patient.birthdate = value,
    );
    let (street, set_street) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.street.clone(),
        |patient_detail_dto, street| patient_detail_dto.street = street,
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
                    <Input id="firstName"
                           name="Nombre"
                           value={first_name}
                           set_value={set_first_name}
                           readonly={read_only} />
                    <Input id="lastName"
                           name="Apellido Paterno"
                           value={last_name}
                           set_value={set_last_name}
                           readonly={read_only} />
                    <Input id="secondLastName"
                           name="Apellido Materno"
                           value={second_name}
                           set_value={set_second_name}
                           readonly={read_only} />

                    // ---------------------------------------------
                    <Input id="street"
                           name="Dirección"
                           value={street}
                           set_value={set_street}
                           class="col-span-2"
                           readonly={read_only} />
                    <Input id="district"
                           name="Distrito"
                           value={district}
                           set_value={set_district}
                           readonly={read_only} />

                    // ---------------------------------------------
                    <Input id="phone"
                           name="Telefono"
                           readonly={read_only}
                           value={phone}
                           set_value={set_phone}
                           _type="tel" />
                    <Input id="email"
                           name="Correo electronico"
                           readonly={read_only}
                           value={email}
                           set_value={set_email}
                           _type="email" />
                    <Input id="birthday"
                           name="Fecha de cumpleaños"
                           readonly={read_only}
                           value={birthday}
                           set_value={set_birthday}
                           _type="date" />

                    // ---------------------------------------------
                    <div class="col-span-3">
                        <hr class="h-px mt-8" />
                    </div>
                    <h2 class="col-span-3">Antecedentes</h2>

                    // ---------------------------------------------
                    // <Input id="alergy"
                    //        name="Alergias"
                    //        readonly={read_only}
                    //        value={birthday}
                    //        set_value={set_birthday}
                    //        class="col-span-3" />

                    // ---------------------------------------------
                    // <Checkbox name="Hepatitis" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Diabetes" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Hemorragia" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Presion alta" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Presion baja" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Colesteros" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="Asma" value={birthday} set_value={set_birthday} readonly={read_only} />
                    // <Checkbox name="TBC" value={birthday} set_value={set_birthday} readonly={read_only} />

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

fn reset_handle(
    event: Event,
    edit_status: RwSignal<EditState>,
    patient_signal: RwSignal<dto::PatientDetailDTO>,
) {
    event.prevent_default();
    let patient = match edit_status.get() {
        EditState::ReadOnly(patient) => patient,
        EditState::Edit(patient) => patient,
    };
    info!("reset_handle, event: {:?}, patient: {:?}", event, patient);
    edit_status.set(EditState::ReadOnly(patient.clone()));
    patient_signal.set(patient.into());
}

fn submit_handle(
    event: SubmitEvent,
    edit_status: RwSignal<EditState>,
    patient_detail_dto: RwSignal<dto::PatientDetailDTO>,
) {
    event.prevent_default();
    info!("submit_handle: {}", event.type_());

    match edit_status.get() {
        EditState::ReadOnly(patient) => edit_status.set(EditState::Edit(patient)),
        EditState::Edit(_) => {
            let patient_detail_dto = patient_detail_dto.get();
            let patient: Patient = patient_detail_dto.into();
            spawn_local(async move {
                DI.patient_service.save(patient.clone()).await;
                edit_status.set(EditState::ReadOnly(patient));
            });
        }
    };
}
