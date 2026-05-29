FROM rust:slim-trixie AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    openssl \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release --bin loreserver && \
    cp /build/target/release/loreserver /build/loreserver-bin

# Generate dev certs (pipe passphrase to the interactive prompt)
RUN echo "docker-build" | bash scripts/server/make-certs.sh /build/certs

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/loreserver-bin /usr/local/bin/loreserver
COPY --from=builder /build/lore-server/config/default.toml /etc/lore/config/default.toml

COPY --from=builder /build/certs/ca.crt /etc/lore/certs/ca.crt
COPY --from=builder /build/certs/server.crt /etc/lore/certs/server.crt
COPY --from=builder /build/certs/server.key /etc/lore/certs/server.key

# Docker environment config: local stores at /data, certs for QUIC and gRPC
RUN cat <<'EOF' > /etc/lore/config/docker.toml
[immutable_store.local]
path = "/data"

[mutable_store.local]
path = "/data"

[server.quic.certificate]
cert_file = "/etc/lore/certs/server.crt"
pkey_file = "/etc/lore/certs/server.key"
EOF

ENV LORE_CONFIG_PATH=/etc/lore/config
ENV LORE_ENV=docker
ENV RUST_LOG=info

RUN mkdir -p /data
VOLUME /data

# QUIC + gRPC: 41337, HTTP: 41339
EXPOSE 41337 41339

ENTRYPOINT ["loreserver"]
