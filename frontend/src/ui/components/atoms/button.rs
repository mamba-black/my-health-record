use leptos::{component, view, IntoView};
use web_sys::MouseEvent;

#[component]
pub fn SubmitButton(label: String) -> impl IntoView {
    view! {
        <button type="submit" class="button-submit">
            {label}
        </button>
    }
}

#[component]
pub fn ResetButton(label: String) -> impl IntoView {
    view! {
        <button type="reset" class="button-reset">
            {label}
        </button>
    }
}
