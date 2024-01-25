# https://windsoilder.github.io/writing_dockerfile_in_rust_project.html
FROM rust:1.75-buster as builder

WORKDIR /app

RUN apt install libssl-dev

# create a new empty project
RUN cargo init

COPY Cargo.toml Cargo.lock ./

# build dependencies, when my source code changes, this build can be cached, we don't need to compile dependency again.
RUN cargo build --release

# remove the dummy build.
RUN cargo clean --release -p healthcheck 

# copy source code
COPY src ./src
COPY .sqlx ./.sqlx
# build with x86_64-unknown-linux-musl to make it run with alpine.
RUN cargo build --release