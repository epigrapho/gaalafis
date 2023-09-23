FROM rust:1.72 as chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin lfs-info-server

FROM ubuntu:22.04 AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y openssl
COPY --from=builder /app/target/release/lfs-info-server /usr/local/bin
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/lfs-info-server"]
