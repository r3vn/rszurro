FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
ADD src src

RUN apt update && apt install -y libclang-dev \
    libsensors-dev \
    libssl-dev

RUN cargo build --release
RUN strip target/release/rszurro

FROM debian:latest
WORKDIR /app

RUN apt update && apt install -y libsensors5 \
    libssl3 \
    libclang1

RUN apt clean
RUN mkdir /data

# Create an user for the application
RUN useradd -ms /bin/bash rszurro
RUN chown -R rszurro:rszurro /app

COPY --from=builder /app/target/release/rszurro .

USER rszurro
ENTRYPOINT ["./rszurro"]
