# syntax=docker/dockerfile:1.4

# ===========================
# Stage 1: Dev build
# ===========================
FROM rust:1.90-bookworm AS dev

WORKDIR /app

RUN cargo install cargo-watch \
    && apt-get update \
    && apt-get install -y protobuf-compiler pkg-config libssl-dev build-essential curl git


COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN --mount=type=ssh cargo fetch

COPY . .

RUN cargo build --release

RUN rm -rf src

# ===========================
# Stage 2: Runtime (production)
# ===========================
FROM rust:1.90-slim-bookworm AS runtime

WORKDIR /app

COPY --from=dev /app/target/release/match_core /usr/local/bin/match_core

EXPOSE 50051

CMD ["match_core"]

# ===========================
# Stage 3: Dev hot-reload
# ===========================
FROM dev AS dev-runtime

WORKDIR /app

VOLUME ["/app/src"]

EXPOSE 50051

CMD ["cargo", "watch", "-x", "run"]
