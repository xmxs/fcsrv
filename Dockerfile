FROM rust:1.75 as builder

WORKDIR /src

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src
RUN cargo build --release

FROM ubuntu:noble-20231214

LABEL org.opencontainers.image.source = "https://github.com/gngpp/fcsrv"
COPY --from=builder /src/target/release/fcsrv /bin/fcsrv
ENTRYPOINT [ "/bin/fcsrv"]