# rand_mwcx

[![Test Status](https://github.com/rust-random/rngs/actions/workflows/test.yml/badge.svg?event=push)](https://github.com/rust-random/rngs/actions)
[![Latest version](https://img.shields.io/crates/v/rand_mwcx.svg)](https://crates.io/crates/rand_mwcx)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://docs.rs/rand_mwcx/badge.svg)](https://docs.rs/rand_mwcx)

Rust implementation of the [scrambled multiply-with-carry](https://prng.di.unimi.it/#id-64-bit)
random number generators MWC128X, MWC192X, MWC64X/32, and MWC128X/32,
described in “[Scrambled Multiply-with-Carry Pseudorandom Number
Generators](https://arxiv.org/abs/2610.27313)”.

This crate depends on [rand_core](https://crates.io/crates/rand_core) and is
part of the [Rand project](https://github.com/rust-random/rand).

Links:

-   [API documentation (docs.rs)](https://docs.rs/rand_mwcx)
-   [Changelog](https://github.com/rust-random/rngs/blob/master/rand_mwcx/CHANGELOG.md)

## Crate Features

`rand_mwcx` is no_std compatible by default.

The `serde` feature includes implementations of `Serialize` and `Deserialize` for the included RNGs.

## License

`rand_mwcx` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
