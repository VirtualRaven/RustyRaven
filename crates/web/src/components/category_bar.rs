use dioxus::prelude::*;
#[component]
pub fn CategoryBar(path: ReadOnlySignal<Vec<String>> ) -> Element {
    rsx! {
        div 
        {
            class: "category-bar",
            for (i,segment) in path().iter().enumerate() 
            {
                span { a {

                    Link {
                        to: crate::Route::ProductPage { segments: path().clone().into_iter().take(i+1).collect() },
                        "{segment}"
                    }
                } }
            }

        }
    }
}