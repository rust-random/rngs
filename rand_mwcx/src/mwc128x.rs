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

const MWC_A1: u64 = 0xff6da236274e252e;

/// The modulus m = `MWC_A1` · 2⁶⁴ − 1.
const MOD: [u64; 2] = [u64::MAX, MWC_A1 - 1];

/// The multiplier 2⁻⁶⁴ mod m = `MWC_A1` of the underlying generator.
const BASE: [u64; 2] = [MWC_A1, 0];

/// An MWC128X random number generator.
///
/// MWC128X is a Marsaglia multiply-with-carry generator with 64-bit digits,
/// lag one, and period approximately 2¹²⁷, whose output is scrambled by X:
/// the result is the digit xored with the carry, as in David Thomas's MWC64X
/// generator. It is not suitable for cryptographic purposes, but it is very
/// fast, as its only 128-bit operations are a multiplication and a sum; since
/// the lag is one, however, each multiplication needs the result of the
/// previous step, so the processor cannot overlap consecutive steps
/// ([`Mwc192X`](crate::Mwc192X), which has lag two, is faster for this
/// reason).
///
/// The underlying generator simulates a multiplicative congruential generator
/// with prime modulus *m* = 0xff6da236274e252e · 2⁶⁴ − 1 and multiplier 2⁻⁶⁴
/// mod *m*, that is, the multiplicative inverse of 2⁶⁴ modulo *m*, which is
/// 0xff6da236274e252e (their product is *m* + 1). The special form of the
/// modulus forces a defect in the lattice structure, which the scrambler X
/// hides.
///
/// The algorithm used here is translated from [the `MWC128X.c` reference
/// source code](https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X.c)
/// by Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mwc128X {
    x: u64,
    c: u64,
}

impl Mwc128X {
    /// Multiply the state, read as the integer *x* + *c* · 2⁶⁴, by `r`
    /// modulo *m*. Since each step multiplies the state by
    /// *A* = 2⁻⁶⁴ mod *m*, jumping ahead by *n* steps multiplies it by
    /// *Aⁿ* mod *m*.
    fn jump_mul(&mut self, r: &[u64; 2]) {
        [self.x, self.c] = mp::mul_mod(&[self.x, self.c], r, &MOD);
    }

    /// Jump forward, equivalently to 2⁶⁴ calls to `next_u64()`.
    ///
    /// This can be used to generate 2⁶³ non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_mwcx::rand_core::SeedableRng;
    /// use rand_mwcx::Mwc128X;
    ///
    /// let rng1 = Mwc128X::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        self.jump_mul(&[0xe059ea8b904c9106, 0xfcf4edde2b624961]);
    }

    /// Jump forward, equivalently to 2⁹⁶ calls to `next_u64()`.
    ///
    /// This can be used to generate 2³¹ starting points, from each of which
    /// `jump()` will generate 2³² non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        self.jump_mul(&[0x989e457a74d91207, 0x4fc2c97ce5369e74]);
    }

    /// Jump forward by *c* · 2*ᵉ* calls to `next_u64()`.
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

    /// Jump forward by an arbitrary number of calls to `next_u64()`.
    ///
    /// This is equivalent to *n* calls to `next_u64()`, where *n* = `jump[0]` +
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
        crate::common::to_bytes([self.x, self.c])
    }

    // Create a new `Mwc128X` from random bytes. The value r used for the carry
    // is read as a fixed-point fraction r / 2⁶⁴ in [0..1) and multiplied by
    // MWC_A1 - 2, the size of the range of valid carries: the carry is
    // 1 + (r * (MWC_A1 - 2) >> 64).
    fn from_random_bytes(seed: [u8; 16]) -> Mwc128X {
        let [x, r] = utils::read_words(&seed);
        Mwc128X {
            x,
            c: 1 + ((r as u128 * (MWC_A1 - 2) as u128) >> 64) as u64,
        }
    }
}

impl SeedableRng for Mwc128X {
    type Seed = [u8; 16];

