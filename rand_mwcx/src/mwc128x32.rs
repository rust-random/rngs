// Copyright 2026 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::mp;
use core::convert::Infallible;
use rand_core::{Rng, SeedableRng, TryRng, utils};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const MWC_A3: u32 = 0xffcccc66;

/// The modulus m = `MWC_A3` · 2⁹⁶ − 1.
const MOD: [u64; 2] = [u64::MAX, ((MWC_A3 - 1) as u64) << 32 | 0xffffffff];

/// The multiplier 2⁻³² mod m = `MWC_A3` · 2⁶⁴ of the underlying generator.
const BASE: [u64; 2] = [0, MWC_A3 as u64];

/// An MWC128X/32 random number generator.
///
/// MWC128X/32 is a Marsaglia multiply-with-carry generator with 32-bit digits,
/// lag three, and period approximately 2¹²⁷, whose output is scrambled by X:
/// the result is the most recent digit xored with the carry, as in David
/// Thomas's MWC64X generator. It is not suitable for cryptographic purposes,
/// but it is very fast: its only 64-bit operations are a multiplication and a
/// sum, and, since the lag is three, each multiplication uses a digit computed
/// three steps before, so the processor can overlap the multiplications of
/// consecutive steps (only the carry must be passed from one step to the next).
///
/// The underlying generator simulates a multiplicative congruential generator
/// with prime modulus *m* = 0xffcccc66 · 2⁹⁶ − 1 and multiplier 2⁻³² mod *m*,
/// that is, the multiplicative inverse of 2³² modulo *m*, which is 0xffcccc66 ·
/// 2⁶⁴ (their product is *m* + 1). The special form of the modulus forces a
/// defect in the lattice structure, which the scrambler X hides.
///
/// The algorithm used here is translated from [the `MWC128X_32.c` reference
/// source code](https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X_32.c)
/// by Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mwc128X32 {
    x: u32,
    y: u32,
    z: u32,
    c: u32,
}

impl Mwc128X32 {
    /// Return the state as the integer
    /// *x* + *y* · 2³² + *z* · 2⁶⁴ + *c* · 2⁹⁶.
    fn to_words(&self) -> [u64; 2] {
        [
            self.x as u64 | (self.y as u64) << 32,
            self.z as u64 | (self.c as u64) << 32,
        ]
    }

    /// Create a generator from the state as returned by
    /// [`to_words`](Self::to_words).
    fn from_words(s: [u64; 2]) -> Mwc128X32 {
        Mwc128X32 {
            x: s[0] as u32,
            y: (s[0] >> 32) as u32,
            z: s[1] as u32,
            c: (s[1] >> 32) as u32,
        }
    }

    /// Multiply the state, read as an integer, by `r` modulo *m*. Since each
    /// step multiplies the state by *A* = 2⁻³² mod *m*, jumping ahead by *n*
    /// steps multiplies it by *Aⁿ* mod *m*.
    fn jump_mul(&mut self, r: &[u64; 2]) {
        *self = Self::from_words(mp::mul_mod(&self.to_words(), r, &MOD));
    }

    /// Jump forward, equivalently to 2⁶⁴ calls to `next_u32()`.
    ///
    /// This can be used to generate 2⁶³ non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_mwcx::rand_core::SeedableRng;
    /// use rand_mwcx::Mwc128X32;
    ///
    /// let rng1 = Mwc128X32::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        self.jump_mul(&[0x3df51ad941ad5cca, 0xe67750f4f6c4e62f]);
    }

    /// Jump forward, equivalently to 2⁹⁶ calls to `next_u32()`.
    ///
    /// This can be used to generate 2³¹ starting points, from each of which
    /// `jump()` will generate 2³² non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        self.jump_mul(&[0x8cfada360d80d5a5, 0xfef31f136713bbb4]);
    }

    /// Jump forward by *c* · 2*ᵉ* calls to `next_u32()`.
    ///
    /// For example, `jump_ce(1, 64)` is equivalent to [`jump`](Self::jump)
    /// and `jump_ce(1, 96)` is equivalent to [`long_jump`](Self::long_jump).
    /// Expressing the distance as *c* · 2*ᵉ* makes it possible to request both
    /// ordinary counts (`jump_ce(k, 0)`) and very large power-of-two jumps
    /// without multiple-precision integers. For the jump to be meaningful,
    /// *c* · 2*ᵉ* should be smaller than the period (approximately 2¹²⁷).
    ///
    /// See [`jump_n`](Self::jump_n) to jump by an arbitrary distance.
    pub fn jump_ce(&mut self, c: u64, e: u32) {
        self.jump_mul(&mp::pow_mod_ce(&BASE, c, e, &MOD));
    }

    /// Jump forward by an arbitrary number of calls to `next_u32()`.
    ///
    /// This is equivalent to *n* calls to `next_u32()`, where *n* = `jump[0]` +
    /// `jump[1]` · 2⁶⁴ is the little-endian integer held in `jump`. Unlike
    /// [`jump_ce`](Self::jump_ce), it can express any jump distance. For the
    /// jump to be meaningful, *n* should be smaller than the period
    /// (approximately 2¹²⁷).
    pub fn jump_n(&mut self, jump: &[u64; 2]) {
        self.jump_mul(&mp::pow_mod(&BASE, jump, &MOD));
    }

    /// Return the internal state of the generator as little-endian bytes
    /// that can be passed to [`SeedableRng::from_seed`] to reconstruct an
    /// identical generator.
    ///
    /// [`SeedableRng::from_seed`]: rand_core::SeedableRng::from_seed
    pub fn state(&self) -> [u8; 16] {
        crate::common::to_bytes(self.to_words())
    }

    // Create a new `Mwc128X32` from random bytes. The value r used for the carry
    // is read as a fixed-point fraction r / 2³² in [0..1) and multiplied by
    // MWC_A3 - 2, the size of the range of valid carries: the carry is
    // 1 + (r * (MWC_A3 - 2) >> 32).
    fn from_random_bytes(seed: [u8; 16]) -> Mwc128X32 {
        let [x, y, z, r] = utils::read_words(&seed);
        Mwc128X32 {
            x,
            y,
            z,
            c: 1 + ((r as u64 * (MWC_A3 - 2) as u64) >> 32) as u32,
        }
    }
}

