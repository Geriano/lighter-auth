FROM rust:1.75.0-slim-buster as builder

WORKDIR /app 

ARG CERT_PEM
ARG CERT_KEY

COPY . . 
COPY $CERT_PEM /app/cert.pem
COPY $CERT_KEY /app/key.pem

RUN apt update && apt install -y libssl-dev pkg-config
RUN rustup component add rustfmt
# RUN cargo install cargo-watch
# RUN cargo install sea-orm-cli
RUN cargo build --release
RUN cargo test --release

FROM rust:1.75.0-slim-buster as runtime

WORKDIR /app

ARG CERT_PEM
ARG CERT_KEY

ENV TZ=Asia/Jakarta
ENV PORT=443
ENV DATABASE_URL=postgres://root:root@lighter-postgres:5432/auth
ENV TLS_CERT=/app/cert.pem
ENV TLS_KEY=/app/key.pem

RUN apt update && apt install -y libssl-dev pkg-config
RUN cargo install sea-orm-cli

COPY --from=builder /app/target/release/lighter-auth /app/auth
COPY --from=builder /app/cert.pem /app/cert.pem
COPY --from=builder /app/key.pem /app/key.pem

CMD /app/auth
