# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changes
- Use Edition 2024 and MSRV 1.85 ([#73])
- Update to `rand_core` v0.10 ([#82])

[#73]: https://github.com/rust-random/rngs/pull/73
[#82]: https://github.com/rust-random/rngs/pull/82

## [0.5.0] - 2025-01-27
- Bump the MSRV to 1.63 ([#58])
- Update to `rand_core` v0.9.0 ([#58])

[#58]: https://github.com/rust-random/rngs/pull/58

## [0.4.0] - 2023-12-08
### Changed
- Update to `rand_core` v0.6
- Bump MSRV to 1.51
- Use return-position impl trait to fix `JitterRng::new` ([#16])

[#16]: https://github.com/rust-random/rngs/pull/16

## [0.3.0] - 2020-09-07
### Changed
- `JitterRng::new_with_timer` accepts closures (this breaks `JitterRng::new`)

## [0.2.1] - 2019-08-16
### Changed
- `TimerError` changed to `repr(u32)` ([rand#864])
- `TimerError` enum values all increased by `1<<30` to match new `rand_core::Error` range ([rand#864])

[rand#864]: https://github.com/rust-random/rand/pull/864

## [0.2.0] - 2019-06-06
- Bump `rand_core` version
- Support new `Error` type in `rand_core` 0.5
- Remove CryptoRng trait bound ([rand#699], [rand#814])
- Enable doc-testing of README

[rand#699]: https://github.com/rust-random/rand/issues/699
[rand#814]: https://github.com/rust-random/rand/pull/814

## [0.1.4] - 2019-05-02
- Change error conversion code to partially fix [rand#738]

[rand#738]: https://github.com/rust-random/rand/issues/738

## [0.1.3] - 2019-02-05
- Use libc in `no_std` mode to fix [rand#723]

[rand#723]: https://github.com/rust-random/rand/issues/723

## [0.1.2] - 2019-01-31
- Fix for older rustc compilers on Windows ([rand#722])

[rand#722]: https://github.com/rust-random/rand/pull/722

## [0.1.1] - 2019-01-29
- Fix for older rustc compilers on Mac OSX / iOS ([rand#720])
- Misc. doc fixes

[rand#720]: https://github.com/rust-random/rand/pull/720

## [0.1.0] - 2019-01-24
Initial release.