    /// Create a new `Mwc128X`.
    ///
    /// The seed is the little-endian representation of the state, that is,
    /// of the integer *s* = *x* + *c* · 2⁶⁴, where *x* is the digit and *c*
    /// is the carry. If *s* is not a valid state, that is, if *s* is zero or
    /// not smaller than the modulus *m* = 0xff6da236274e252e · 2⁶⁴ − 1, the
    /// carry is set to one.
    #[inline]
    fn from_seed(seed: [u8; 16]) -> Mwc128X {
        // The seed is an explicit state: unlike in seed_from_u64() and
        // from_rng(), the carry is not mapped into the range of valid carries.
        let s = utils::read_words(&seed);
        let [x, c] = s;
        Mwc128X {
            x,
            c: if mp::is_valid(&s, &MOD) { c } else { 1 },
        }
    }

    /// Seed an `Mwc128X` from a `u64` as suggested by the reference
    /// implementation: the digit and the carry are initialized to
    /// valid values using the output of a SplitMix generator seeded with `seed`.
    fn seed_from_u64(seed: u64) -> Mwc128X {
        Mwc128X::from_random_bytes(crate::common::splitmix_bytes::<16>(seed))
    }

    /// Create a new `Mwc128X`, initializing the digit and the carry to valid
    /// values using the output of `rng`.
    fn from_rng<R: Rng + ?Sized>(rng: &mut R) -> Mwc128X {
        let mut seed = [0; 16];
        rng.fill_bytes(&mut seed);
        Mwc128X::from_random_bytes(seed)
    }

    /// Create a new `Mwc128X`, initializing the digit and the carry to valid
    /// values using the output of `rng`.
    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Mwc128X, R::Error> {
        let mut seed = [0; 16];
        rng.try_fill_bytes(&mut seed)?;
        Ok(Mwc128X::from_random_bytes(seed))
    }
}

