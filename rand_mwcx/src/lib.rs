// Copyright 2026 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate implements the scrambled multiply-with-carry pseudorandom number
//! generators described in “[Scrambled Multiply-with-Carry Pseudorandom Number
//! Generators][paper]” by Sebastiano Vigna. Multiply-with-carry (MWC)
//! generators, introduced by George Marsaglia, simulate multiplicative
//! congruential generators with a large prime modulus using word-sized
//! multiplications: they are very fast, have a provably long period, and can
//! jump ahead by any number of steps. Their output is scrambled by X, which
//! xors the most recent digit with the carry, as in David Thomas's MWC64X
//! generator. However, they are not cryptographically secure and their output
//! can be predicted by observing a few samples.
//!
//! The following generators are implemented:
//!
//! # 64-bit generators
//! - [`Mwc192X`]: Recommended for all purposes. Excellent speed and a period
//!   of approximately 2¹⁹¹.
//! - [`Mwc128X`]: An alternative to `Mwc192X` with a smaller state (128 bits)
//!   and a period of approximately 2¹²⁷. It is slightly slower, as each
//!   multiplication needs the result of the previous step.
//!
//! # 32-bit generators
//! - [`Mwc128X32`]: Recommended for all purposes. Excellent speed and a period
//!   of approximately 2¹²⁷.
//! - [`Mwc64X32`]: An alternative to `Mwc128X32` with a smaller state (64 bits)
//!   and a period of approximately 2⁶³, meant for environments in which
//!   registers are scarce, such as GPUs.
//!
//! The 64-bit generators need a fast 64 × 64 → 128-bit multiplication, and
//! the 32-bit generators a fast 32 × 32 → 64-bit multiplication.
//!
//! All generators provide `jump()` and `long_jump()` functions, which jump
//! ahead by a fixed large number of steps, `jump_ce(c, e)`, which jumps ahead
//! by *c* · 2*ᵉ* steps, and `jump_n()`, which jumps ahead by an arbitrary number
//! of steps.
//!
//! # Example
//!
//! To initialize a generator, use the [`SeedableRng`][rand_core::SeedableRng] trait:
//!
//! ```
//! use rand_core::{SeedableRng, Rng};
//! use rand_mwcx::Mwc192X;
//!
//! let mut rng = Mwc192X::seed_from_u64(0);
//! let x = rng.next_u64();
//! ```
//!
//! [paper]: https://arxiv.org/abs/2610.27313

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico"
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![allow(clippy::unreadable_literal)]
#![no_std]

mod common;
mod mp;
mod mwc128x;
mod mwc128x32;
mod mwc192x;
mod mwc64x32;

pub use mwc64x32::Mwc64X32;
pub use mwc128x::Mwc128X;
pub use mwc128x32::Mwc128X32;
pub use mwc192x::Mwc192X;
pub use rand_core;
