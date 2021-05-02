# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added `impl std::error::Error for Error`.
- Added `Copy, Clone, PartialEq, Eq` traits for `Error`.

### Changed
- Changed the return type of `iq_to_cplx` from `Complex<f32>` to `Complex<i8>`.

## [0.1.0] - 2021-03-02
- Initial release

[Unreleased]: https://github.com/newAM/hackrfone/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/newAM/hackrfone/releases/tag/v0.1.0
