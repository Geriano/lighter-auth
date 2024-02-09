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
RUN cargo build --release
RUN cargo test --release

FROM base as runtime

ARG DATABASE_URL
ARG KAFKA_URL
ARG PORT

WORKDIR /app 

COPY . . 

ENV TZ=Asia/Jakarta
ENV PORT=${PORT}
ENV KAFKA_URL=${KAFKA_URL}
ENV DATABASE_URL=${DATABASE_URL}

COPY --from=builder /app/target/release/lighter-auth /app/lighter-auth

CMD /app/lighter-auth

