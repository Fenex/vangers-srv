FROM rust:alpine AS builder

WORKDIR /vangers-srv

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

COPY vangers-srv ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/vangers-srv/target \
    cargo build --release

RUN --mount=type=cache,target=/vangers-srv/target \
    mkdir -p /vangers-srv/artifacts \
    && cp /vangers-srv/target/release/vangers-srv /vangers-srv/artifacts/vangers-srv

FROM scratch

WORKDIR /vangers-srv

COPY --from=builder /vangers-srv/artifacts/vangers-srv /vangers-srv/vangers-srv

CMD ["/vangers-srv/vangers-srv"]
