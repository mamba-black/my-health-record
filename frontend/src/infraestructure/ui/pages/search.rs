use log::info;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::domain::patient::Patient;
use crate::infraestructure::ui::app_state_context::AppStateContext;
use crate::infraestructure::ui::organisms::header::MedicalHeader;
use crate::infraestructure::ui::route::Route;

#[derive(Debug, Properties, PartialEq)]
struct GridProperties {
    patients: Vec<Patient>,
}

#[derive(Debug, Properties, PartialEq)]
struct SearchInputProperties {
    patient_handler: Callback<Vec<Patient>>,
}

#[function_component]
pub fn Search() -> Html {
    let patients_status = use_state(Vec::<Patient>::new);
    let patients = patients_status.to_vec();

    let patient_handler = Callback::from(move |patients: Vec<Patient>| {
        patients_status.set(patients);
    });

    html! {
    <div class="min-h-full">
      <MedicalHeader />

      <header class="bg-white shadow">
        <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 class="text-3xl font-bold tracking-tight text-gray-900">{"Busqueda de pacientes"}</h1>
        </div>
      </header>
      <main>
        <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
          <div class="px-10 py-10 bg-white rounded-2xl">
            <SearchInput {patient_handler} />
            <br/>
            <Grid {patients}/>
          </div>
        </div>
      </main>
    </div>
    }
}

#[function_component]
fn SearchInput(properties: &SearchInputProperties) -> Html {
    let patient_handler = properties.patient_handler.clone();

    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();

        let patients_vec: Vec<Patient> = vec![
            Patient::new("123".to_string(), "Leslie Alexander".to_string(), "leslie.alexander@example.com".to_string(), "Co-Founder / CEO".to_string(), false, "https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Michael Foster".to_string(), "michael.foster@example.com".to_string(), "Co-Founder / CEO".to_string(), false, "https://images.unsplash.com/photo-1519244703995-f4e0f30006d5?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Dries Vincent".to_string(), "dries.vincent@example.com".to_string(), "Business Relations".to_string(), false, "https://images.unsplash.com/photo-1506794778202-cad84cf45f1d?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Lindsay Walton".to_string(), "lindsay.walton@example.com".to_string(), "Front-end Developer".to_string(), false, "https://images.unsplash.com/photo-1517841905240-472988babdf9?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Courtney Henry".to_string(), "courtney.henry@example.com".to_string(), "Designer".to_string(), true, "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Tom Cook".to_string(), "email".to_string(), "empresa".to_string(), true, "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Tom Cook".to_string(), "email".to_string(), "empresa".to_string(), true, "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            Patient::new("123".to_string(), "Tom Cook".to_string(), "email".to_string(), "empresa".to_string(), true, "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
        ];
        patient_handler.emit(patients_vec);
    });

    html! {
    <form onsubmit={onsubmit}>
        <label for="default-search" class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white">{"Search"}</label>
        <div class="relative">
            <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                <svg aria-hidden="true" class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
            </div>
            <input type="search" id="default-search" class="block w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search Mockups, Logos..." required=true />
            <button type="submit" class="text-white absolute right-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">{"Busqueda"}</button>
        </div>
    </form>
    }
}


fn onclick(patient: &Patient, app_state_context: &AppStateContext) {
    info!("Paciente: {}", patient);
    app_state_context.dispatch(Some(patient.clone()));
}

#[function_component]
fn Grid(properties: &GridProperties) -> Html {
    let patients = &properties.patients;

    let app_state_context = use_context::<AppStateContext>().unwrap();

    html! {
    <ul role="list" class="divide-y divide-gray-100">
    {patients.iter().map(|patient| {
        let patient_grid = patient.clone();
        let patient_onclick = patient.clone();
        let app_state_context = app_state_context.clone();
        html!{
          <li onclick={move |_| {onclick(&patient_onclick, &app_state_context)}}>
            <Link<Route> classes="flex justify-between gap-x-6 py-5" to={Route::HistoryDetail{id: patient_grid.id}}>
             <div class="flex gap-x-4">
               <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src={patient_grid.avatar} alt="" />
               <div class="min-w-0 flex-auto">
                 <p class="text-sm font-semibold leading-6 text-gray-900">
                     {patient_grid.name}
                     <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                       <span  class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>
                       <span class="relative">{"Cita programada"}</span>
                     </span>
                 </p>
                 <p class="mt-1 truncate text-xs leading-5 text-gray-500">{patient_grid.email}</p>
               </div>
             </div>
             <div class="hidden sm:flex sm:flex-col sm:items-end">
               <p class="text-sm leading-6 text-gray-900">{patient_grid.other}</p>
               {
                 if patient_grid.online {html!{
                   <div class="mt-1 flex items-center gap-x-1.5">
                     <div class="flex-none rounded-full bg-emerald-500/20 p-1">
                       <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
                     </div>
                     <p class="text-xs leading-5 text-gray-500">{"Online"}</p>
                   </div>
                 }}
                 else {
                    html!{
                     <p class="mt-1 text-xs leading-5 text-gray-500">{"Last seen "}<time datetime="2023-01-23T13:23Z">{"3h ago"}</time></p>
                   }
                 }
               }
             </div>
            </Link<Route>>
          </li>
        }
    }).collect::<Html>()}
    </ul>
    }
}
