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

RUN cargo build --release --bin server-proxy-fs
RUN cargo build --release --bin server-proxy-fs-locks-pg
RUN cargo build --release --bin server-proxy-sbs
RUN cargo build --release --bin server-proxy-sbs-locks-pg
RUN cargo build --release --bin server-signer-sbs
RUN cargo build --release --bin server-signer-sbs-locks-pg

FROM ubuntu:22.04 AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y openssl
EXPOSE 3000

FROM runtime AS runtime-proxy-fs
COPY --from=builder /app/target/release/server-proxy-fs /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-proxy-fs-locks-pg
COPY --from=builder /app/target/release/server-proxy-fs-locks-pg /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-proxy-sbs
COPY --from=builder /app/target/release/server-proxy-sbs /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-proxy-sbs-locks-pg
COPY --from=builder /app/target/release/server-proxy-sbs-locks-pg /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-signer-sbs
COPY --from=builder /app/target/release/server-signer-sbs /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM runtime AS runtime-signer-sbs-locks-pg
COPY --from=builder /app/target/release/server-signer-sbs-locks-pg /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

