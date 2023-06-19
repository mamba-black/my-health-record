use yew::prelude::*;
use yew_router::prelude::*;

use health_record::infrastructure::ui::app_state_context::{AppState, AppStateContext};
use health_record::infrastructure::ui::pages::history_detail::HistoryDetail;
use health_record::infrastructure::ui::pages::home::Home;
use health_record::infrastructure::ui::pages::not_found::NotFound;
use health_record::infrastructure::ui::pages::search::Search;
use health_record::infrastructure::ui::route::Route;

#[function_component]
pub fn App() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    let app_context = use_reducer(|| AppState::default());

    html! {
    <ContextProvider <AppStateContext> context={app_context}>
      <BrowserRouter>
        <Switch<Route> render={switch} />
      </BrowserRouter>
    </ContextProvider <AppStateContext>>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Histories => html! { <Search /> },
        Route::HistoryDetail { id } => html! { <HistoryDetail {id} /> },
        _ => html! { <NotFound /> },
    }
}
