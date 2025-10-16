# Stage 1: dev build
FROM rust:1.90-bookworm AS dev

WORKDIR /app

RUN cargo install cargo-watch

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build
RUN rm -rf src

COPY . .

# Stage 2: Runtime (minimal production)
FROM rust:1.90-slim-bookworm AS runtime

WORKDIR /app

COPY --from=dev /app/target/release/match_core /usr/local/bin/match-core

EXPOSE 3000

CMD ["match-core"]

# Stage 3: Dev hot-reload
FROM dev AS dev-runtime

WORKDIR /app

VOLUME ["/app/src"]

EXPOSE 3000

CMD ["cargo", "watch", "-x", "run"]