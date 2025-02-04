#==========================================================
FROM rust:1.84 AS build
WORKDIR /opt
COPY . .
RUN cargo build --release
# ENTRYPOINT ["/opt/target/release/orchestrator"]
#==========================================================
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /opt/target/release/orchestrator /usr/local/bin/orchestrator
ENTRYPOINT ["/usr/local/bin/orchestrator"]
#==========================================================
