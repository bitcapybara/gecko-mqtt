#!/bin/bash

docker build -t registry.cn-beijing.aliyuncs.com/bitcapybara/gecko-mqtt:latest . -f standalone.Dockerfile

docker push registry.cn-beijing.aliyuncs.com/bitcapybara/gecko-mqtt:latest