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

/// A xoshiro128++ random number generator.
///
/// The xoshiro128++ algorithm is not suitable for cryptographic purposes, but
/// is very fast and has excellent statistical properties.
///
/// The algorithm used here is translated from [the `xoshiro128plusplus.c`
/// reference source code](http://xoshiro.di.unimi.it/xoshiro128plusplus.c) by
/// David Blackman and Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Xoshiro128PlusPlus {
    s: [u32; 4],
}

impl Xoshiro128PlusPlus {
    /// Jump forward, equivalently to 2^64 calls to `next_u32()`.
    ///
    /// This can be used to generate 2^64 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoroshiro128PlusPlus;
    ///
    /// let rng1 = Xoroshiro128PlusPlus::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(u32, self, [0x8764000b, 0xf542d2d3, 0x6fa035c3, 0x77f2db5b]);
    }

    /// Jump forward, equivalently to 2^96 calls to `next_u32()`.
    ///
    /// This can be used to generate 2^32 starting points, from each of which
    /// `jump()` will generate 2^32 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(u32, self, [0xb523952e, 0x0b6f099f, 0xccf5a0ef, 0x1c580662]);
    }

    /// Jump forward by c · 2^e calls to `next_u32()`.
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
            u32,
            self,
            [0x1b489db6de18fc01, 0x00fc65a2006254b1],
            c,
            e,
            array4
        );
    }

    /// Jump forward by an arbitrary number of calls to `next_u32()`.
    ///
    /// This is equivalent to *n* calls to `next_u32()`, where *n* = `jump[0]` +
    /// `jump[1]` · 2^64 is the little-endian integer held in `jump`.
    /// Unlike [`jump_ce`](Self::jump_ce), it can express any jump distance.
    /// For the jump to be meaningful, *n* should be smaller than the period
    /// 2^128 − 1.
    pub fn jump_n(&mut self, jump: &[u64; 2]) {
        impl_jump_n!(
            u32,
            self,
            [0x1b489db6de18fc01, 0x00fc65a2006254b1],
            jump,
            array4
        );
    }
}

impl_state_array_of_four!(Xoshiro128PlusPlus, u32);

impl SeedableRng for Xoshiro128PlusPlus {
    type Seed = [u8; 16];

    /// Create a new `Xoshiro128PlusPlus`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    #[inline]
    fn from_seed(seed: [u8; 16]) -> Xoshiro128PlusPlus {
        Xoshiro128PlusPlus {
            s: utils::read_words(crate::common::zero_seed_fallback(&seed)),
        }
    }

    /// Seed a `Xoshiro128PlusPlus` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoshiro128PlusPlus {
        from_splitmix!(seed)
    }
}

impl TryRng for Xoshiro128PlusPlus {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let result_starstar = plusplus_u32!(self.s[0], self.s[3]);
        impl_xoshiro_u32!(self);
        Ok(result_starstar)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        utils::next_u64_via_u32(self)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dest, || self.try_next_u32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Xoshiro128PlusPlus {
        Xoshiro128PlusPlus::seed_from_u64(0x0123456789abcdef)
    }

    #[test]
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = fresh();
            for _ in 0..d {
                a.next_u32();
            }
            let mut b = fresh();
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        let mut a = fresh();
        for _ in 0..3 * 256 {
            a.next_u32();
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
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = fresh();
            a.jump_ce(d, 0);
            let mut b = fresh();
            b.jump_n(&[d, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, 0])");
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
            Xoshiro128PlusPlus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plusplus.c
        let expected = [
            641, 1573767, 3222811527, 3517856514, 836907274, 4247214768, 3867114732, 1355841295,
            495546011, 621204420,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    fn test_jump() {
        let mut rng =
            Xoshiro128PlusPlus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        rng.jump();
        // These values were produced by instrumenting the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plus.c
        assert_eq!(rng.s[0], 2843103750);
        assert_eq!(rng.s[1], 2038079848);
        assert_eq!(rng.s[2], 1533207345);
        assert_eq!(rng.s[3], 44816753);
    }

    #[test]
    fn test_long_jump() {
        let mut rng =
            Xoshiro128PlusPlus::from_seed([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        rng.long_jump();
        // These values were produced by instrumenting the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plus.c
        assert_eq!(rng.s[0], 1611968294);
        assert_eq!(rng.s[1], 2125834322);
        assert_eq!(rng.s[2], 966769569);
        assert_eq!(rng.s[3], 3193880526);
    }

    #[test]
    fn zero_seed_maps_to_seed_from_u64_zero() {
        let from_zero = Xoshiro128PlusPlus::from_seed([0u8; 16]);
        let from_sm0 = Xoshiro128PlusPlus::seed_from_u64(0);
        assert_eq!(from_zero, from_sm0);
    }

    #[test]
    fn state_roundtrip() {
        let rng = Xoshiro128PlusPlus::seed_from_u64(42);
        let clone = Xoshiro128PlusPlus::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
