use leptos::*;
use web_sys::Event;

#[component]
pub fn Input<F>(
    id: String,
    name: String,
    value: RwSignal<String>,
    readonly: F,
    #[prop(default = "text".to_string())] _type: String,
) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let on_input = move |e: Event| {
        let _value = event_target_value(&e);
        value.update(|mut v| *v = _value.clone());
    };

    view! {
        <label class="block mb-2 text-sm font-bold text-gray-700" for={id.clone()}>
            {name.clone()}
        </label>
        <input
            class="w-full px-3 py-2 text-sm leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
            id={id}
            type=_type
            placeholder={name}
            readonly={readonly}
            on:input=on_input
            prop:value={value.read_only()}
            />
    }
}
