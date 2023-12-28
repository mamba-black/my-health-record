use leptos::*;
use web_sys::Event;

#[component]
pub fn Input<F>(
    id: String,
    name: String,
    value: Signal<String>,
    set_value: SignalSetter<String>,
    readonly: F,
    #[prop(default = "text".to_string())] _type: String,
    #[prop(default = "".to_string())] class: String,
) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let on_input = move |e: Event| {
        let _value = event_target_value(&e);
        set_value(_value.clone());
    };

    view! {
        <div class=format!("{} history-input", class)>
            <label for={id.clone()}>
                {name.clone()}
            </label>
            <input
                id={id}
                type=_type
                placeholder={name}
                readonly={readonly}
                on:input=on_input
                prop:value={value}
                />
        </div>
    }
}
