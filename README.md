![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/hackrfone.svg)](https://crates.io/crates/hackrfone)
[![docs.rs](https://docs.rs/hackrfone/badge.svg)](https://docs.rs/hackrfone/)
[![CI](https://github.com/newAM/hackrfone/workflows/CI/badge.svg)](https://github.com/newAM/hackrfone/actions)

# HackRF One

This is a rust API for the [HackRF One] software defined radio.

This is not a wrapper around `libhackrf`, this is a re-implementation of
`libhackrf` in rust, using the [rusb] `libusb` wrappers.

This is currently in an *experimental* state, and it is incomplete.
For full feature support use the official `libhackrf` C library.

[rusb]: https://github.com/a1ien/rusb
[HackRF One]: https://greatscottgadgets.com/hackrf/one/
