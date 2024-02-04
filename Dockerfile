FROM rust

ARG CERT_PEM
ARG CERT_KEY

ENV TZ=Asia/Jakarta
ENV PORT=443
ENV DATABASE_URL=postgres://root:root@lighter-postgres:5432/auth
ENV TLS_CERT=/app/cert.pem
ENV TLS_KEY=/app/key.pem

WORKDIR /app 
COPY . . 
COPY $CERT_PEM /app/cert.pem
COPY $CERT_KEY /app/key.pem

RUN apt update && apt install -y libssl-dev pkg-config
RUN rustup component add rustfmt
# RUN cargo install cargo-watch
RUN cargo install sea-orm-cli
RUN cargo build --release
RUN cargo test --release

CMD sh entrypoint.sh
