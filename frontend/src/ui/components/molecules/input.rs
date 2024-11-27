use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use rand::Rng;
use web_sys::Event;

#[component]
pub fn Input<F>(
    #[prop(optional)] id: &'static str,
    name: &'static str,
    value: Signal<String>,
    set_value: SignalSetter<String>,
    readonly: F,
    #[prop(default = "text")] _type: &'static str,
    #[prop(optional)] class: &'static str,
) -> impl IntoView
where
    F: Fn() -> bool + Send + 'static,
{
    let on_input = move |e: Event| {
        let _value = event_target_value(&e);
        set_value(_value.clone());
    };

    let id = if id.is_empty() {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(1000..=9999);

        format!("input-{}-{}", name, random_number)
    } else {
        id.to_string()
    };

    view! {
        <div class=format!("{} history-input", class)>
            <label for={id.clone()}>
                {name}
            </label>
            <input
                id={id}
                type=_type
                placeholder={name}
                disabled={readonly}
                on:input=on_input
                prop:value={value}
                />
        </div>
    }
}
