#===============================================================================
FROM rust:alpine AS build

RUN apk add --no-cache \
  musl-dev \
  build-base \
  curl \
  ca-certificates \
  pkgconfig

WORKDIR /opt
COPY . .

RUN cargo build --release
#===============================================================================
FROM scratch AS prod

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /opt/target/release/job-orchestrator /usr/bin/job-orchestrator

#===============================================================================
FROM ghcr.io/haddocking/prodigy:v2.4.0 AS example

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /opt/target/release/job-orchestrator /usr/bin/job-orchestrator

ENTRYPOINT [ "" ]

#===============================================================================
FROM prod AS default

