use parse_display::Display;
use yew_router::prelude::*;

#[derive(Display, Clone, PartialEq, Routable, Debug)]
pub enum PublicRoute {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/p")]
    PrivatesRoot,
    #[at("/p/*")]
    Privates,
    #[at("/*")]
    NotFound,
}

#[derive(Display, Clone, PartialEq, Routable, Debug)]
pub enum PrivateRoute {
    #[at("/p/histories")]
    Histories,
    #[display("/p/histories/{id}")]
    #[at("/p/histories/:id")]
    HistoryDetail { id: String },
    #[at("/p")]
    NotFound,
    #[at("/p/*")]
    NotFound2,
}
