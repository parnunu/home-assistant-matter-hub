FROM rust:1.88-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig dbus-dev avahi-dev clang llvm-dev
WORKDIR /src
COPY . .
RUN cargo build --release -p hamh-app

FROM ghcr.io/hassio-addons/base:18.2.1
RUN apk add --no-cache libstdc++ dbus avahi-libs
ENV HAMH_API_PORT=8482
ENV HAMH_STORAGE_LOCATION=/config/.hamh-storage
ENV RUST_LOG=info
VOLUME /config
COPY --from=builder /src/target/release/hamh-app /usr/local/bin/hamh-app
COPY docker/rework-addon-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh
LABEL io.hass.type="addon" io.hass.arch="armhf|aarch64|i386|amd64"
CMD ["/docker-entrypoint.sh"]
