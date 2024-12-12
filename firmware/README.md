# Building the firmware
This is a small document for setting up a development environment to build the firmware. You'll need a couple things to get started.

1. (Linux only) Install the [RPi Pico udev rules](https://github.com/raspberrypi/picotool/blob/master/udev/99-picotool.rules). Add the following rules:
```
SUBSYSTEM=="usb", \
    ATTRS{idVendor}=="2e8a", \
    ATTRS{idProduct}=="000c", \
    TAG+="uaccess" \
    MODE="660", \  
    GROUP="plugdev"
```

2. Install the appropriate target toolchain: `rustup target add thumbv6m-none-eabi`.
3. Install cmake for tool compilation.
4. (Linux only) Install libudev-dev.
5. Install tools: `cargo install flip-link`.
5. Install tools: `cargo install --locked probe-rs-tools`. This is for installing firmware over Pico probe.
6. Run `cargo run` to install.
