FROM rust:1.63.0 AS build

ARG TARGET=x86_64-unknown-linux-musl

WORKDIR /gecko-mqtt
COPY . /gecko-mqtt/

RUN apt update && apt upgrade -y && \
    apt install -y protobuf-compiler libprotobuf-dev musl-dev && \ 
    rustup target add $TARGET && \
    rustup toolchain install stable-$TARGET && \
    cargo build --release --target $TARGET --bin standalone && \
    mv target/$TARGET/release/standalone /bin/gecko-mqtt-standalone

FROM scratch

EXPOSE 1883

WORKDIR /gecko-mqtt
COPY --from=build /bin/gecko-mqtt-standalone /gecko-mqtt
ADD examples/config /gecko-mqtt/

ENV CONFIG_FILE=standalone.toml

CMD ["./gecko-mqtt-standalone"]