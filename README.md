# Middlesp

This is the script for our ESP32 to allow the calculator to control and interact with the WiFi module over serial.

## Setup

See [esp-rs docs](https://docs.esp-rs.org/book/installation/index.html). Summary:

```sh
cargo install espup
espup install
sudo pacman -S --needed gcc git make flex bison gperf python cmake ninja ccache dfu-util libusb python-pip
cargo install espflash # For the cargo run
```

## Running

Literally plug the ESP32 into a usb port and run:

```sh
cargo run --release
```
