# ===== Builder (Nightly, edition 2024) =====================================
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app
COPY . .

# Build nur gewünschten Binärnamen
ARG BUILD_BIN=kern-server
RUN cargo build --release --bin ${BUILD_BIN}

# ===== Runtime (schlank) ==========================================
FROM debian:buster-slim
ARG BUILD_BIN=kern-server
COPY --from=builder /app/target/release/${BUILD_BIN} /usr/local/bin/${BUILD_BIN}
ENTRYPOINT ["/usr/local/bin/kern-server"]