impl SeedableRng for Mwc128X32 {
    type Seed = [u8; 16];

    /// Create a new `Mwc128X32`.
    ///
    /// The seed is the little-endian representation of the state, that is,
    /// of the integer *s* = *x* + *y* · 2³² + *z* · 2⁶⁴ + *c* · 2⁹⁶, where
    /// *x*, *y*, and *z* are the digits and *c* is the carry. If *s* is not a
    /// valid state, that is, if *s* is zero or not smaller than the modulus
    /// *m* = 0xffcccc66 · 2⁹⁶ − 1, the carry is set to one.
    #[inline]
    fn from_seed(seed: [u8; 16]) -> Mwc128X32 {
        // The seed is an explicit state: unlike in seed_from_u64() and
        // from_rng(), the carry is not mapped into the range of valid carries.
        let s = utils::read_words(&seed);
        let mut rng = Self::from_words(s);
        if !mp::is_valid(&s, &MOD) {
            rng.c = 1;
        }
        rng
    }

    /// Seed an `Mwc128X32` from a `u64` as suggested by the reference
    /// implementation: the digits and the carry are initialized to
    /// valid values using the output of a SplitMix generator seeded with `seed`.
    fn seed_from_u64(seed: u64) -> Mwc128X32 {
        Mwc128X32::from_random_bytes(crate::common::splitmix_bytes::<16>(seed))
    }

    /// Create a new `Mwc128X32`, initializing the digits and the carry to valid
    /// values using the output of `rng`.
    fn from_rng<R: Rng + ?Sized>(rng: &mut R) -> Mwc128X32 {
        let mut seed = [0; 16];
        rng.fill_bytes(&mut seed);
        Mwc128X32::from_random_bytes(seed)
    }

    /// Create a new `Mwc128X32`, initializing the digits and the carry to valid
    /// values using the output of `rng`.
    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Mwc128X32, R::Error> {
        let mut seed = [0; 16];
        rng.try_fill_bytes(&mut seed)?;
        Ok(Mwc128X32::from_random_bytes(seed))
    }
}

impl TryRng for Mwc128X32 {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let result = self.z ^ self.c;
        let t = MWC_A3 as u64 * self.x as u64 + self.c as u64;
        self.x = self.y;
        self.y = self.z;
        self.z = t as u32;
        self.c = (t >> 32) as u32;
        Ok(result)
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

    // x = 0x89abcdef, y = 0x76543210, z = 0x4b5a6978, c = 1
    const SEED: [u8; 16] = [
        0xef, 0xcd, 0xab, 0x89, 0x10, 0x32, 0x54, 0x76, 0x78, 0x69, 0x5a, 0x4b, 1, 0, 0, 0,
    ];

    #[test]
    fn reference() {
        let mut rng = Mwc128X32::from_seed(SEED);
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X_32.c
        let expected = [
            0x4b5a6979, 0x6792c5cf, 0xedfc7024, 0x50a57a05, 0x329b3e06, 0x1f1db247, 0x4c7e40d2,
            0x3cbaba78, 0x687c6b34, 0xe8c71e6d,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn reference_jumps() {
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X_32.c
        let jumped = |f: fn(&mut Mwc128X32)| {
            let mut rng = Mwc128X32::from_seed(SEED);
            f(&mut rng);
            rng
        };
        let expected = |x, y, z, c| Mwc128X32 { x, y, z, c };
        assert_eq!(
            jumped(Mwc128X32::jump),
            expected(0x69ec7c23, 0x85620075, 0xa5dc93ac, 0x4e034463)
        );
        assert_eq!(
            jumped(Mwc128X32::long_jump),
            expected(0xee6eebbd, 0x140f21ca, 0x45518300, 0x1321a37d)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(3, 70)),
            expected(0x8cb9387d, 0x68a3bfad, 0x2d7e249b, 0x7b14417a)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(12345, 0)),
            expected(0x2602fd2c, 0xd27ae5a4, 0x6dc5ec3d, 0x593ac844)
        );
        assert_eq!(
            jumped(|rng| rng.jump_n(&[0x9e3779b97f4a7c15, 0x0123456789abcdef])),
            expected(0xb374e488, 0x31d87421, 0xfb798f94, 0x2fe2152b)
        );
    }

