FROM rust:1.59 as planner

RUN set -eu; \
    apt-get update; \
    apt-get install -y \
      clang \
      lld \
      ;

RUN useradd -d /app -m app

USER app
WORKDIR /app
RUN cargo install cargo-chef

FROM planner as cacher

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM planner as builder
COPY --from=cacher /app/recipe.json ./
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

ARG SQLX_OFFLINE=true

RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim

RUN set -eu; \
    apt-get update -yq; \
    apt-get install -yq --no-install-recommends \
      ca-certificates \
      openssl; \
    apt-get autoremove -y; \
    apt-get clean -y; \
    rm -rf /var/lib/apt/lists/

RUN useradd -d /app app

USER app
WORKDIR /app

COPY --from=builder /app/target/release/zero2prod zero2prod
ENV APP_ENVIRONMENT=production
CMD ["./zero2prod"]
