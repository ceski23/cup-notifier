FROM clux/muslrust:1.89.0-nightly-2025-05-16 AS planner
RUN cargo install cargo-chef

WORKDIR /app
COPY . .
# Prepare a build plan ("recipe")
RUN cargo chef prepare --recipe-path recipe.json

FROM planner AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Build the whole project
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cup_notifier /cup_notifier
ENTRYPOINT ["/cup_notifier"]