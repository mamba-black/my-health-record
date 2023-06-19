use std::rc::Rc;

use display_json::DisplayAsJson;
use serde::Serialize;
use yew::prelude::*;

use crate::domain::patient::Patient;

#[derive(Serialize, DisplayAsJson, Clone, PartialEq, Default)]
pub struct AppState {
    pub patient: Option<Patient>
}

impl Reducible for AppState {
    type Action = Option<Patient>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        AppState { patient: action }.into()
    }
}

pub type AppStateContext = UseReducerHandle<AppState>;