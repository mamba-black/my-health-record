use leptos::*;
use leptos_router::*;

use medical_front::domain::patient::Patient;
use medical_front::ui::components::route::private;
use medical_front::ui::components::route::public;
use medical_front::ui::pages::history_detail::HistoryDetail;
use medical_front::ui::pages::home::Home;
use medical_front::ui::pages::not_found::NotFound;
use medical_front::ui::pages::private_home::PrivateHome;
use medical_front::ui::pages::search::Search;

// use medical_front::components::route::{PrivateRoute, PublicRoute};
// use medical_front::{initCodeClient, CodeClientConfig};
//
#[component]
pub fn App() -> impl IntoView {
    // let signal = create_rw_signal(cx, AppState::default());
    // let (patient, patient_set) = create_signal(cx, Option::<Patient>::None);
    // let (patient, patient_set) = create_signal(cx, "Test");
    let a: RwSignal<Option<Patient>> = create_rw_signal(Option::<Patient>::None);
    provide_context(a);
    //     let app_context: UseReducerHandle<AppState> = use_reducer(|| AppState::default());

    //     html! {
    //     <ContextProvider <AppStateContext> context={app_context}>
    //       <BrowserRouter>
    //         <Switch<PublicRoute> render={public_switch} />
    //       </BrowserRouter>
    //     </ContextProvider <AppStateContext>>
    //     }
    view! {
        <Router>
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
            patient.id: {move || a.with(|x| {
                if x.is_some() {
                    x.clone().unwrap().name.clone()
                } else {
                    "None".to_string()
                }
            })}
        </Router>
    }
}

// fn public_switch(route: PublicRoute) -> Html {
//     match route {
//         PublicRoute::Home => html! { <Home /> },
//         PublicRoute::About => html! { <Home /> },
//         PublicRoute::PrivatesRoot | PublicRoute::Privates => {
//             html! { <PrivateSwitch /> }
//         }
//         PublicRoute::NotFound => html! { <NotFound /> },
//     }
// }
//
// #[function_component]
// fn PrivateSwitch() -> Html {
//     let a = gloo_storage::LocalStorage::get::<String>("access_token");
//     if a.is_ok() {
//         info!("token: {}", a.unwrap());
//         html! { <Switch<PrivateRoute> render={private_switch}/> }
//     } else {
//         info!("No hay token");
//         // initCodeClient(CodeClientConfig::new (
//         //     "937186309482-uc7dm6bc6o6p3disa546hq25n8dbov42.apps.googleusercontent.com".to_string(),
//         //     "https://www.googleapis.com/auth/calendar.readonly".to_string(),
//         //     "redirect".to_string(),
//         //     "https://your.domain/code_callback_endpoint".to_string(),
//         // ));
//         html! { <Switch<PrivateRoute> render={private_switch}/> }
//     }
//
//     // let auth = auth2::getAuthInstance();
//     // if auth.isSignedIn() {
//     //     info!("Esta logeado contra google");
//     // } else {
//     //     info!("Tienes que logearte contra google");
//     // }
//     // auth.signIn();
// }
//
// fn private_switch(route: PrivateRoute) -> Html {
//     match route {
//         PrivateRoute::Histories => html! { <Search /> },
//         PrivateRoute::HistoryDetail { id } => html! { <HistoryDetail id={id.clone()} /> },
//         PrivateRoute::NotFound | PrivateRoute::NotFound2 => html! { <NotFound /> },
//     }
// }
