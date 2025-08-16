use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec;

use dioxus::html::{image, FileEngine};
use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::*;
use dioxus::signals::Signal;

use crate::server;
use crate::server::AuthenticatedRequest;

#[derive(PartialEq, Clone, Props)]
pub struct ImageUploadButtonProps {
    thumbnails: Signal<BTreeMap<u32, u32>>,
    multiple: Option<bool>,
}

#[component]
pub fn ImageUploadButton(props: ImageUploadButtonProps) -> Element {
    let mut thumbnails = props.thumbnails.clone();
    let mut is_loading = use_signal(|| false);

    let allow_multiple = props.multiple.unwrap_or(false);

    let read_files = move |file_engine: Arc<dyn FileEngine>| async move {
        let files = file_engine.files();
        let mut file_contents: Vec<Vec<u8>> = Vec::new();
        file_contents.reserve(files.len());

        for file_name in &files {
            if let Some(contents) = file_engine.read_file(file_name).await {
                info!("Read {} {}", file_name, contents.len());
                file_contents.push(contents);
            } else {
                info!("Failed to read {}", file_name);
            }
        }

        if !file_contents.is_empty() {
            is_loading.set(true);
            let resp = server::upload_images(AuthenticatedRequest {
                data: file_contents,
            })
            .await;
            is_loading.set(false);
            info!("Server responded with {:#?}", resp);

            match resp {
                Ok(mut images) => {
                    let mut image_ids: Vec<_> = images.iter().map(|(a, _)| a.clone()).collect();
                    images.into_iter().for_each(|(image, variant)| {
                        thumbnails.write().insert(image, variant);
                    });
                    info!("Image upload completed");
                }
                Err(e) => {}
            }
        }
    };
    let upload_file = move |evt: FormEvent| async move {
        if let Some(file_engine) = evt.files() {
            read_files(file_engine).await;
        }
    };
    rsx! {
        div {
            class: "imageuploadcontainer",
            label {
                class: if *is_loading.read() { "loading"},
                for: "fileupload",
                div {}
                div {}
                span { class: "loader"}
            }
            input {
                disabled: if *is_loading.read() {"true"},
                id:"fileupload",
                type:"file",
                accept: "image/*",
                multiple: if allow_multiple { true },
                onchange: upload_file
            }

        }
    }
}
