FROM rust:1.63.0 AS build

WORKDIR /gecko-mqtt
COPY . /gecko-mqtt/

RUN apt update && apt upgrade -y && \
    apt install -y protobuf-compiler libprotobuf-dev musl-dev && \ 
    rustup target add x86_64-unknown-linux-musl && \
    rustup toolchain install stable-x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl --bin standalone && \
    mv target/x86_64-unknown-linux-musl/release/standalone /bin/gecko-mqtt-standalone

FROM scratch

EXPOSE 1883

WORKDIR /gecko-mqtt
COPY --from=build /bin/gecko-mqtt-standalone /gecko-mqtt
ADD examples/config /gecko-mqtt/

ENV CONFIG_FILE=standalone.toml

CMD ["./gecko-mqtt-standalone"]