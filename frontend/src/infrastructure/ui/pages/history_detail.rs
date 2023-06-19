use yew::prelude::*;

use crate::domain::patient::Patient;
use crate::infrastructure::ui::app_state_context::AppStateContext;
use crate::infrastructure::ui::organisms::header::MedicalHeader;

#[derive(Properties, Clone, PartialEq)]
pub struct HistoryDetailProperties {
    pub id: String,
}

#[function_component]
pub fn HistoryDetail(properties: &HistoryDetailProperties) -> Html {
    let patient = use_context::<AppStateContext>()
        .and_then(|app_state_context| app_state_context.patient.to_owned());

    match patient {
        Some(patient) if patient.id == properties.id =>
            html! {
            <div class="min-h-full">
              <MedicalHeader />

              <header class="bg-white shadow">
                <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
                  <h1 class="text-3xl font-bold tracking-tight text-gray-900">{"Historial del Paciente"}</h1>
                </div>
              </header>
              <main>
                <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
                  <div class="px-10 py-10 bg-white rounded-2xl">
                    <PatientDetail {patient}/>
                  </div>
                </div>
              </main>
            </div>
            },
        _ => html! { <div>{"Seleccione un paciente"}</div> }
    }
}

#[derive(Properties, Clone, PartialEq)]
struct PatientDetailProperties {
    patient: Patient,
}

enum EditState {
    Read,
    Edit,
}

#[function_component]
fn PatientDetail(properties: &PatientDetailProperties) -> Html {
    let patient = &properties.patient;
    let name = patient.name.clone();

    let edit_state1 = use_state(|| EditState::Read);
    let edit_state2 = edit_state1.clone();
    let edit_state3 = edit_state1.clone();
    let (edit_button, read_only) = match *edit_state1 {
        EditState::Read => ("Editar", true),
        EditState::Edit => ("Guardar", false),
    };

    html! {
    <div class="lg:wa-7/12 lg:justify-around">
      <form>
        <div class="mb-4 mx-auto md:flex">
          <div class="mb-4 md:mr-2 md:mb-0 md:w-1/3">
            <label class="block mb-2 text-sm font-bold text-gray-700" for="firstName">
              {"Nombre"}
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
              {"Apellido Paterno"}
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
              {"Apellido Materno"}
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
              {"Dirección"}
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
              {"Email"}
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
              {"Password"}
            </label>
            <input
                class="w-full px-3 py-2 mb-3 text-sm leading-tight text-gray-700 border border-red-500 rounded shadow appearance-none focus:outline-none focus:shadow-outline"
                id="password"
                type="password"
                readonly={read_only}
                value={""}
                placeholder="******************"/>
            <p class="text-xs italic text-red-500">{"Please choose a password."}</p>
          </div>
          <div class="md:ml-2">
            <label class="block mb-2 text-sm font-bold text-gray-700" for="c_password">
              {"Confirm Password"}
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
        <div class="mb-6 text-center md:flex md:flex-row-reverse">
          {
            match *edit_state3 {
              EditState::Read => html! { },
              EditState::Edit => html! {
                <button
                    class="w-full px-4 py-2 font-bold text-white bg-blue-500 rounded-full hover:bg-blue-700 focus:outline-none focus:shadow-outline mb-2 md:mb-0 md:ml-2 md:w-32"
                    onclick={move |_| {edit_state3.set(EditState::Read);}}
                    type="button">
                  {"Cancelar"}
                </button>
              },
            }
          }
          <button
              class="w-full px-4 py-2 font-bold text-white bg-red-500 rounded-full hover:bg-red-700 focus:outline-none focus:shadow-outline md:w-32"
              onclick={move |_| {
                edit_state2.set(EditState::Edit);
                match *edit_state1 {
                  EditState::Read => edit_state1.set(EditState::Edit),
                  EditState::Edit => edit_state1.set(EditState::Read),
                }
              }}
              type="button">
            {edit_button}
          </button>
          </div>
          <hr class="mb-6 border-t" />
        </form>
    </div>
    }
}
