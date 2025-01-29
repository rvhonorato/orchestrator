#==========================================================
FROM rust:1.84 AS build
WORKDIR /opt
COPY . .
RUN cargo build --release
WORKDIR /data
ENTRYPOINT [ "/opt/target/release/orchestrator" ]
#==========================================================
# TODO: Maybe use `scratch` here to minimize the image size
