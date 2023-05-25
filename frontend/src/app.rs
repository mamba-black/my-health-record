use yew::prelude::*;
use yew_router::prelude::*;
use log::info;
use health_record::infraestructure::ui::route::Route;
use health_record::infraestructure::ui::pages::home::Home;
use health_record::infraestructure::ui::pages::search::Search;



#[function_component(App)]
pub fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
    <BrowserRouter>
      <Switch<Route> render={switch} />
    </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Histories => html! { <Search /> },
        _ => {
            info!("route: {}", route);
            html! { <h1> {"NO SE ENCUENTRA"} </h1> }
        }
    }
}

