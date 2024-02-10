FROM rust:1.75.0-slim-bullseye as base

WORKDIR /app 

COPY . . 

ENV TZ=Asia/Jakarta

RUN apt update && apt install -y libssl-dev pkg-config curl ca-certificates
RUN rustup component add rustfmt
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y cargo-watch
RUN cargo build --release

CMD cargo watch -s "sh entrypoint.sh"
