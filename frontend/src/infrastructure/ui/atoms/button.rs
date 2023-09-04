// #[derive(Properties, Clone, PartialEq)]
// pub struct FirstButtonProperties {
//     pub label: String,
// }

use leptos::{component, view, IntoView};

#[component]
// pub fn FirstButton(property: &FirstButtonProperties) -> Html {
pub fn FirstButton(label: String) -> impl IntoView {
    view! {
    <button type="submit" class="text-white absolute right-2.5 bottom-2.5 bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
        // {property.label.clone()}
        {label}
    </button>
    }
}
