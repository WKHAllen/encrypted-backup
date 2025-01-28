//! Number slider UI component.

use crate::classes::*;
use crate::hooks::*;
use crate::number::*;
use dioxus::prelude::*;

/// A number slider component.
#[component]
pub fn Slider<N: Number + 'static>(
    /// The slider state.
    state: Signal<N>,
    /// The slider label.
    label: Option<String>,
    /// Optional info to display under the slider.
    info: Option<String>,
    /// The minimum value.
    #[props(default = N::NUMBER_MIN)]
    min: N,
    /// The maximum value.
    #[props(default = N::NUMBER_MAX)]
    max: N,
    /// The step size.
    #[props(default = N::NUMBER_STEP)]
    step: N,
    /// Whether the slider is disabled.
    #[props(default = false)]
    disabled: bool,
    /// An optional class name.
    class: Option<String>,
) -> Element {
    let id = use_id();
    let label = match label {
        Some(text) => format!("{}: {}", text, state()),
        None => format!("{}", state()),
    };
    let info = info.as_deref().unwrap_or_default();

    rsx! {
        div {
            class: classes!("slider-container", disabled.then_some("slider-container-disabled"), class),

            label {
                class: "slider-label",
                r#for: "{id}",
                "{label}"
            }

            input {
                id: "{id}",
                class: "slider",
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                disabled: disabled,
                value: "{state()}",
                oninput: move |event| {
                    state.set(event.value().parse().ok().unwrap());
                }
            }

            span {
                class: "info",
                "{info}"
            }
        }
    }
}
