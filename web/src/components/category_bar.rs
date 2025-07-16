use dioxus::prelude::*;
#[component]
pub fn CategoryBar(path: ReadOnlySignal<Vec<String>> ) -> Element {
    rsx! {
        div 
        {
            class: "category-bar",
            for segment in path() 
            {
                span { "{segment}" }
            }

        }
    }
}