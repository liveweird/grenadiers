# Pin manifest indexes so supported platforms resolve to reviewed image contents.
FROM node:24.21.0-alpine@sha256:ebfe2f90462722a7a4de65e91990e97fe0d401c70e0e762c5b53302f905ec1c1 AS web
WORKDIR /build/web
COPY web/package.json web/package-lock.json web/.npmrc ./
RUN npm ci
COPY web/ ./
COPY api/ /build/api/
COPY assets/art/arena-source.png assets/art/arena-source.json /build/assets/art/
RUN npm run build

FROM rust:1.99.1-bookworm@sha256:48b7f435b8fc6378ce43b44f15e3a47fd8919b2825f835c39bd4bd3da50af5b8 AS server
ARG TARGETPLATFORM
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core/ core/
COPY server/ server/
COPY api/ api/
# Force workspace rebuilds even when source mtimes predate restored cache entries.
# Keep these roots aligned with the production Cargo targets.
RUN --mount=type=cache,id=grenadiers-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=grenadiers-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=grenadiers-cargo-target-${TARGETPLATFORM},target=/build/target,sharing=locked \
    touch core/src/lib.rs server/src/lib.rs server/src/main.rs \
    && cargo build --release --locked -p grenadiers-server \
    && cp target/release/grenadiers-server /build/grenadiers-server

FROM debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587 AS runtime
LABEL org.opencontainers.image.title="Grenadiers" \
      org.opencontainers.image.source="https://github.com/liveweird/grenadiers" \
      org.opencontainers.image.documentation="https://github.com/liveweird/grenadiers#readme"
RUN apt-get update && apt-get upgrade -y --no-install-recommends \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=server /build/grenadiers-server /app/grenadiers-server
COPY --from=web /build/web/dist /app/web/dist
ENV SERVER_ADDR=0.0.0.0:8085 WEB_DIST_DIR=/app/web/dist RUST_LOG=grenadiers_server=info,tower_http=info
USER 65532:65532
EXPOSE 8085
ENTRYPOINT ["/app/grenadiers-server"]
