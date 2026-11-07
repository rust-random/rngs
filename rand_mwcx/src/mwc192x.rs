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

const MWC_A2: u64 = 0xff4e24dd418f5433;

/// The modulus m = `MWC_A2` · 2¹²⁸ − 1.
const MOD: [u64; 3] = [u64::MAX, u64::MAX, MWC_A2 - 1];

/// The multiplier 2⁻⁶⁴ mod m = `MWC_A2` · 2⁶⁴ of the underlying generator.
const BASE: [u64; 3] = [0, MWC_A2, 0];

/// An MWC192X random number generator.
///
/// MWC192X is a Marsaglia multiply-with-carry generator with 64-bit digits,
/// lag two, and period approximately 2¹⁹¹, whose output is scrambled by X:
/// the result is the most recent digit xored with the carry, as in David
/// Thomas's MWC64X generator. It is not suitable for cryptographic purposes,
/// but it is one of the fastest generators available: its only 128-bit
/// operations are a multiplication and a sum, and, since the lag is two, each
/// multiplication uses a digit computed two steps before, so the processor can
/// overlap the multiplications of consecutive steps (only the carry must be
/// passed from one step to the next).
///
/// The underlying generator simulates a multiplicative congruential generator
/// with prime modulus *m* = 0xff4e24dd418f5433 · 2¹²⁸ − 1 and multiplier 2⁻⁶⁴
/// mod *m*, that is, the multiplicative inverse of 2⁶⁴ modulo *m*, which is
/// 0xff4e24dd418f5433 · 2⁶⁴ (their product is *m* + 1). The special form of the
/// modulus forces a defect in the lattice structure, which the scrambler X
/// hides.
///
/// The algorithm used here is translated from [the `MWC192X.c` reference
/// source code](https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC192X.c)
/// by Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mwc192X {
    x: u64,
    y: u64,
    c: u64,
}

impl Mwc192X {
    /// Multiply the state, read as the integer *x* + *y* · 2⁶⁴ + *c* · 2¹²⁸,
    /// by `r` modulo *m*. Since each step multiplies the state by
    /// *A* = 2⁻⁶⁴ mod *m*, jumping ahead by *n* steps multiplies it by
    /// *Aⁿ* mod *m*.
    fn jump_mul(&mut self, r: &[u64; 3]) {
        [self.x, self.y, self.c] = mp::mul_mod(&[self.x, self.y, self.c], r, &MOD);
    }

    /// Jump forward, equivalently to 2⁹⁶ calls to `next_u64()`.
    ///
    /// This can be used to generate 2⁹⁵ non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_mwcx::rand_core::SeedableRng;
    /// use rand_mwcx::Mwc192X;
    ///
    /// let rng1 = Mwc192X::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        self.jump_mul(&[0xd323a2a406f64957, 0x21239e0fbd1ccbb5, 0x702eb67f0959a76b]);
    }

    /// Jump forward, equivalently to 2¹⁴⁴ calls to `next_u64()`.
    ///
    /// This can be used to generate 2⁴⁷ starting points, from each of which
    /// `jump()` will generate 2⁴⁸ non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        self.jump_mul(&[0x671d31e08a897fff, 0xbceb1c681d8e45f7, 0xd6c731b43ecad305]);
    }

    /// Jump forward by *c* · 2*ᵉ* calls to `next_u64()`.
    ///
    /// For example, `jump_ce(1, 96)` is equivalent to [`jump`](Self::jump)
    /// and `jump_ce(1, 144)` is equivalent to [`long_jump`](Self::long_jump).
    /// Expressing the distance as *c* · 2*ᵉ* makes it possible to request both
    /// ordinary counts (`jump_ce(k, 0)`) and very large power-of-two jumps
    /// without multiple-precision integers. For the jump to be meaningful,
    /// *c* · 2*ᵉ* should be smaller than the period (approximately 2¹⁹¹).
    ///
    /// See [`jump_n`](Self::jump_n) to jump by an arbitrary distance.
    pub fn jump_ce(&mut self, c: u64, e: u32) {
        self.jump_mul(&mp::pow_mod_ce(&BASE, c, e, &MOD));
    }

    /// Jump forward by an arbitrary number of calls to `next_u64()`.
    ///
    /// This is equivalent to *n* calls to `next_u64()`, where *n* = `jump[0]` +
    /// `jump[1]` · 2⁶⁴ + `jump[2]` · 2¹²⁸ is the little-endian integer held
    /// in `jump`. Unlike [`jump_ce`](Self::jump_ce), it can express any jump
    /// distance. For the jump to be meaningful, *n* should be smaller than the
    /// period (approximately 2¹⁹¹).
    pub fn jump_n(&mut self, jump: &[u64; 3]) {
        self.jump_mul(&mp::pow_mod(&BASE, jump, &MOD));
    }

    /// Return the internal state of the generator as little-endian bytes
    /// that can be passed to [`SeedableRng::from_seed`] to reconstruct an
    /// identical generator.
    ///
    /// [`SeedableRng::from_seed`]: rand_core::SeedableRng::from_seed
    pub fn state(&self) -> [u8; 24] {
        crate::common::to_bytes([self.x, self.y, self.c])
    }

    // Create a new `Mwc192X` from random bytes. The value r used for the carry
    // is read as a fixed-point fraction r / 2⁶⁴ in [0..1) and multiplied by
    // MWC_A2 - 2, the size of the range of valid carries: the carry is
    // 1 + (r * (MWC_A2 - 2) >> 64).
    fn from_random_bytes(seed: [u8; 24]) -> Mwc192X {
        let [x, y, r] = utils::read_words(&seed);
        Mwc192X {
            x,
            y,
            c: 1 + ((r as u128 * (MWC_A2 - 2) as u128) >> 64) as u64,
        }
    }
}

