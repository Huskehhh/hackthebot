FROM rust:slim-buster as builder
RUN apt-get update && apt-get install -y build-essential libssl-dev openssl pkg-config

WORKDIR /home/rust/
COPY . .

RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/hackthebot /usr/bin/hackthebot

USER 1000

CMD ["hackthebot"]
