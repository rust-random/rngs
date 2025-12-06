# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

This is a new implementation by [@tertu-m](https://github.com/tertu-m) and [@dhardy](https://github.com/dhardy/). The prior implementation has been renamed to [sfc-prng](https://crates.io/crates/sfc-prng).

Compared to the prior implementation:

### Changed

- Value-stability is not preserved since constructors use a different number of mixing rounds

### Removed

- The `new` and `new_u64` constructors are not included
