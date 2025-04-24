FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN apt-get update -y && apt-get install -y musl-tools
RUN rustup target add aarch64-unknown-linux-musl
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
RUN cargo build --release --target=aarch64-unknown-linux-musl

FROM alpine:latest
WORKDIR /app

COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/sushi .
# COPY Makefile .
COPY justfile .

RUN apk update 
RUN apk add vim tmux net-tools netcat-openbsd bash just redis

CMD ["sh", "-c", "ifconfig eth0 | awk '/inet /{print $2}' && bash"]

