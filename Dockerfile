FROM rust:1.84-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --package changelogger-cli && \
    cp target/release/changelogger /usr/local/bin/changelogger

FROM gcr.io/distroless/cc-debian12:latest
COPY --from=builder /usr/local/bin/changelogger /usr/local/bin/changelogger
COPY --from=builder /app/templates /templates
ENTRYPOINT ["changelogger"]