    #[test]
    fn reference_seed_from_u64() {
        let mut rng = Mwc128X32::seed_from_u64(42);
        assert_eq!(
            rng,
            Mwc128X32 {
                x: 0x2feb6e95,
                y: 0xbdd73226,
                z: 0xb266f103,
                c: 0x28e7b329
            }
        );
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X_32.c
        let expected = [0x9a81422a, 0x4971a782, 0xa26f262a, 0x5464e7be];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc128X32::from_seed(SEED);
            for _ in 0..d {
                a.next_u32();
            }
            let mut b = Mwc128X32::from_seed(SEED);
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        // c * 2ᵉ with e > 0, small enough to step literally: 3 * 2⁸ = 768.
        let mut a = Mwc128X32::from_seed(SEED);
        for _ in 0..3 * 256 {
            a.next_u32();
        }
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_ce(3, 8);
        assert_eq!(a, b, "jump_ce(3, 8)");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_matches_predefined_jumps() {
        let mut a = Mwc128X32::from_seed(SEED);
        a.jump();
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_ce(1, 64);
        assert_eq!(a, b, "jump_ce(1, 64) == jump()");

        let mut a = Mwc128X32::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_ce(1, 96);
        assert_eq!(a, b, "jump_ce(1, 96) == long_jump()");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc128X32::from_seed(SEED);
            a.jump_ce(d, 0);
            let mut b = Mwc128X32::from_seed(SEED);
            b.jump_n(&[d, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, 0])");
        }
        // Distances that need the high word: 2⁶⁴ == jump(), 2⁹⁶ == long_jump().
        let mut a = Mwc128X32::from_seed(SEED);
        a.jump();
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_n(&[0, 1]);
        assert_eq!(a, b, "jump_n(2^64) == jump()");

        let mut a = Mwc128X32::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_n(&[0, 1 << 32]);
        assert_eq!(a, b, "jump_n(2^96) == long_jump()");

        // A distance jump_ce cannot express (odd part exceeds 64 bits):
        // 3 + 5 · 2⁶⁴, checked via A^(a + b) = A^a · A^b.
        let mut a = Mwc128X32::from_seed(SEED);
        a.jump_ce(3, 0);
        a.jump_ce(5, 64);
        let mut b = Mwc128X32::from_seed(SEED);
        b.jump_n(&[3, 5]);
        assert_eq!(a, b, "jump_n(3 + 5 · 2^64)");
    }

    #[test]
    fn from_rng_maps_carry() {
        use crate::common::ConstRng;
        // Zero bytes give carry one; all-one bytes give the largest carry
        // produced by the mapping, a - 2.
        assert_eq!(
            Mwc128X32::from_rng(&mut ConstRng(0)),
            Mwc128X32 {
                x: 0,
                y: 0,
                z: 0,
                c: 1
            }
        );
        let largest = Mwc128X32 {
            x: u32::MAX,
            y: u32::MAX,
            z: u32::MAX,
            c: MWC_A3 - 2,
        };
        assert_eq!(Mwc128X32::from_rng(&mut ConstRng(0xff)), largest);
        assert_eq!(Mwc128X32::try_from_rng(&mut ConstRng(0xff)), Ok(largest));
    }

    #[test]
    fn invalid_seeds() {
        let from_words = |s: [u64; 2]| Mwc128X32::from_seed(crate::common::to_bytes(s));
        let with_carry = |s: [u64; 2], c| Mwc128X32 {
            c,
            ..Mwc128X32::from_words(s)
        };
        // The zero state and the states not smaller than m get carry one.
        assert_eq!(from_words([0; 2]), with_carry([0; 2], 1));
        assert_eq!(from_words(MOD), with_carry(MOD, 1));
        assert_eq!(from_words([5, u64::MAX]), with_carry([5, u64::MAX], 1));
        // The other states, including those with zero or maximum carry, are
        // left unchanged.
        let below = [u64::MAX - 1, MOD[1]];
        assert_eq!(from_words(below), Mwc128X32::from_words(below));
        assert_eq!(from_words([1, 0]), Mwc128X32::from_words([1, 0]));
    }

    #[test]
    fn state_roundtrip() {
        let mut rng = Mwc128X32::seed_from_u64(42);
        rng.next_u32();
        let clone = Mwc128X32::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
