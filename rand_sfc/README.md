# rand_sfc

[![Test Status](https://github.com/rust-random/rngs/actions/workflows/test.yml/badge.svg?event=push)](https://github.com/rust-random/rngs/actions)
[![Latest version](https://img.shields.io/crates/v/rand_sfc.svg)](https://crates.io/crates/rand_sfc)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://docs.rs/rand_sfc/badge.svg)](https://docs.rs/rand_sfc)

Rust implementation of the SFC random number generators, originally developed for the [PractRand](https://pracrand.sourceforge.net/) random number generator test suite.

This crate depends on [rand_core](https://crates.io/crates/rand_core) and is
part of the [Rand project](https://github.com/rust-random/rand).

Links:

-   [API documentation (docs.rs)](https://docs.rs/rand_sfc)

## Crate Features

`rand_sfc` is no_std compatible by default.

The `serde` feature includes implementations of `Serialize` and `Deserialize` for the included RNGs.

## License

`rand_sfc` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
