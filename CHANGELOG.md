# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added tested platforms to the README.

### Fixed
- Fixed a panic condition in the `rx` example that occurred when the sample
  thread disconnected before the main thread requested a disconnection.

## [0.2.0] - 2021-05-09
### Added
- Added `impl std::error::Error for Error`.
- Added `Copy, Clone, PartialEq, Eq` traits for `Error`.
- Added `iq_to_cplx_i8`.
- Added a threaded RX example.

### Changed
- Changed the name of `iq_to_cplx` to `iq_to_cplx_f32`.

## [0.1.0] - 2021-05-02
- Initial release

[Unreleased]: https://github.com/newAM/hackrfone/compare/v0.1.0...HEAD
[0.2.0]: https://github.com/newAM/hackrfone/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/newAM/hackrfone/releases/tag/v0.1.0
