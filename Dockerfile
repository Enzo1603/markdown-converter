FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final Image
FROM debian:stable-slim

ENV DEBIAN_FRONTEND=noninteractive

# Install Pandoc
RUN apt-get update && \
    apt-get install -y pandoc && \
    apt-get autoremove && \
    apt-get autoclean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/markdown_converter /app/
EXPOSE 8000

CMD ["./markdown_converter"]
