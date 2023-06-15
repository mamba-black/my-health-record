use parse_display::Display;
use yew_router::prelude::*;

#[derive(Display, Clone, PartialEq, Routable, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/histories")]
    Histories,
    #[at("/about")]
    About,
    #[display("histories/{id}")]
    #[at("/histories/:id")]
    HistoryDetail { id: String },
}
