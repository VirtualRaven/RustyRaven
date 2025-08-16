use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct CloseButtonProps {
    onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn CloseButton(props: CloseButtonProps) -> Element {
    rsx! {
        div {
            onclick: move |e|  props.onclick.call(e),
            class: "close-container",
            div {class: "leftright"}
            div {class: "rightleft"}
        }
    }
}
