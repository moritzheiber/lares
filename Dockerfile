FROM rust:alpine as builder

WORKDIR /usr/src/binary
COPY . .
RUN apk add --no-cache openssl-dev musl-dev && \
    cargo build --release --locked

FROM alpine:latest

ENV LARES_HOST="0.0.0.0" \
    LARES_PORT="4000"

COPY --from=builder /usr/src/binary/target/release/lares /usr/bin/lares

EXPOSE "${LARES_PORT}/tcp"
CMD ["/usr/bin/lares","server"]
