# Building

```text  
cargo build --release --example can --features="rt rtic usb-logging" --target thumbv7em-none-eabihf
```

```text  
rust-objcopy -O ihex target/thumbv7em-none-eabihf/release/examples/can can.hex
```