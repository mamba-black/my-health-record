use gloo_storage::Storage;
use log::info;
use yew::prelude::*;
use yew_router::prelude::*;
// use medical_front::auth2;

use medical_front::infrastructure::ui::app_state_context::{AppState, AppStateContext};
use medical_front::infrastructure::ui::pages::history_detail::HistoryDetail;
use medical_front::infrastructure::ui::pages::home::Home;
use medical_front::infrastructure::ui::pages::not_found::NotFound;
use medical_front::infrastructure::ui::pages::search::Search;
use medical_front::infrastructure::ui::route::{PrivateRoute, PublicRoute};
use medical_front::{initCodeClient, CodeClientConfig};

#[function_component]
pub fn App() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let app_context: UseReducerHandle<AppState> = use_reducer(|| AppState::default());

    html! {
    <ContextProvider <AppStateContext> context={app_context}>
      <BrowserRouter>
        <Switch<PublicRoute> render={public_switch} />
      </BrowserRouter>
    </ContextProvider <AppStateContext>>
    }
}

fn public_switch(route: PublicRoute) -> Html {
    match route {
        PublicRoute::Home => html! { <Home /> },
        PublicRoute::About => html! { <Home /> },
        PublicRoute::PrivatesRoot | PublicRoute::Privates => {
            html! { <PrivateSwitch /> }
        }
        PublicRoute::NotFound => html! { <NotFound /> },
    }
}

#[function_component]
fn PrivateSwitch() -> Html {
    let a = gloo_storage::LocalStorage::get::<String>("access_token");
    if a.is_ok() {
        info!("token: {}", a.unwrap());
        html! { <Switch<PrivateRoute> render={private_switch}/> }
    } else {
        info!("No hay token");
        // initCodeClient(CodeClientConfig::new (
        //     "937186309482-uc7dm6bc6o6p3disa546hq25n8dbov42.apps.googleusercontent.com".to_string(),
        //     "https://www.googleapis.com/auth/calendar.readonly".to_string(),
        //     "redirect".to_string(),
        //     "https://your.domain/code_callback_endpoint".to_string(),
        // ));
        html! { <Switch<PrivateRoute> render={private_switch}/> }
    }

    // let auth = auth2::getAuthInstance();
    // if auth.isSignedIn() {
    //     info!("Esta logeado contra google");
    // } else {
    //     info!("Tienes que logearte contra google");
    // }
    // auth.signIn();
}

fn private_switch(route: PrivateRoute) -> Html {
    match route {
        PrivateRoute::Histories => html! { <Search /> },
        PrivateRoute::HistoryDetail { id } => html! { <HistoryDetail id={id.clone()} /> },
        PrivateRoute::NotFound | PrivateRoute::NotFound2 => html! { <NotFound /> },
    }
}
