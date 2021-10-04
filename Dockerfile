# Build
FROM rust:1.55.0-buster as rust
WORKDIR /src
COPY . .
RUN cargo build --release

# Pack
FROM debian:stable-slim
WORKDIR /opt/skull
COPY --from=rust /src/target/release/skull /opt/skull/skull
EXPOSE 80
ENV CLICOLOR_FORCE 1

ENTRYPOINT ["./skull"]
CMD [ "-t", "1", "-p", "80", "-s", "/data" ]
