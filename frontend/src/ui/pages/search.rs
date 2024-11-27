// use leptos::tracing::instrument::WithSubscriber;
// use leptos::prelude::*;
use leptos::either::Either;
use leptos::prelude::{
    create_signal, event_target_value, ClassAttribute, For, Get, GlobalAttributes, OnAttribute,
    PropAttribute, ReadSignal, With, WithUntracked, WriteSignal,
};
use leptos::task::spawn_local;
use leptos::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{use_location, use_navigate, use_query};
use leptos_router::params::Params;
use leptos_router::*;
use log::*;
use std::ops::Add;
use web_sys::{MouseEvent, SubmitEvent};

use crate::di::DI;
use crate::domain::patient::Patient;
use crate::services::patient_service::PatientService;
use crate::ui::components::atoms::button::SubmitButton;
use crate::ui::components::route::private;

#[derive(Debug, Params, PartialEq)]
struct HistoryDetailQueries {
    name: String,
}

#[component]
pub fn Search() -> impl IntoView {
    debug!("Component Search");
    let patients: Vec<Patient> = vec![];
    let (patients, set_patients) = create_signal(patients);

    view! {
        <header class="bg-white shadow">
            <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
                <h1 class="text-3xl font-bold tracking-tight text-gray-900">Busqueda de pacientes</h1>
            </div>
        </header>
        <main>
            <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
                <div class="px-10 py-10 bg-white rounded-2xl">
                    <SearchInput set_patients />
                    <br/>
                    <Grid patients />
                </div>
            </div>
        </main>
    }
}

fn find_patient(
    event: Option<SubmitEvent>,
    patient_name: String,
    set_patients: WriteSignal<Vec<Patient>>,
) {
    let location = use_location();
    let navigate = use_navigate();

    info!("patient_name: {}", patient_name);

    let mut url = location
        .pathname
        .with_untracked(|e| e.clone())
        .to_string()
        .add("?");

    let query = location.query.with_untracked(|q| q.clone());

    for (key, value) in query {
        if key == "name" {
            url = url.add(format!("{}={}&", key, patient_name.clone()).as_str());
        } else {
            url = url.add(format!("{}={}&", key, value).as_str());
        }
    }

    _ = navigate(url.as_str(), Default::default());

    spawn_local(async move {
        if let Some(event) = event {
            event.prevent_default();
        }

        let patients_response = DI
            .patient_service
            .search_patient(patient_name.to_string())
            .await;
        match patients_response {
            Ok(patients) => set_patients(patients),
            Err(_) => {}
        };
    });
}

#[component]
fn SearchInput(set_patients: WriteSignal<Vec<Patient>>) -> impl IntoView {
    let queries = use_query::<HistoryDetailQueries>();

    let (patient_name, patient_name_set) = create_signal("".to_string());

    queries.with_untracked(move |q| match q {
        Ok(queries) => {
            find_patient(None, queries.name.clone(), set_patients);
            patient_name_set(queries.name.clone());
            ()
        }
        Err(_) => (),
    });

    let name_get = move || {
        debug!("Memo<Result<Queries>>: {:?}", queries);
        let name = queries.with(move |q| match q {
            Ok(queries) => queries.name.clone(),
            Err(_) => "".to_string(),
        });
        debug!("patient_name: {}", name);
        name
    };

    view! {
        <form on:submit=move |event| find_patient(Some(event), patient_name.get(), set_patients)>
            <label for="default-search" class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white">Search</label>
            <div class="relative items-center h-14">
                <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                    <svg aria-hidden="true" class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                </div>
                <input
                    type="search"
                    id="default-search"
                    class="w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                    placeholder="Busqueda de pacientes, nombre..."
                    required=true
                    on:input=move |e| patient_name_set(event_target_value(&e).clone())
                    prop:value=name_get
                    />
                <span class="absolute right-2.5 pt-2"><SubmitButton label={"Busqueda".to_string()} /></span>
            </div>
        </form>
    }
}

#[component]
fn Grid(patients: ReadSignal<Vec<Patient>>) -> impl IntoView {
    let patient_onclick = |event: MouseEvent, patient: &Patient| {
        event.prevent_default();
        debug!("Paciente: {}", patient);

        DI.patient_service.update_app_status(patient.clone());

        let path = format!(
            "{}/{}",
            private::PRIVATE,
            private::HISTORY_DETAIL.replace(":id", &patient.id)
        );

        let navigate = use_navigate();
        _ = navigate(path.as_str(), Default::default());
    };

    view! {
        <ul role="list" class="divide-y divide-gray-100">
            <For
                each=patients
                key=|patient| patient.id.clone()
                children= move |patient| {
                    let patient1 = patient.clone();
                    let full_name = patient.full_name();
                    view!{
                        <li>
                            <a on:click=move |e| {patient_onclick(e, &patient1)}
                                class="flex justify-between gap-x-6 p-5 m-2 border rounded-lg border-transparent hover:border-blue-500 hover:bg-sky-100 hover:shadow-lg hover:cursor-pointer">
                                // href={private::HISTORY_DETAIL.replace(":id", &patient.id)}>
                                <div class="flex gap-x-4">
                                    <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src={&patient.avatar.unwrap_or("".to_string())} alt="" />
                                    <div class="min-w-0 flex-auto">
                                        <p class="text-sm font-semibold leading-6 text-gray-900">
                                            {full_name}
                                            <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                                                <span  class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>
                                                <span class="relative">Cita programada</span>
                                            </span>
                                        </p>
                                        <p class="mt-1 truncate text-xs leading-5 text-gray-500">{patient.email}</p>
                                    </div>
                                </div>
                                <div class="hidden sm:flex sm:flex-col sm:items-end">
                                    <p class="text-sm leading-6 text-gray-900">{}</p>
                                    {
                                        if patient.online {Either::Left(view!{<>
                                            <div class="mt-1 flex items-center gap-x-1.5">
                                                <div class="flex-none rounded-full bg-emerald-500/20 p-1">
                                                    <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
                                                </div>
                                                <p class="text-xs leading-5 text-gray-500">Online</p>
                                            </div>
                                        </>})} else {Either::Right(view!{<>
                                            <p class="mt-1 text-xs leading-5 text-gray-500">"Last seen "<time datetime="2023-01-23T13:23Z">3h ago</time></p>
                                        </>})}
                                    }
                                </div>
                            </a>
                        </li>
                    }
                } />
        </ul>
    }
}
