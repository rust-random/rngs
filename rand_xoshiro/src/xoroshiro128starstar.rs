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

/// A xoroshiro128** random number generator.
///
/// The xoroshiro128** algorithm is not suitable for cryptographic purposes, but
/// is very fast and has excellent statistical properties.
///
/// The algorithm used here is translated from [the `xoroshiro128starstar.c`
/// reference source code](http://xoshiro.di.unimi.it/xoroshiro128starstar.c) by
/// David Blackman and Sebastiano Vigna.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Xoroshiro128StarStar {
    s0: u64,
    s1: u64,
}

impl Xoroshiro128StarStar {
    /// Jump forward, equivalently to 2^64 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^64 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoroshiro128StarStar;
    ///
    /// let rng1 = Xoroshiro128StarStar::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(u64, self, [0xdf900294d8f554a5, 0x170865df4b3201fc]);
    }

    /// Jump forward, equivalently to 2^96 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^32 starting points, from each of which
    /// `jump()` will generate 2^32 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(u64, self, [0xd2a98b26625eee7b, 0xdddf9b1090aa7ac1]);
    }

    /// Jump forward by c · 2^e calls to `next_u64()`.
    ///
    /// For example, `jump_ce(1, 64)` is equivalent to [`jump`](Self::jump)
    /// and `jump_ce(1, 96)` is equivalent to [`long_jump`](Self::long_jump).
    /// Expressing the distance as c · 2^e makes it possible to request both
    /// ordinary counts (`jump_ce(k, 0)`) and very large power-of-two jumps
    /// without multiple-precision integers. For the jump to be meaningful,
    /// c · 2^e should be smaller than the period 2^128 − 1.
    ///
    /// See [`jump_n`](Self::jump_n) to jump by an arbitrary distance.
    pub fn jump_ce(&mut self, c: u64, e: u32) {
        impl_jump_ce!(
            u64,
            self,
            [0x095b8f76579aa001, 0x0008828e513b43d5],
            c,
            e,
            pair
        );
    }

    /// Jump forward by an arbitrary number of calls to `next_u64()`.
    ///
    /// This is equivalent to *n* calls to `next_u64()`, where *n* = `jump[0]` +
    /// `jump[1]` · 2^64 + … is the little-endian integer held in `jump`.
    /// Unlike [`jump_ce`](Self::jump_ce), it can express any jump distance.
    /// For the jump to be meaningful, *n* should be smaller than the period
    /// 2^128 − 1.
    pub fn jump_n(&mut self, jump: &[u64; 2]) {
        impl_jump_n!(
            u64,
            self,
            [0x095b8f76579aa001, 0x0008828e513b43d5],
            jump,
            pair
        );
    }
}

impl TryRng for Xoroshiro128StarStar {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.next_u64() as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let r = starstar_u64!(self.s0);
        impl_xoroshiro_u64!(self);
        Ok(r)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dest, || self.try_next_u64())
    }
}

impl_state_pair!(Xoroshiro128StarStar, u64);

impl SeedableRng for Xoroshiro128StarStar {
    type Seed = [u8; 16];

    /// Create a new `Xoroshiro128StarStar`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    fn from_seed(seed: [u8; 16]) -> Xoroshiro128StarStar {
        let s: [_; 2] = utils::read_words(crate::common::zero_seed_fallback(&seed));

        Xoroshiro128StarStar { s0: s[0], s1: s[1] }
    }

    /// Seed a `Xoroshiro128StarStar` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoroshiro128StarStar {
        from_splitmix!(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Xoroshiro128StarStar {
        Xoroshiro128StarStar::seed_from_u64(0x0123456789abcdef)
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
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
        b.jump_ce(1, 64);
        assert_eq!(a, b, "jump_ce(1,64) == jump()");

        let mut a = fresh();
        a.long_jump();
        let mut b = fresh();
        b.jump_ce(1, 96);
        assert_eq!(a, b, "jump_ce(1,96) == long_jump()");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = fresh();
            a.jump_ce(d, 0);
            let mut b = fresh();
            b.jump_n(&[d, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, …])");
        }
        // A distance that needs a high limb: 2^64 == jump().
        let mut a = fresh();
        a.jump();
        let mut b = fresh();
        b.jump_n(&[0, 1]);
        assert_eq!(a, b, "jump_n(2^64) == jump()");

        // A distance jump_ce cannot express (odd part exceeds 64 bits):
        // 3 + 5 · 2^64, checked via x^(a + b) = x^a · x^b.
        let mut a = fresh();
        a.jump_ce(3, 0);
        a.jump_ce(5, 64);
        let mut b = fresh();
        b.jump_n(&[3, 5]);
        assert_eq!(a, b, "jump_n(3 + 5 · 2^64)");
    }

    #[test]
    fn reference() {
        let mut rng =
            Xoroshiro128StarStar::from_seed([1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128starstar.c
        let expected = [
            5760,
            97769243520,
            9706862127477703552,
            9223447511460779954,
            8358291023205304566,
            15695619998649302768,
            8517900938696309774,
            16586480348202605369,
            6959129367028440372,
            16822147227405758281,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    fn zero_seed_maps_to_seed_from_u64_zero() {
        let from_zero = Xoroshiro128StarStar::from_seed([0u8; 16]);
        let from_sm0 = Xoroshiro128StarStar::seed_from_u64(0);
        assert_eq!(from_zero, from_sm0);
    }

    #[test]
    fn state_roundtrip() {
        let rng = Xoroshiro128StarStar::seed_from_u64(42);
        let clone = Xoroshiro128StarStar::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
