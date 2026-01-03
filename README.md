![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/hackrfone.svg)](https://crates.io/crates/hackrfone)
[![docs.rs](https://docs.rs/hackrfone/badge.svg)](https://docs.rs/hackrfone/)
[![CI](https://github.com/newAM/hackrfone/workflows/CI/badge.svg)](https://github.com/newAM/hackrfone/actions)

# HackRF One

This is a rust API for the [HackRF One] software defined radio.

This is not a wrapper around `libhackrf`, this is a re-implementation of
`libhackrf` in rust, using the [nusb] user-space rust library.


This is currently in an **experimental** state, and it is incomplete.
For full feature support use the official `libhackrf` C library.

This is tested only on Linux, but it will likely work on other platforms where
`libhackrf` works.

## USB user-space access
Adapted from [nusb-linux-setup].
This is only needed if you want to run as non-root user, although the initial setup does require sudo.
1. ```shell 
   # Create a `udev` rule
   echo 'SUBSYSTEMS=="usb", ATTRS{idVendor}=="1d50", ATTRS{idProduct}=="6089", MODE="0660", GROUP="hackrfone"' | sudo tee /etc/udev/rules.d/99-hackrfone-usb.rules
   ```
2. ```shell
   # Add current user to group
   sudo groupadd hackrfone
   sudo usermod -aG hackrfone $USER
   ```
3. Best to restart the system now so new group shows up and device is mapped correctly

[nusb]: https://github.com/kevinmehall/nusb
[nusb-linux-setup]: https://docs.rs/nusb/latest/nusb/index.html#linux
[HackRF One]: https://greatscottgadgets.com/hackrf/one/
