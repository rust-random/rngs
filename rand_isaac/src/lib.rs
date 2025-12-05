// Copyright 2018-2023 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The ISAAC and ISAAC-64 random number generators.
//!
//! To initialize a generator, use the [`SeedableRng`][rand_core::SeedableRng] trait.

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]
#![allow(
    clippy::too_many_arguments,
    clippy::many_single_char_names,
    clippy::identity_op
)]
#![cfg_attr(not(all(feature = "serde", test)), no_std)]

const RAND_SIZE_LOG2: usize = 8;
const RAND_SIZE: usize = 1 << RAND_SIZE_LOG2;

pub mod isaac;
pub mod isaac64;

#[cfg(feature = "serde")]
mod serde_impl;

pub use self::isaac::IsaacRng;
pub use self::isaac64::Isaac64Rng;
