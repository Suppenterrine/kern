# ===== Builder (Nightly, edition 2024) =====================================
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app

# 1) Alle Build-Tools & libs für openssl-sys (vendored)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev perl make build-essential ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 2) Copy source & Cargo cache (schlanke Multi-Stage-Caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "// dummy" > src/lib.rs \
    && cargo fetch

COPY . .

ARG BUILD_BIN=kern-server
RUN cargo build --release --bin ${BUILD_BIN}

# ===== Runtime (ultraschlank) ============================================
FROM debian:bullseye-slim AS runtime

# 4) Nur nötige Certs & Strip-Tool
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates binutils && \
    rm -rf /var/lib/apt/lists/*

ARG BUILD_BIN=kern-server
COPY --from=builder /app/target/release/${BUILD_BIN} /usr/local/bin/${BUILD_BIN}

# 5) Strip reduce Binary-Size
RUN strip /usr/local/bin/${BUILD_BIN}

ENTRYPOINT ["/usr/local/bin/kern-server"]
