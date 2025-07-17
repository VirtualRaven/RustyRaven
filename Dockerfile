FROM rust:1-alpine3.22 AS builder

RUN apk add musl-dev openssl openssl-libs-static openssl-dev  alpine-sdk pkgconf
RUN  cargo install dioxus-cli@0.6.3
RUN mkdir builddir outputdir 
COPY Cargo.lock Cargo.toml builddir/
COPY api/ builddir/api/
COPY db/ builddir/db/
COPY image/ builddir/image/
COPY .cargo/ builddir/.cargo/
COPY .sqlx/ builddir/.sqlx/
COPY web/ builddir/web/
RUN ls builddir && cd builddir/web && \ 
    dx bundle --release --fullstack --out-dir ../../outputdir