impl TryRng for Mwc128X {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok((self.next_u64() >> 32) as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let result = self.x ^ self.c;
        let t = MWC_A1 as u128 * self.x as u128 + self.c as u128;
        self.x = t as u64;
        self.c = (t >> 64) as u64;
        Ok(result)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dest, || self.try_next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // x = 0x0123456789abcdef, c = 1
    const SEED: [u8; 16] = [
        0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 1, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn reference() {
        let mut rng = Mwc128X::from_seed(SEED);
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X.c
        let expected = [
            0x0123456789abcdee,
            0xba17a27d87b42a4a,
            0xa622a4b120a9f538,
            0x1bb7f645c6670559,
            0x18529bad6097d72b,
            0xf501042c00493549,
            0x37e0328b3ce80699,
            0x958cd94f7eb2e02f,
            0x68e32730e418e30a,
            0x4519e7cc8909e77d,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn reference_jumps() {
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X.c
        let jumped = |f: fn(&mut Mwc128X)| {
            let mut rng = Mwc128X::from_seed(SEED);
            f(&mut rng);
            rng
        };
        let expected = |x, c| Mwc128X { x, c };
        assert_eq!(
            jumped(Mwc128X::jump),
            expected(0x5a936408afe2e245, 0xe471051f76a629fe)
        );
        assert_eq!(
            jumped(Mwc128X::long_jump),
            expected(0x7e2cba541c5f4c51, 0x30a344ed8d373892)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(3, 70)),
            expected(0x34d3926d1a7906f8, 0x131cbd0e516ab608)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(12345, 0)),
            expected(0xe7ef89c7c08b0a84, 0x63f3b10034fa793a)
        );
        assert_eq!(
            jumped(|rng| rng.jump_n(&[0x9e3779b97f4a7c15, 0x0123456789abcdef])),
            expected(0x9724571fae3a4c3b, 0xec86a5e18074080a)
        );
    }

    #[test]
    fn reference_seed_from_u64() {
        let mut rng = Mwc128X::seed_from_u64(42);
        assert_eq!(
            rng,
            Mwc128X {
                x: 0xbdd732262feb6e95,
                c: 0x28d87b64b2612e5a
            }
        );
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC128X.c
        let expected = [
            0x950f49429d8a40cf,
            0xadadbfee0054f2ad,
            0xcdbcae4e7346dc25,
            0x512fd18f7df72000,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc128X::from_seed(SEED);
            for _ in 0..d {
                a.next_u64();
            }
            let mut b = Mwc128X::from_seed(SEED);
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        // c * 2ᵉ with e > 0, small enough to step literally: 3 * 2⁸ = 768.
        let mut a = Mwc128X::from_seed(SEED);
        for _ in 0..3 * 256 {
            a.next_u64();
        }
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_ce(3, 8);
        assert_eq!(a, b, "jump_ce(3, 8)");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_matches_predefined_jumps() {
        let mut a = Mwc128X::from_seed(SEED);
        a.jump();
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_ce(1, 64);
        assert_eq!(a, b, "jump_ce(1, 64) == jump()");

        let mut a = Mwc128X::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_ce(1, 96);
        assert_eq!(a, b, "jump_ce(1, 96) == long_jump()");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc128X::from_seed(SEED);
            a.jump_ce(d, 0);
            let mut b = Mwc128X::from_seed(SEED);
            b.jump_n(&[d, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, 0])");
        }
        // Distances that need the high word: 2⁶⁴ == jump(), 2⁹⁶ == long_jump().
        let mut a = Mwc128X::from_seed(SEED);
        a.jump();
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_n(&[0, 1]);
        assert_eq!(a, b, "jump_n(2^64) == jump()");

        let mut a = Mwc128X::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_n(&[0, 1 << 32]);
        assert_eq!(a, b, "jump_n(2^96) == long_jump()");

        // A distance jump_ce cannot express (odd part exceeds 64 bits):
        // 3 + 5 · 2⁶⁴, checked via A^(a + b) = A^a · A^b.
        let mut a = Mwc128X::from_seed(SEED);
        a.jump_ce(3, 0);
        a.jump_ce(5, 64);
        let mut b = Mwc128X::from_seed(SEED);
        b.jump_n(&[3, 5]);
        assert_eq!(a, b, "jump_n(3 + 5 · 2^64)");
    }

    #[test]
    fn from_rng_maps_carry() {
        use crate::common::ConstRng;
        // Zero bytes give carry one; all-one bytes give the largest carry
        // produced by the mapping, a - 2.
        assert_eq!(Mwc128X::from_rng(&mut ConstRng(0)), Mwc128X { x: 0, c: 1 });
        let largest = Mwc128X {
            x: u64::MAX,
            c: MWC_A1 - 2,
        };
        assert_eq!(Mwc128X::from_rng(&mut ConstRng(0xff)), largest);
        assert_eq!(Mwc128X::try_from_rng(&mut ConstRng(0xff)), Ok(largest));
    }

    #[test]
    fn invalid_seeds() {
        let from_words = |s: [u64; 2]| Mwc128X::from_seed(crate::common::to_bytes(s));
        // The zero state and the states not smaller than m get carry one.
        assert_eq!(from_words([0; 2]), Mwc128X { x: 0, c: 1 });
        assert_eq!(from_words(MOD), Mwc128X { x: u64::MAX, c: 1 });
        assert_eq!(from_words([5, u64::MAX]), Mwc128X { x: 5, c: 1 });
        // The other states, including those with zero or maximum carry, are
        // left unchanged.
        assert_eq!(
            from_words([u64::MAX - 1, MWC_A1 - 1]),
            Mwc128X {
                x: u64::MAX - 1,
                c: MWC_A1 - 1
            }
        );
        assert_eq!(from_words([1, 0]), Mwc128X { x: 1, c: 0 });
    }

    #[test]
    fn state_roundtrip() {
        let mut rng = Mwc128X::seed_from_u64(42);
        rng.next_u64();
        let clone = Mwc128X::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
