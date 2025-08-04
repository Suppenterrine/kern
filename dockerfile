FROM rust:1.78 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin kern-server

FROM debian:buster-slim
COPY --from=builder /app/target/release/kern-server /usr/local/bin/kern-server
CMD ["kern-server"]