impl SeedableRng for Mwc192X {
    type Seed = [u8; 24];

    /// Create a new `Mwc192X`.
    ///
    /// The seed is the little-endian representation of the state, that is,
    /// of the integer *s* = *x* + *y* · 2⁶⁴ + *c* · 2¹²⁸, where *x* and *y*
    /// are the digits and *c* is the carry. If *s* is not a valid state, that
    /// is, if *s* is zero or not smaller than the modulus
    /// *m* = 0xff4e24dd418f5433 · 2¹²⁸ − 1, the carry is set to one.
    #[inline]
    fn from_seed(seed: [u8; 24]) -> Mwc192X {
        // The seed is an explicit state: unlike in seed_from_u64() and
        // from_rng(), the carry is not mapped into the range of valid carries.
        let s = utils::read_words(&seed);
        let [x, y, c] = s;
        Mwc192X {
            x,
            y,
            c: if mp::is_valid(&s, &MOD) { c } else { 1 },
        }
    }

    /// Seed an `Mwc192X` from a `u64` as suggested by the reference
    /// implementation: the digits and the carry are initialized to
    /// valid values using the output of a SplitMix generator seeded with `seed`.
    fn seed_from_u64(seed: u64) -> Mwc192X {
        Mwc192X::from_random_bytes(crate::common::splitmix_bytes::<24>(seed))
    }

    /// Create a new `Mwc192X`, initializing the digits and the carry to valid
    /// values using the output of `rng`.
    fn from_rng<R: Rng + ?Sized>(rng: &mut R) -> Mwc192X {
        let mut seed = [0; 24];
        rng.fill_bytes(&mut seed);
        Mwc192X::from_random_bytes(seed)
    }

    /// Create a new `Mwc192X`, initializing the digits and the carry to valid
    /// values using the output of `rng`.
    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Mwc192X, R::Error> {
        let mut seed = [0; 24];
        rng.try_fill_bytes(&mut seed)?;
        Ok(Mwc192X::from_random_bytes(seed))
    }
}

