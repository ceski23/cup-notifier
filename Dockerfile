FROM rust:1.89.0 AS planner

RUN cargo install cargo-chef
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM planner AS builder
ARG TARGET
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target ${TARGET} --recipe-path recipe.json
COPY . .
RUN cargo build --release --target ${TARGET}

FROM gcr.io/distroless/cc
ARG TARGET
COPY --from=builder /app/target/${TARGET}/release/cup_notifier /cup_notifier
ENTRYPOINT ["/cup_notifier"]