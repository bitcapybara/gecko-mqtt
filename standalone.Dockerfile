FROM rust:1.63.0 AS build

WORKDIR /gecko-mqtt
COPY . /gecko-mqtt/

RUN apt update && apt upgrade -y && \
    apt install -y protobuf-compiler libprotobuf-dev && \ 
    cargo build --release --bin standalone && \
    mv target/release/standalone /bin/gecko-mqtt-standalone

FROM scratch

EXPOSE 1883

WORKDIR /gecko-mqtt
COPY --from=build /bin/gecko-mqtt-standalone /gecko-mqtt
ADD examples/config /gecko-mqtt/

ENV CONFIG_FILE=standalone.toml

RUN ls
# COPY --from=build /usr/share/zoneinfo/Asia/Shanghai /usr/share/zoneinfo/Asia/Shanghai

# RUN rm -f /etc/localtime && \
#     ln -sv /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
# ENV TZ="Asia/Shanghai"

CMD ["./gecko-mqtt-standalone"]