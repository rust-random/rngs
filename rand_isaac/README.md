# rand_isaac

[![Test Status](https://github.com/rust-random/rngs/actions/workflows/test.yml/badge.svg?event=push)](https://github.com/rust-random/rngs/actions)
[![Latest version](https://img.shields.io/crates/v/rand_isaac.svg)](https://crates.io/crates/rand_isaac)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://img.shields.io/badge/api-master-yellow.svg)](https://rust-random.github.io/rand/rand_isaac)
[![API](https://docs.rs/rand_isaac/badge.svg)](https://docs.rs/rand_isaac)

Implements the ISAAC and ISAAC-64 random number generators.

ISAAC stands for "Indirection, Shift, Accumulate, Add, and Count" which are
the principal bitwise operations employed. It is the most advanced of a
series of array based random number generator designed by Robert Jenkins
in 1996[^1][^2].

ISAAC is notably fast and produces excellent quality random numbers for
non-cryptographic applications.

Links:

-   [API documentation (master)](https://rust-random.github.io/rand/rand_isaac)
-   [API documentation (docs.rs)](https://docs.rs/rand_isaac)
-   [Changelog](https://github.com/rust-random/rngs/blob/master/rand_isaac/CHANGELOG.md)

[rand]: https://crates.io/crates/rand
[^1]: Bob Jenkins, [*ISAAC: A fast cryptographic random number generator*](http://burtleburtle.net/bob/rand/isaacafa.html)
[^2]: Bob Jenkins, [*ISAAC and RC4*](http://burtleburtle.net/bob/rand/isaac.html)


## Crate Features

`rand_isaac` is `no_std` compatible. It does not require any functionality
outside of the `core` lib, thus there are no features to configure.

The `serde` feature includes implementations of `Serialize` and `Deserialize`
for the included RNGs.


# License

`rand_isaac` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
