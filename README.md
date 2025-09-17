### E-ecommerce store in Rust ###

This repository contains the source code for [sjfconcept.se](https://www.sjfconcept.se) an E-ecommerce 
written in RUST. It is a fast single page site application based on the Dioxus web framework.

The code is split into six crates
* Web - The main crate which contains the Dioxus UI components. This crate has two rust features,
 when compiled into native code with the feature "server" it becomes an Axum based HTTP server hosting
the Dioxus powered website. With feature "Web" and compiled into WASM it becomes the client side UI/Application.
* Image - Implements image resizing, Image storage/retrival in S3 object storage and an S3 Cache.
* DB - Uses the Rust SQLX lib to interface with the website's PostgreSQL database
* Auth - Provides Passkey (Webauthn) authentication for the websites administrative pages
* Payment - Realizes the checkout flow by interfacing with the payment provider Stripe
* API - Common data types shared among the crates




---
![SJF-concept](https://github.com/user-attachments/assets/7d0243df-07de-4f56-8d47-c7c9e317080d)
