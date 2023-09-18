use leptos::{component, view, IntoView};

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="mt-2 ml-2 md:m-0 ">
            <h1 class="text-sm md:text-6xl md:font-bold">NO SE ENCUENTRA</h1>
        </div>
    }
}
