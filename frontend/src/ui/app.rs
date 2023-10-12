use leptos::*;
use leptos_router::*;
use log::info;

use crate::di::DI;
use crate::services::patient_service::{Load, PatientService};
use crate::ui::components::route::private;
use crate::ui::components::route::public;
use crate::ui::pages::history_detail::HistoryDetail;
use crate::ui::pages::home::Home;
use crate::ui::pages::not_found::NotFound;
use crate::ui::pages::private_home::PrivateHome;
use crate::ui::pages::search::Search;

// use medical_front::components::route::{PrivateRoute, PublicRoute};
// use medical_front::{initCodeClient, CodeClientConfig};
//
#[component]
pub fn App() -> impl IntoView {
    let load = DI.patient_service.get_loading().clone();

    let loading = move || {
        load.with(|load| {
            info!("load: {:?}", load);
            match load {
                Load::None => { view! { <div/> } },
                Load::Loading => {
                    view! {
                        <div class="absolute bg-white bg-opacity-60 z-10 h-full w-full flex items-center justify-center">
                          <div class="flex items-center">
                            <span class="text-3xl mr-4">Loading</span>
                            <svg class="animate-spin h-5 w-5 text-gray-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                          </div>
                        </div>
                    }
                },
            }
        })
    };

    view! {
        <Router>
            {loading}
            <Routes>
                <Route path=public::HOME                view=Home/>
                <Route path=private::PRIVATE            view=PrivateHome>
                    <Route path=""                      view=NotFound/>
                    <Route path=private::HISTORIES      view=Search/>
                    <Route path=private::HISTORY_DETAIL view=HistoryDetail/>
                </Route>
                <Route path=public::NOT_FOUND           view=NotFound/>
                //PrivateRoute::HistoryDetail { id } => html! { <HistoryDetail id={id.clone()} /> },
            </Routes>
        </Router>
    }
}
