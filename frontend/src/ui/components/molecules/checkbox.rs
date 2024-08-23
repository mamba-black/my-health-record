use leptos::{
    component, event_target_checked, event_target_value, view, IntoView, Signal, SignalGet,
    SignalSetter, SignalWith,
};
use log::info;
use rand::Rng;
use web_sys::Event;

#[component]
pub fn Checkbox<F>(
    #[prop(optional)] id: &'static str,
    name: &'static str,
    readonly: F,
    value: Signal<bool>,
    set_value: SignalSetter<bool>,
) -> impl IntoView
where
    F: Fn() -> bool + 'static + Clone,
{
    info!("Checkbox: {:?}", value.clone());
    let on_input = move |e: Event| {
        let _value = event_target_checked(&e);
        info!("Checkbox: {:?}", _value.clone());
        set_value(_value.clone());
    };
    let id = if id.is_empty() {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(1000..=9999);

        format!("input-{}-{}", name.clone(), random_number)
    } else {
        id.to_string()
    };

    let input_readonly = readonly.clone();
    let is300_readonly = readonly.clone();
    let is500_readonly = readonly;

    view! {
        <div class="history-checkbox flex items-center mb-4">
            <input id={id.clone()}
                disabled={input_readonly}
                type="checkbox"
                on:change=on_input
                prop:checked={value}
                class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600" />
            // text-gray-900 dark:text-gray-300
            <label
                for={id}
                class="ms-2 text-sm font-medium"
                class:text-gray-300={move || is300(is300_readonly(), value.get())}
                class:text-gray-500={move || is500(is500_readonly(), value.get())}>{name}</label>
        </div>
    }
}

fn is300(readonly: bool, value: bool) -> bool {
    if !readonly {
        false
    } else {
        !value
    }
}

fn is500(readonly: bool, value: bool) -> bool {
    if !readonly {
        !value
    } else {
        false
    }
}
