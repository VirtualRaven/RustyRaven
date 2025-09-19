FROM rust:1-alpine3.22 AS builder

RUN apk add musl-dev openssl openssl-libs-static openssl-dev  alpine-sdk pkgconf
RUN  cargo install dioxus-cli@0.6.3 --locked
RUN mkdir builddir outputdir 
COPY Cargo.lock Cargo.toml builddir/
COPY crates/ builddir/crates/
COPY .cargo/ builddir/.cargo/
COPY .sqlx/ builddir/.sqlx/
RUN ls builddir && cd builddir/crates/web && \ 
    dx bundle  --fullstack --out-dir ../../../outputdir --release


FROM alpine:3.22 
COPY --from=builder outputdir/ application/
CMD [ "application/server"]