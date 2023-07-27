use yew::prelude::*;
use yew_router::prelude::*;

use medical_front::infrastructure::ui::app_state_context::{AppState, AppStateContext};
use medical_front::infrastructure::ui::pages::history_detail::HistoryDetail;
use medical_front::infrastructure::ui::pages::home::Home;
use medical_front::infrastructure::ui::pages::not_found::NotFound;
use medical_front::infrastructure::ui::pages::search::Search;
use medical_front::infrastructure::ui::route::{PrivateRoute, PublicRoute};

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
            html! { <Switch<PrivateRoute> render={private_switch}/> }
        }
        PublicRoute::NotFound => html! { <NotFound /> },
    }
}

fn private_switch(route: PrivateRoute) -> Html {
    match route {
        PrivateRoute::Histories => html! { <Search /> },
        PrivateRoute::HistoryDetail { id } => html! { <HistoryDetail {id} /> },
        PrivateRoute::NotFound | PrivateRoute::NotFound2 => html! { <NotFound /> },
    }
}
