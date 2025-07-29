#===============================================================================
FROM rust:1.84 AS build

WORKDIR /opt

COPY . .

RUN cargo build --release

#===============================================================================
FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  openssl \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=build /opt/target/release/orchestrator /usr/local/bin/orchestrator

ENTRYPOINT ["/usr/local/bin/orchestrator"]
#===============================================================================