impl TryRng for Mwc192X {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok((self.next_u64() >> 32) as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let result = self.y ^ self.c;
        let t = MWC_A2 as u128 * self.x as u128 + self.c as u128;
        self.x = self.y;
        self.y = t as u64;
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

    // x = 0x0123456789abcdef, y = 0xfedcba9876543210, c = 1
    const SEED: [u8; 24] = [
        0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
        0xfe, 1, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn reference() {
        let mut rng = Mwc192X::from_seed(SEED);
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC192X.c
        let expected = [
            0xfedcba9876543211,
            0xac8dcd6abd635774,
            0xaa0f361e7b30715d,
            0xe104b15c76fca2e4,
            0x2c12e1a7db8cbfa1,
            0x6105bf55a327d362,
            0x62ace12dcd4dc6ba,
            0x90e3539226e6de34,
            0x112e01165d99bb15,
            0x607f209a25bea6ba,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn reference_jumps() {
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC192X.c
        let jumped = |f: fn(&mut Mwc192X)| {
            let mut rng = Mwc192X::from_seed(SEED);
            f(&mut rng);
            rng
        };
        let expected = |x, y, c| Mwc192X { x, y, c };
        assert_eq!(
            jumped(Mwc192X::jump),
            expected(0x5438b996afe69147, 0x4d8061a4a1f2862f, 0xd600e9332b37e5a7)
        );
        assert_eq!(
            jumped(Mwc192X::long_jump),
            expected(0x9ff4a43e2bc6a812, 0xf695800ae27947cf, 0x008f2e60cfea1794)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(3, 70)),
            expected(0x746e2db78364cb25, 0x5b3243b447ce93d9, 0x6e64eea0ebc89f14)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(12345, 0)),
            expected(0x10ef1947237ccbb6, 0x4de156bc5cf48087, 0xe8c5b59ed41706bd)
        );
        assert_eq!(
            jumped(|rng| rng.jump_n(&[0x9e3779b97f4a7c15, 0x0123456789abcdef, 0])),
            expected(0xed3400ca05085951, 0x3968a6728db67b17, 0x82c26737259041f9)
        );
    }

    #[test]
    fn reference_seed_from_u64() {
        let mut rng = Mwc192X::seed_from_u64(42);
        assert_eq!(
            rng,
            Mwc192X {
                x: 0xbdd732262feb6e95,
                y: 0x28efe333b266f103,
                c: 0x4720da507373ae09
            }
        );
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC192X.c
        let expected = [
            0x6fcf3963c1155f0a,
            0xbc7618077e112a40,
            0xa650bfb788e891df,
            0x2cd35b11be644bb3,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc192X::from_seed(SEED);
            for _ in 0..d {
                a.next_u64();
            }
            let mut b = Mwc192X::from_seed(SEED);
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        // c * 2ᵉ with e > 0, small enough to step literally: 3 * 2⁸ = 768.
        let mut a = Mwc192X::from_seed(SEED);
        for _ in 0..3 * 256 {
            a.next_u64();
        }
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_ce(3, 8);
        assert_eq!(a, b, "jump_ce(3, 8)");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_matches_predefined_jumps() {
        let mut a = Mwc192X::from_seed(SEED);
        a.jump();
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_ce(1, 96);
        assert_eq!(a, b, "jump_ce(1, 96) == jump()");

        let mut a = Mwc192X::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_ce(1, 144);
        assert_eq!(a, b, "jump_ce(1, 144) == long_jump()");
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_n_matches_jump_ce() {
        // jump_n(&[d, 0, 0]) == jump_ce(d, 0) for a single-word distance.
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc192X::from_seed(SEED);
            a.jump_ce(d, 0);
            let mut b = Mwc192X::from_seed(SEED);
            b.jump_n(&[d, 0, 0]);
            assert_eq!(a, b, "jump_n(&[{d}, 0, 0])");
        }
        // Distances that need high words: 2⁹⁶ == jump(), 2¹⁴⁴ == long_jump().
        let mut a = Mwc192X::from_seed(SEED);
        a.jump();
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_n(&[0, 1 << 32, 0]);
        assert_eq!(a, b, "jump_n(2^96) == jump()");

        let mut a = Mwc192X::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_n(&[0, 0, 1 << 16]);
        assert_eq!(a, b, "jump_n(2^144) == long_jump()");

        // A distance jump_ce cannot express (odd part exceeds 64 bits):
        // 3 + 5 · 2⁶⁴, checked via A^(a + b) = A^a · A^b.
        let mut a = Mwc192X::from_seed(SEED);
        a.jump_ce(3, 0);
        a.jump_ce(5, 64);
        let mut b = Mwc192X::from_seed(SEED);
        b.jump_n(&[3, 5, 0]);
        assert_eq!(a, b, "jump_n(3 + 5 · 2^64)");
    }

    #[test]
    fn from_rng_maps_carry() {
        use crate::common::ConstRng;
        // Zero bytes give carry one; all-one bytes give the largest carry
        // produced by the mapping, a - 2.
        assert_eq!(
            Mwc192X::from_rng(&mut ConstRng(0)),
            Mwc192X { x: 0, y: 0, c: 1 }
        );
        let largest = Mwc192X {
            x: u64::MAX,
            y: u64::MAX,
            c: MWC_A2 - 2,
        };
        assert_eq!(Mwc192X::from_rng(&mut ConstRng(0xff)), largest);
        assert_eq!(Mwc192X::try_from_rng(&mut ConstRng(0xff)), Ok(largest));
    }

    #[test]
    fn invalid_seeds() {
        let from_words = |s: [u64; 3]| Mwc192X::from_seed(crate::common::to_bytes(s));
        // The zero state and the states not smaller than m get carry one.
        assert_eq!(from_words([0; 3]), Mwc192X { x: 0, y: 0, c: 1 });
        assert_eq!(
            from_words(MOD),
            Mwc192X {
                x: u64::MAX,
                y: u64::MAX,
                c: 1
            }
        );
        assert_eq!(from_words([5, 6, u64::MAX]), Mwc192X { x: 5, y: 6, c: 1 });
        // The other states, including those with zero or maximum carry, are
        // left unchanged.
        let below = [u64::MAX - 1, u64::MAX, MWC_A2 - 1];
        assert_eq!(
            from_words(below),
            Mwc192X {
                x: below[0],
                y: below[1],
                c: below[2]
            }
        );
        assert_eq!(from_words([0, 1, 0]), Mwc192X { x: 0, y: 1, c: 0 });
    }

    #[test]
    fn state_roundtrip() {
        let mut rng = Mwc192X::seed_from_u64(42);
        rng.next_u64();
        let clone = Mwc192X::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
