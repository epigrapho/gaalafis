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
RUN cargo build --release --bin sbs_custom_signing
RUN cargo build --release --bin sbs_custom_signing_locks

FROM ubuntu:22.04 AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y openssl
EXPOSE 3000

FROM runtime AS runtime-proxy-sbs
COPY --from=builder /app/target/release/sbs_custom_signing /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-proxy-sbs-locks-pg
COPY --from=builder /app/target/release/sbs_custom_signing_locks /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-signer-sbs-locks-pg
COPY --from=builder /app/target/release/lfs-info-server /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

