use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::*;

use crate::server::auth::AuthApiError;

#[component]
fn Register(user_name: ReadOnlySignal<String>) -> Element {
    #[derive(Clone)]
    enum State {
        Idle,
        TerminalChallenge,
        StartRegistration,
        Finishing,
        Created,
        Error(String),
    }
    let mut state = use_signal(|| State::Idle);
    let mut challenge = use_signal(|| String::from(""));

    let current_state = (*state.read()).clone();
    match current_state {
        State::Idle => rsx! {
            button {
                onclick: move |_| async move{
                    state.set(State::TerminalChallenge);
                    match crate::server::auth::terminal_challenge(user_name.read().clone()).await
                    {
                        Ok(()) => {

                        }
                        Err(e) => {
                            state.set(State::Error(e.to_string()));
                        }
                    }
                },
                "Skapa konto!"
            }
        },
        State::TerminalChallenge => rsx! {
            div {
                label {"Ange kod från administratör"}
                input {
                    r#type: "text",
                    oninput: move |e| {
                        challenge.set(e.data().value());
                    },
                    "{challenge}"
                }
                button {

                    onclick: move |_| async move {
                        state.set(State::StartRegistration);
                        use thiserror::Error;
                        #[derive(Error, Debug)]
                        pub enum RegisterError {
                            #[error("Server auth: {0}")]
                            ServerError(#[from] ServerFnError<AuthApiError> ),
                            #[error("Javascript error")]
                            JavascriptError
                        }

                        use crate::server::auth::*;
                        let mut register  = async || -> Result<(),RegisterError>  {
                           let r = start_registration(user_name.read().clone(), (*challenge.read()).clone()).await?;
                            let nav = web_sys::window().unwrap().navigator();
                            let options : web_sys::CredentialCreationOptions =  r.into();
                            let promise = nav.credentials().create_with_options(&options).map_err(|_| RegisterError::JavascriptError)?;
                            let p : web_sys::PublicKeyCredential =  wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| RegisterError::JavascriptError)?.into();
                            state.set(State::Finishing);
                            finish_registration(p.into()).await.map_err(|e| e.into())
                        };

                        match register().await
                        {
                            Ok(()) => {
                                state.set(State::Created);
                            },
                            Err(e) => {
                                state.set(State::Error(e.to_string()))
                            }
                        };




                    },
                    "Bekräfta kod"
                }
            }
        },
        State::StartRegistration => rsx! {
            button {
                class:"green",
                disabled: true,
                "Registrerar..."
            }
        },
        State::Finishing => rsx! {
            button {
                class:"green",
                disabled: true,
                "Avslutrar..."
            }
        },
        State::Created => rsx! {
            button {
                class:"green",
                disabled: true,
                "Konto registrerat"
            }
        },
        State::Error(e) => rsx! {
            button {
                class:"red",
                title: e,
                onclick: move |_| state.set(State::Idle),
                "Misslyckades"
            }
        },
    }
}

#[component]
fn Login(user_name: ReadOnlySignal<String>) -> Element {
    #[derive(Clone)]
    enum State {
        Idle,
        Pending,
        LoggedIn,
        Error(String),
    }

    let mut state = use_signal(|| State::Idle);

    let current_state = (*state.read()).clone();

    match current_state {
        State::Idle => rsx! {
            button {
                onclick: move |_| async move {

                        use thiserror::Error;
                        #[derive(Error, Debug)]
                        pub enum LoginError {
                            #[error("Server auth: {0}")]
                            ServerError(#[from] ServerFnError<AuthApiError> ),
                            #[error("Javascript error")]
                            JavascriptError
                        }

                    state.set(State::Pending);
                    let login = async || ->  Result<(), LoginError> {
                        let res = crate::server::auth::start_authentication(user_name.read().clone()).await?;
                        let nav = web_sys::window().unwrap().navigator();
                        let options : web_sys::CredentialRequestOptions =  res.into();
                        let promise = nav.credentials().get_with_options(&options).map_err(|_| LoginError::JavascriptError )?;
                        let p : web_sys::PublicKeyCredential = wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| LoginError::JavascriptError )?.into();
                        Ok(crate::server::auth::finish_authentication(p.into()).await?)
                    };
                    match login().await
                    {
                        Ok(()) => {
                            state.set(State::LoggedIn);
                            let nav = navigator();
                            nav.push( NavigationTarget::<crate::Route>::Internal(crate::Route::CategoryList {  }) );
                        },
                        Err(e) => {
                            state.set(State::Error(e.to_string()));
                            warn!("Login failed due to {}",e);
                        }
                    }

                },
                "Logga in"
            }
        },

        State::Pending => rsx! {
            button {
                disabled: true,
                "Loggar in..."
             }
        },
        State::LoggedIn => rsx! {
            button {
                disabled: true,
                "Inloggad!"
             }
        },
        State::Error(e) => rsx! {
            button {
                title: e,
                class: "red",
                onclick: move |_| {state.set(State::Idle); },
                "Misslyckades"
             }
        },
    }
}

#[component]
pub fn Auth() -> Element {
    let mut user_name = use_signal(|| String::from(""));

    rsx! {
        document::Link { rel: "stylesheet", href: super::category::ADMIN_CSS }
        div {
            class: "login",
            div {
                h2 {
                    "Inloggning"
                }
            }
            div {
                label {
                    for: "username",
                    "Användarnamn"
                }
                input {
                    id:"username",
                    r#type: "text",
                    oninput: move |evt|
                    {
                        user_name.set(evt.data().value());
                    }
                }
            }
            div {
                Register {user_name}
                Login {user_name}
            }
        }
    }
}
