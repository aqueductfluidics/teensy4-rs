FROM ubuntu:20.04 as base

RUN apt-get update -y --fix-missing
RUN apt-get install -y make gcc-arm-none-eabi binutils-arm-none-eabi libnewlib-arm-none-eabi

FROM base AS build.1

WORKDIR /

RUN mkdir -p /home/make

COPY /bin /home/make/bin

VOLUME /home/make

WORKDIR /home/make

RUN make -C bin libt4usb.a
RUN make -C bin libt4eeprom.a

RUN /bin/bash
