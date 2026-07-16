// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::convert::Infallible;
use rand_core::{Rng, SeedableRng, TryRng, utils};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Seed512;

/// A xoshiro512+ random number generator.
///
/// The xoshiro512+ algorithm is not suitable for cryptographic purposes, but
/// is very fast and has good statistical properties, besides a low linear
/// complexity in the lowest bits.
///
/// The algorithm used here is translated from [the `xoshiro512plus.c`
/// reference source code](http://xoshiro.di.unimi.it/xoshiro512plus.c) by
/// David Blackman and Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Xoshiro512Plus {
    s: [u64; 8],
}

impl Xoshiro512Plus {
    /// Jump forward, equivalently to 2^256 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^256 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoshiro512Plus;
    ///
    /// let rng1 = Xoshiro512Plus::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(
            u64,
            self,
            [
                0x33ed89b6e7a353f9,
                0x760083d7955323be,
                0x2837f2fbb5f22fae,
                0x4b8c5674d309511c,
                0xb11ac47a7ba28c25,
                0xf1be7667092bcc1c,
                0x53851efdb6df0aaf,
                0x1ebbc8b23eaf25db
            ]
        );
    }

    /// Jump forward, equivalently to 2^384 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^128 starting points, from each of which
    /// `jump()` will generate 2^128 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(
            u64,
            self,
            [
                0x11467fef8f921d28,
                0xa2a819f2e79c8ea8,
                0xa8299fc284b3959a,
                0xb4d347340ca63ee1,
                0x1cb0940bedbff6ce,
                0xd956c5c4fa1f8e17,
                0x915e38fd4eda93bc,
                0x5b3ccdfa5d7daca5
            ]
        );
    }

    /// Jump forward by c · 2^e calls to `next_u64()`.
    ///
    /// For example, `jump_ce(1, 256)` is equivalent to [`jump`](Self::jump)
    /// and `jump_ce(1, 384)` is equivalent to [`long_jump`](Self::long_jump).
    /// Expressing the distance as c · 2^e makes it possible to request both
    /// ordinary counts (`jump_ce(k, 0)`) and very large power-of-two jumps
    /// without multiple-precision integers. For the jump to be meaningful,
    /// c · 2^e should be smaller than the period 2^512 − 1.
    ///
    /// See [`jump_n`](Self::jump_n) to jump by an arbitrary distance.
    pub fn jump_ce(&mut self, c: u64, e: u32) {
        impl_jump_ce!(
            u64,
            self,
            [
                0xcf3cff0c00000001,
                0x7fdc78d886f00c63,
                0xf05e63fca6d7b781,
                0x7a67058e7bbab6f0,
                0xf11eef832e32518f,
                0x51ba7c47edc758ad,
                0x8f2d27268ce4b20b,
                0x0000500055d8b77f
            ],
            c,
            e,
            array8
        );
    }

    /// Jump forward by an arbitrary number of calls to `next_u64()`.
    ///
    /// This is equivalent to *n* calls to `next_u64()`, where *n* = `jump[0]` +
    /// `jump[1]` · 2^64 + … is the little-endian integer held in `jump`.
    /// Unlike [`jump_ce`](Self::jump_ce), it can express any jump distance.
    /// For the jump to be meaningful, *n* should be smaller than the period
    /// 2^512 − 1.
    pub fn jump_n(&mut self, jump: &[u64; 8]) {
        impl_jump_n!(
            u64,
            self,
            [
                0xcf3cff0c00000001,
                0x7fdc78d886f00c63,
                0xf05e63fca6d7b781,
                0x7a67058e7bbab6f0,
                0xf11eef832e32518f,
                0x51ba7c47edc758ad,
                0x8f2d27268ce4b20b,
                0x0000500055d8b77f
            ],
            jump,
            array8
        );
    }
}

impl_state_seed512!(Xoshiro512Plus);

impl SeedableRng for Xoshiro512Plus {
    type Seed = Seed512;

    /// Create a new `Xoshiro512Plus`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    #[inline]
    fn from_seed(seed: Seed512) -> Xoshiro512Plus {
        Xoshiro512Plus {
            s: utils::read_words(crate::common::zero_seed_fallback(&seed.0)),
        }
    }

    /// Seed a `Xoshiro512Plus` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoshiro512Plus {
        from_splitmix!(seed)
    }
}

impl TryRng for Xoshiro512Plus {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        // The lowest bits have some linear dependencies, so we use the
        // upper bits instead.
        Ok((self.next_u64() >> 32) as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let result_plus = self.s[0].wrapping_add(self.s[2]);
        impl_xoshiro_large!(self);
        Ok(result_plus)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dest, || self.try_next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Xoshiro512Plus {
        Xoshiro512Plus::seed_from_u64(0x0123456789abcdef)
    }

    #[test]
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = fresh();
            for _ in 0..d {
                a.next_u64();
            }
            let mut b = fresh();
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        let mut a = fresh();
        for _ in 0..3 * 256 {
            a.next_u64();
        }
        let mut b = fresh();
        b.jump_ce(3, 8);
        assert_eq!(a, b, "jump_ce(3, 8)");
    }

    #[test]
    fn jump_ce_matches_predefined_jumps() {
        let mut a = fresh();
        a.jump();
        let mut b = fresh();
        b.jump_ce(1, 256);
        assert_eq!(a, b, "jump_ce(1,256) == jump()");

        let mut a = fresh();
        a.long_jump();
        let mut b = fresh();
        b.jump_ce(1, 384);
        assert_eq!(a, b, "jump_ce(1,384) == long_jump()");
    }

    #[test]
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0, …]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = fresh();
            a.jump_ce(d, 0);
            let mut b = fresh();
            b.jump_n(&[d, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, …])");
        }
        // A distance that needs a high limb: 2^256 == jump().
        let mut a = fresh();
        a.jump();
        let mut b = fresh();
        b.jump_n(&[0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(a, b, "jump_n(2^256) == jump()");

        // A distance jump_ce cannot express (odd part exceeds 64 bits):
        // 3 + 5 · 2^64, checked via x^(a + b) = x^a · x^b.
        let mut a = fresh();
        a.jump_ce(3, 0);
        a.jump_ce(5, 64);
        let mut b = fresh();
        b.jump_n(&[3, 5, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a, b, "jump_n(3 + 5 · 2^64)");
    }

    #[test]
    fn reference() {
        #[rustfmt::skip]
        let mut rng = Xoshiro512Plus::from_seed(Seed512(
            [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
             3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
             5, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
             7, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]));
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro512plus.c
        let expected = [
            4,
            8,
            4113,
            25169936,
            52776585412635,
            57174648719367,
            9223482039571869716,
            9331471677901559830,
            9340533895746033672,
            14078399799840753678,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    fn zero_seed_maps_to_seed_from_u64_zero() {
        let from_zero = Xoshiro512Plus::from_seed(Seed512([0u8; 64]));
        let from_sm0 = Xoshiro512Plus::seed_from_u64(0);
        assert_eq!(from_zero, from_sm0);
    }

    #[test]
    fn state_roundtrip() {
        let rng = Xoshiro512Plus::seed_from_u64(42);
        let clone = Xoshiro512Plus::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
