use leptos::component;
use leptos::either::Either;
use leptos::html::HtmlElement;
use leptos::prelude::*;
use leptos::task::spawn_local;
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
    use chrono::NaiveDate;

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
        pub hepatitis: bool,
        pub diabetes: bool,
        pub hemorrhage: bool,
        pub high_pressure: bool,
        pub low_pressure: bool,
        pub cholesterol: bool,
        pub asthma: bool,
        pub tbc: bool,
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
                birthdate: patient.birthdate.format("%Y-%m-%d").to_string(),
                phone_number: patient.phone_number.unwrap_or_default(),
                email: patient.email.unwrap_or_default(),
                other: patient.other,
                online: patient.online,
                avatar: patient.avatar.unwrap_or_default(),
                street: street,
                district: district,
                allergies: patient.allergies,
                hepatitis: patient.hepatitis,
                diabetes: patient.diabetes,
                hemorrhage: patient.hemorrhage,
                high_pressure: patient.high_pressure,
                low_pressure: patient.low_pressure,
                cholesterol: patient.cholesterol,
                asthma: patient.asthma,
                tbc: patient.tbc,
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
                .birthdate(
                    patient_detail_dto
                        .birthdate
                        .parse::<NaiveDate>()
                        .expect("Error en el parseo de fecha"),
                )
                .phone_number(Some(patient_detail_dto.phone_number))
                .email(Some(patient_detail_dto.email))
                .other(patient_detail_dto.other)
                .online(patient_detail_dto.online)
                .avatar(Some(patient_detail_dto.avatar))
                .address(Some(address))
                .allergies(patient_detail_dto.allergies)
                .hepatitis(patient_detail_dto.hepatitis)
                .diabetes(patient_detail_dto.diabetes)
                .hemorrhage(patient_detail_dto.hemorrhage)
                .high_pressure(patient_detail_dto.high_pressure)
                .low_pressure(patient_detail_dto.low_pressure)
                .cholesterol(patient_detail_dto.cholesterol)
                .asthma(patient_detail_dto.asthma)
                .tbc(patient_detail_dto.tbc)
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
    let (allergies, set_allergies) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.allergies.clone().join(","),
        |patient_detail_dto, allergies: String| {
            patient_detail_dto.allergies = allergies.split(',').map(|s| s.to_string()).collect()
        },
    );
    let (hepatitis, set_hepatitis) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.hepatitis.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.hepatitis = value,
    );
    let (diabetes, set_diabetes) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.diabetes.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.diabetes = value,
    );
    let (hemorrhage, set_hemorrhage) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.hemorrhage.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.hemorrhage = value,
    );
    let (high_pressure, set_high_pressure) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.high_pressure.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.high_pressure = value,
    );
    let (low_pressure, set_low_pressure) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.low_pressure.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.low_pressure = value,
    );
    let (cholesterol, set_cholesterol) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.cholesterol.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.cholesterol = value,
    );
    let (asthma, set_asthma) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.asthma.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.asthma = value,
    );
    let (tbc, set_tbc) = create_slice(
        patient_signal,
        |patient_detail_dto| patient_detail_dto.tbc.clone(),
        |patient_detail_dto, value: bool| patient_detail_dto.tbc = value,
    );

    let read_only = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly(_) => true,
            EditState::Edit(_) => false,
        })
    };

    let edit_button = move || {
        edit_status.with(|status| match status {
            EditState::ReadOnly(_) => Either::Left(view! {
                <div><SubmitButton label="Editar".to_string() /></div>
            }),
            EditState::Edit(_) => Either::Right(view! {
                 <div class="space-x-3">
                    <SubmitButton label="Guardar".to_string() />
                    <ResetButton label="Cancelar".to_string() />
                </div>
            }),
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
                    <Input id="alergy"
                           name="Alergias"
                           readonly={read_only}
                           value={allergies}
                           set_value={set_allergies}
                           class="col-span-3" />

                    // ---------------------------------------------
                    <Checkbox name="Hepatitis" value={hepatitis} set_value={set_hepatitis} readonly={read_only} />
                    <Checkbox name="Diabetes" value={diabetes} set_value={set_diabetes} readonly={read_only} />
                    <Checkbox name="Hemorragia" value={hemorrhage} set_value={set_hemorrhage} readonly={read_only} />
                    <Checkbox name="Presion alta" value={high_pressure} set_value={set_high_pressure} readonly={read_only} />
                    <Checkbox name="Presion baja" value={low_pressure} set_value={set_low_pressure} readonly={read_only} />
                    <Checkbox name="Colesterol" value={cholesterol} set_value={set_cholesterol} readonly={read_only} />
                    <Checkbox name="Asma" value={asthma} set_value={set_asthma} readonly={read_only} />
                    <Checkbox name="TBC" value={tbc} set_value={set_tbc} readonly={read_only} />

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
