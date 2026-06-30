FROM rust:1.83-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY migrations ./migrations
COPY src ./src
COPY templates ./templates
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/recipe_creator /usr/local/bin/recipe_creator
COPY migrations ./migrations
COPY templates ./templates
COPY static ./static
COPY .env.example ./.env.example

RUN mkdir -p /app/data

ENV PORT=8090
ENV DATABASE_URL=sqlite:/app/data/recipe_creator.db?mode=rwc

EXPOSE 8090
CMD ["recipe_creator"]
