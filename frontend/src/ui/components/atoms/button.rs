use leptos::{component, view, IntoView};
use web_sys::MouseEvent;

#[component]
pub fn SubmitButton(label: String) -> impl IntoView {
    view! {
        <button type="submit"
                class="h-9 text-white bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
            // {property.label.clone()}
            {label}
        </button>
    }
}

#[component]
pub fn ResetButton(label: String) -> impl IntoView {
    view! {
        <button type="reset"
                class="h-9 text-white bottom-2.5 bg-red-700 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 font-medium rounded-lg text-sm px-4 dark:bg-red-600 dark:hover:bg-red-700 dark:focus:ring-red-800">
            // {property.label.clone()}
            {label}
        </button>
    }
}
