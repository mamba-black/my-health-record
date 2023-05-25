use parse_display::Display;
use yew_router::prelude::*;

#[derive(Display, Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/histories")]
    Histories,
    #[at("/about")]
    About,
}

