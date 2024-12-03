FROM rust:1.83 as builder

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release --target wasm32-unknown-unknown

FROM debian:bullseye-slim

RUN mkdir /artifacts

COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.wasm /artifacts/
COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.idl /artifacts/

ENTRYPOINT ["echo", "Artifacts published to /artifacts"]
