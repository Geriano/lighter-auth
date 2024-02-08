FROM rust:latest as builder

WORKDIR /app 

COPY . . 

RUN apt update && apt install -y libssl-dev pkg-config curl ca-certificates
RUN rustup component add rustfmt
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y cargo-watch
RUN cargo binstall -y sea-orm-cli
RUN cargo build --release
RUN cargo test --release

ENV TZ=Asia/Jakarta
ENV PORT=5678
ENV DATABASE_URL=postgres://root:root@lighter-postgres:5432/auth
ENV TLS_CERT=/app/cert.pem
ENV TLS_KEY=/app/key.pem

CMD cargo watch -x "run --release"
