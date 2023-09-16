use leptos::*;
use leptos_router::*;
use log::*;
use tonic_web_wasm_client::Client;
use web_sys::{MouseEvent, SubmitEvent};

use crate::api::patient_service_client::*;
use crate::api::*;
// use crate::components::app_state_context::AppStateContext;
use crate::components::atoms::button::FirstButton;
use crate::components::route::private;
use crate::di::DI;
use crate::domain::patient::Patient;

// use crate::components::route::PrivateRoute;

#[component]
pub fn Search() -> impl IntoView {
    let patients: Vec<Patient> = vec![];
    let (patients, patients_set) = create_signal(patients);

    view! {
    <>
      <header class="bg-white shadow">
        <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 class="text-3xl font-bold tracking-tight text-gray-900">Busqueda de pacientes</h1>
        </div>
      </header>
      <main>
        <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
          <div class="px-10 py-10 bg-white rounded-2xl">
            <SearchInput patients_set=patients_set />
            <br/>
            <Grid patients=patients />
          </div>
        </div>
      </main>
    </>
    }
}

fn build_client() -> PatientServiceClient<Client> {
    let wasm_client = Client::new("http://localhost:9000".to_string());
    PatientServiceClient::new(wasm_client)
}

#[component]
fn SearchInput(patients_set: WriteSignal<Vec<Patient>>) -> impl IntoView {
    let onsubmit = move |event: SubmitEvent| {
        spawn_local(async move {
            event.prevent_default();
            let mut client = build_client();
            let patient_response = client.search_patient(SearchPatientRequest {
                name: Some("Miuler".to_string()),
            });
            match patient_response.await {
                Ok(response) => {
                    let patients = response.into_inner().patients;

                    let patients_vec: Vec<Patient> = patients
                        .into_iter()
                        .map(|response| {
                            Patient::new(
                                response.id,
                                response.first_name,
                                response.email.unwrap_or("".to_string()),
                                response.note.unwrap_or("".to_string()),
                                false,
                                response.icon.unwrap_or("".to_string()),
                            )
                        })
                        .collect::<Vec<_>>();
                    patients_set(patients_vec);
                }
                Err(_) => {}
            };
        });
    };

    view! {
    <form on:submit=onsubmit>
        <label for="default-search" class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white">Search</label>
        <div class="relative">
            <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                <svg aria-hidden="true" class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
            </div>
            <input type="search" id="default-search" class="block w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search Mockups, Logos..." required=true />
            <FirstButton label={"Busqueda".to_string()} />
        </div>
    </form>
    }
}

#[component]
fn Grid(patients: ReadSignal<Vec<Patient>>) -> impl IntoView {
    let patient_onclick = |event: MouseEvent, patient: &Patient| {
        event.prevent_default();
        debug!("Paciente: {}", patient);

        DI.patient_service.search_patient(patient.clone());

        let path = format!(
            "{}/{}",
            private::PRIVATE,
            private::HISTORY_DETAIL.replace(":id", &patient.id)
        );
        // window().location().set_pathname(path.as_str());
        // provide_context(cx, AppState { patient: Some(patient.clone()) });

        let navigate = use_navigate();
        _ = navigate(path.as_str(), Default::default());
    };

    view! {
    <ul role="list" class="divide-y divide-gray-100">
        <For
            each=patients
            key=|patient| patient.id.clone()
            view= move |patient| {
                let patient1 = patient.clone();
                view! {
                      <li>
                        <a on:click=move |e| {patient_onclick(e, &patient1)}
                           class="flex justify-between gap-x-6 py-5">
                           // href={private::HISTORY_DETAIL.replace(":id", &patient.id)}>
                         <div class="flex gap-x-4">
                           <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src={patient.avatar} alt="" />
                           <div class="min-w-0 flex-auto">
                             <p class="text-sm font-semibold leading-6 text-gray-900">
                                 {patient.name}
                                 <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                                   <span  class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>
                                   <span class="relative">Cita programada</span>
                                 </span>
                             </p>
                             <p class="mt-1 truncate text-xs leading-5 text-gray-500">{patient.email}</p>
                           </div>
                         </div>
                         <div class="hidden sm:flex sm:flex-col sm:items-end">
                           <p class="text-sm leading-6 text-gray-900">{patient.other}</p>
                           {
                             if patient.online {view!{
                               <>
                               <div class="mt-1 flex items-center gap-x-1.5">
                                 <div class="flex-none rounded-full bg-emerald-500/20 p-1">
                                   <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
                                 </div>
                                 <p class="text-xs leading-5 text-gray-500">Online</p>
                               </div>
                               </>
                             }} else {
                                view!{
                                <>
                                 <p class="mt-1 text-xs leading-5 text-gray-500">"Last seen "<time datetime="2023-01-23T13:23Z">3h ago</time></p>
                                </>
                               }
                             }
                           }
                         </div>
                        </a>
                      </li>
                }
            }
        />
    </ul>
    }
}
