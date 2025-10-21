// Copyright 2025 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate implements the sfc (Small Fast Counting) family of pseudorandom
//! number generators designed by Chris Doty-Humphrey, author of the [PractRand]
//! pseudorandom number generator testing suite. The generators have a small
//! state, good statistical quality, and high performance. Like most generators
//! intended for non-cryptographic use, they can be [predicted]. They also have
//! a variable period, but the counter present in the state provides a fixed
//! lower bound and [most states] are on the longest cycle.
//!
//! This crate provides:
//! - [`Sfc64`]: 64 bit output, seed space 192 bits, worst-case period 2^64,
//!   and expected period ~2^255.
//! - [`Sfc32`]: 32 bit output, seed space 96 bits, worst-case period 2^32,
//!   and expected period ~2^127.
//!
//! The implementations provided are derived from PractRand.
//!
//! [PractRand]: https://pracrand.sourceforge.net/
//! [predicted]: https://github.com/michaelni/randomtests/blob/main/sfc64-breach.c
//! [most states]: https://www.pcg-random.org/posts/random-invertible-mapping-statistics.html

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![no_std]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod sfc32;
mod sfc64;

pub use rand_core;
pub use sfc32::Sfc32;
pub use sfc64::Sfc64;