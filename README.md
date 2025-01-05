# rpi-plant-moisture

[![Rust](https://github.com/cmackenzie1/rpi-plant-moisture/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/cmackenzie1/rpi-plant-moisture/actions/workflows/rust.yml)

A simple rust program to read the moisture level of a plant using a capacitive moisture sensor and a Raspberry Pi via I2C.

If you haven't already, you'll need to enable I2C on your Raspberry Pi. You can do this by running `sudo raspi-config` and enabling I2C in the interfaces section. You'll also need to install the `i2c-tools` package by running `sudo apt install i2c-tools` to use the `i2cdetect` command.

Most of the code is based on the CircuitPython Seesaw library for the STEMMA Soil Sensor. You can find the original code [here](https://github.com/adafruit/Adafruit_CircuitPython_seesaw).

![rpi with sensor](static/IMG_3127.jpeg)

## Prerequisites

### Hardware

- [Raspberry Pi Zero W 2](https://www.adafruit.com/product/5291)
- [Adafruit STEMMA Soil Sensor - I2C Capacitive Moisture Sensor](https://www.adafruit.com/product/4026)
- [JST PH 2mm Female Socket](https://www.adafruit.com/product/3950)

### Software

- [Rust](https://www.rust-lang.org/learn/get-started)
- [Cargo Cross](https://github.com/cross-rs/cross) (for cross compiling, requires Docker)
- (Optional): Grafana

## Usage

If you are developing on the Pi itself, you can run the program using cargo

```bash
# Run the program using cargo (assuming you are on the Raspberry Pi)
cargo run -- -h
Usage: rpi-plant-moisture [OPTIONS]

Options:
  -d, --device <DEVICE>                      [default: /dev/i2c-1]
  -m, --metrics-addr <METRICS_ADDR>          [default: 0.0.0.0:3000]
  -q, --quiet
  -i, --interval-seconds <INTERVAL_SECONDS>  [default: 60]
  -h, --help                                 Print help
  -V, --version                              Print version
```

Otherwise, if you are cross-compiling from another machine, you'll need to install the build target or use [cargo-cross](https://github.com/cross-rs/cross)

```bash
# Using Rust toolchain
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# Using cargo-cross
cross build --release --target aarch64-unknown-linux-gnu
```

After building the binary, `scp` it to your Pi and run it.

```bash
# copy the binary to the Raspberry Pi
scp target/aarch64-unknown-linux-gnu/release/rpi-plant-moisture <username>@<ip>:~/rpi-plant-moisture
# run the binary on the Raspberry Pi via SSH
./rpi-plant-moisture
```

If everything is working, the output should look something like this:

```text
Starting soil sensor readings...
Prometheus metrics are available at http://0.0.0.0:3000/metrics
Temperature: 20.73°C
Moisture: 1008 (200 - 2000)
---
```

Happy Planting!

## Final product

![Calamansi tree](static/IMG_3134.jpeg)
