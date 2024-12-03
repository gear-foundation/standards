FROM rust:1.83 as builder

RUN rustup component add rust-src --toolchain 1.83.0-x86_64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN mkdir /artifacts

RUN ls -al /usr/src/myapp/target/wasm32-unknown-unknown/release/

COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.wasm /artifacts/
COPY --from=builder /usr/src/myapp/target/wasm32-unknown-unknown/release/*.idl /artifacts/

RUN ls -al /artifacts

ENTRYPOINT ["echo", "Artifacts published to /artifacts"]
