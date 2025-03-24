FROM rust:1.85 as builder
WORKDIR /usr/src/rssensplaner
COPY . .
RUN cargo install --path .

FROM debian:stretch-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rssensplaner /usr/local/bin/rssensplaner
CMD ["rssensplaner"]
