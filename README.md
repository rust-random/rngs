# Rand

[![Build Status](https://travis-ci.org/rust-random/rngs.svg?branch=master)](https://travis-ci.org/rust-random/rngs)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/rust-random/rngs?svg=true)](https://ci.appveyor.com/project/rust-random/rngs)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://img.shields.io/badge/api-master-yellow.svg)](https://rust-random.github.io/rand)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.32+-lightgray.svg)](https://github.com/rust-random/rngs#rust-version-requirements)

Extra random number generators provided by the Rust Random project.
The main repository, [rust-random/rand](https://github.com/rust-random/rand),
includes all generators which are a direct dependency of the `rand` crate.
This repository houses extra generators maintained by the project.

Documentation:
-   [Our generators](https://rust-random.github.io/book/guide-rngs.html)
-   [The Rust Rand Book](https://rust-random.github.io/book)
-   [API reference (master)](https://rust-random.github.io/rand)
-   [API reference (docs.rs)](https://docs.rs/rand)


### Rust version requirements

Rand requires **Rustc version 1.32 or greater**.

Travis CI always has a build with a pinned version of Rustc matching the oldest
supported Rust release. The current policy is that this can be updated in any
Rand release if required, but the change must be noted in the changelog.

# License

Rand is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
