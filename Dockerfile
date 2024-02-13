FROM rust:1.75.0-bullseye as builder

ARG DATABASE_URL
ARG KAFKA_URL
ARG PORT
ARG CARGO_FLAGS

WORKDIR /app 

COPY . . 

ENV TZ=Asia/Jakarta
ENV PORT=${PORT}
ENV KAFKA_URL=${KAFKA_URL}
ENV DATABASE_URL=${DATABASE_URL}

RUN apt update
RUN apt install -y libssl-dev pkg-config
RUN rustup component add rustfmt
RUN cargo build --release ${CARGO_FLAGS}
RUN cargo test --release ${CARGO_FLAGS}

FROM debian:bullseye-slim as runtime

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

