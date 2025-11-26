# ===== Builder (Nightly, edition 2024) =====================================
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app

# 1) Build-Tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config libssl-dev perl make build-essential ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 2) Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY xtask/Cargo.toml ./xtask/
RUN mkdir src && echo "// dummy" > src/lib.rs \
    && mkdir -p xtask/src && echo "fn main() {}" > xtask/src/main.rs \
    && cargo fetch

COPY . .

ARG BUILD_BIN=kern-server
RUN cargo build --release --bin ${BUILD_BIN}

# ===== Runtime (glibc neu genug) ==========================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates binutils && \
    rm -rf /var/lib/apt/lists/*

ARG BUILD_BIN=kern-server
COPY --from=builder /app/target/release/${BUILD_BIN} /usr/local/bin/${BUILD_BIN}

RUN strip /usr/local/bin/${BUILD_BIN}

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/kern-server"]
