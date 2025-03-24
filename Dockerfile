FROM rust:1.85 AS builder
WORKDIR /usr/src/rssensplaner
COPY . .
RUN cargo install --path .

FROM debian:latest
RUN apt-get update && apt-get install -y openssl libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rssensplaner /usr/local/bin/rssensplaner
CMD ["rssensplaner"]
