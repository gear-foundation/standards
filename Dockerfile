FROM rust:1.83 as builder

RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release --target wasm32-unknown-unknown

FROM debian:bullseye-slim

RUN mkdir /artifacts

COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.wasm /artifacts/
COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.idl /artifacts/

ENTRYPOINT ["echo", "Artifacts published to /artifacts"]
