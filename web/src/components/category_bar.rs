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
                    href: { 
                        let mut path = path()
                        .iter()
                        .take(i+1)
                        .fold(String::from(sjf_api::product::PRODUCTS_PATH), |mut a,s | {
                            a.push('/');
                            a.push_str(s);
                            a
                        });
                        path.push('/');
                        path

                    },
                    "{segment}"
                } }
            }

        }
    }
}