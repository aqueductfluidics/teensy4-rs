FROM ubuntu:20.04 as base

RUN apt-get update -y --fix-missing
RUN apt-get install -y make gcc-arm-none-eabi binutils-arm-none-eabi libnewlib-arm-none-eabi

FROM base AS build.1

WORKDIR /

RUN mkdir -p /home/make

COPY /bin /home/make/bin

WORKDIR /home/make

RUN make -C bin libt4usb.a
RUN make -C bin libt40eeprom.a CPPFLAGS="-DTEENSY40" 
RUN make -C bin libt41eeprom.a CPPFLAGS="-DTEENSY41" 

RUN /bin/bash
