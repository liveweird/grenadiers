# Pin manifest indexes so supported platforms resolve to reviewed image contents.
FROM node:24.21.0-alpine@sha256:ebfe2f90462722a7a4de65e91990e97fe0d401c70e0e762c5b53302f905ec1c1 AS web
WORKDIR /build/web
COPY web/package.json web/package-lock.json web/.npmrc ./
RUN npm ci
COPY web/ ./
COPY api/ /build/api/
COPY assets/art/arena-source.png assets/art/arena-source.json /build/assets/art/
RUN npm run build

FROM rust:1.99.1-bookworm@sha256:e12566d5abc17a0da741628782cdb504b68b8961f5bfcf677936bf3e8b32de29 AS server
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core/ core/
COPY server/ server/
COPY api/ api/
RUN cargo build --release --locked -p grenadiers-server

FROM debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587 AS runtime
RUN apt-get update && apt-get upgrade -y --no-install-recommends \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=server /build/target/release/grenadiers-server /app/grenadiers-server
COPY --from=web /build/web/dist /app/web/dist
ENV SERVER_ADDR=0.0.0.0:8085 WEB_DIST_DIR=/app/web/dist RUST_LOG=grenadiers_server=info
USER 65532:65532
EXPOSE 8085
ENTRYPOINT ["/app/grenadiers-server"]
