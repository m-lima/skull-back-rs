# Build
FROM docker.io/rust:1.73.0-bookworm as rust
WORKDIR /src
COPY . .
RUN cargo build --release --bin server

# Pack
FROM docker.io/debian:stable-20231009-slim
WORKDIR /opt/skull
COPY --from=rust /src/target/release/server /opt/skull/skull
EXPOSE 80
ENV CLICOLOR_FORCE 1

ENTRYPOINT ["./skull"]
