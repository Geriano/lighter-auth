FROM rust:1.75.0-slim-bullseye as base

FROM base as builder

ARG DATABASE_URL
ARG KAFKA_URL
ARG PORT

WORKDIR /app 

COPY . . 

ENV TZ=Asia/Jakarta
ENV PORT=${PORT}
ENV KAFKA_URL=${KAFKA_URL}
ENV DATABASE_URL=${DATABASE_URL}

RUN apt update && apt install -y libssl-dev pkg-config curl ca-certificates
RUN rustup component add rustfmt
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y cargo-watch
RUN cargo binstall -y sea-orm-cli
RUN sea migrate up --database-url $DATABASE_URL
RUN cargo build --release

CMD cargo watch -s "sh entrypoint.sh"
