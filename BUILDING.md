# Building

## Add Target 

```text
rustup target add thumbv7em-none-eabihf
```

Install cargo-binutils

```text
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview
```

```text  
cargo build --release --example can --features="rt rtic usb-logging" --target thumbv7em-none-eabihf

cargo build --release --example uart --features="rt rtic usb-logging" --target thumbv7em-none-eabihf
```

```text  
rust-objcopy -O ihex target/thumbv7em-none-eabihf/release/examples/can build/can.hex

rust-objcopy -O ihex target/thumbv7em-none-eabihf/release/examples/uart build/uart.hex

```
