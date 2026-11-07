// Copyright 2026 Developers of the Rand project.
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

const MWC_A1: u32 = 0xfff145f8;

/// The modulus m = `MWC_A1` · 2³² − 1, which fits in 64 bits, so the jump
/// functions need only 128-bit arithmetic.
const MOD: u64 = ((MWC_A1 as u64) << 32) - 1;

/// Return `a` · `b` mod m.
fn mul_mod(a: u64, b: u64) -> u64 {
    (a as u128 * b as u128 % MOD as u128) as u64
}

/// Return `MWC_A1ⁿ` mod m, where `MWC_A1` = 2⁻³² mod m is the multiplier
/// of the underlying generator.
fn pow_mod(mut n: u64) -> u64 {
    let mut r = 1;
    let mut a = MWC_A1 as u64;
    while n != 0 {
        if n & 1 != 0 {
            r = mul_mod(r, a);
        }
        a = mul_mod(a, a);
        n >>= 1;
    }
    r
}

/// An MWC64X/32 random number generator.
///
/// MWC64X/32 is a Marsaglia multiply-with-carry generator with 32-bit digits,
/// lag one, and period approximately 2⁶³, whose output is scrambled by X:
/// the result is the digit xored with the carry. It is David Thomas's MWC64X
/// generator with a different multiplier. It is not suitable for
/// cryptographic purposes. Its only 64-bit operations are a multiplication
/// and a sum, but, since the lag is one, each multiplication needs the result
/// of the previous step, so the processor cannot overlap consecutive steps (in
/// [`Mwc128X32`](crate::Mwc128X32), which has lag three, it can). It is meant
/// for environments in which registers are scarce, such as GPUs: otherwise,
/// use [`Mwc128X32`](crate::Mwc128X32), which has a larger period and is at
/// least as fast.
///
/// The underlying generator simulates a multiplicative congruential generator
/// with prime modulus *m* = 0xfff145f8 · 2³² − 1 and multiplier 2⁻³² mod *m*,
/// that is, the multiplicative inverse of 2³² modulo *m*, which is 0xfff145f8
/// (their product is *m* + 1). The special form of the modulus forces a defect
/// in the lattice structure, which the scrambler X hides.
///
/// The algorithm used here is translated from [the `MWC64X_32.c` reference
/// source code](https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC64X_32.c)
/// by Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mwc64X32 {
    x: u32,
    c: u32,
}

impl Mwc64X32 {
    /// Multiply the state, read as the integer *x* + *c* · 2³², by `r`
    /// modulo *m*. Since each step multiplies the state by
    /// *A* = 2⁻³² mod *m*, jumping ahead by *n* steps multiplies it by
    /// *Aⁿ* mod *m*.
    fn jump_mul(&mut self, r: u64) {
        let s = mul_mod((self.c as u64) << 32 | self.x as u64, r);
        self.x = s as u32;
        self.c = (s >> 32) as u32;
    }

    /// Jump forward, equivalently to 2³² calls to `next_u32()`.
    ///
    /// This can be used to generate 2³¹ non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_mwcx::rand_core::SeedableRng;
    /// use rand_mwcx::Mwc64X32;
    ///
    /// let rng1 = Mwc64X32::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        self.jump_mul(0x1012f7168d155184);
    }

    /// Jump forward, equivalently to 2⁴⁸ calls to `next_u32()`.
    ///
    /// This can be used to generate 2¹⁵ starting points, from each of which
    /// `jump()` will generate 2¹⁶ non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        self.jump_mul(0x7b046c7221442678);
    }

    /// Jump forward by *c* · 2*ᵉ* calls to `next_u32()`.
    ///
    /// For example, `jump_ce(1, 32)` is equivalent to [`jump`](Self::jump)
    /// and `jump_ce(1, 48)` is equivalent to [`long_jump`](Self::long_jump).
    /// Expressing the distance as *c* · 2*ᵉ* makes it possible to request both
    /// ordinary counts (`jump_ce(k, 0)`) and power-of-two jumps. For the jump
    /// to be meaningful, *c* · 2*ᵉ* should be smaller than the period
    /// (approximately 2⁶³).
    ///
    /// See [`jump_n`](Self::jump_n) to jump by an arbitrary distance.
    pub fn jump_ce(&mut self, c: u64, e: u32) {
        let mut r = pow_mod(c);
        for _ in 0..e {
            r = mul_mod(r, r);
        }
        self.jump_mul(r);
    }

    /// Jump forward by an arbitrary number of calls to `next_u32()`.
    ///
    /// This is equivalent to `n` calls to `next_u32()`. Unlike
    /// [`jump_ce`](Self::jump_ce), it can express any jump distance. For the
    /// jump to be meaningful, `n` should be smaller than the period
    /// (approximately 2⁶³).
    pub fn jump_n(&mut self, n: u64) {
        self.jump_mul(pow_mod(n));
    }

    /// Return the internal state of the generator as little-endian bytes
    /// that can be passed to [`SeedableRng::from_seed`] to reconstruct an
    /// identical generator.
    ///
    /// [`SeedableRng::from_seed`]: rand_core::SeedableRng::from_seed
    pub fn state(&self) -> [u8; 8] {
        ((self.c as u64) << 32 | self.x as u64).to_le_bytes()
    }

    // Create a new `Mwc64X32` from random bytes. The value r used for the carry
    // is read as a fixed-point fraction r / 2³² in [0..1) and multiplied by
    // MWC_A1 - 2, the size of the range of valid carries: the carry is
    // 1 + (r * (MWC_A1 - 2) >> 32).
    fn from_random_bytes(seed: [u8; 8]) -> Mwc64X32 {
        let [x, r] = utils::read_words(&seed);
        Mwc64X32 {
            x,
            c: 1 + ((r as u64 * (MWC_A1 - 2) as u64) >> 32) as u32,
        }
    }
}

impl SeedableRng for Mwc64X32 {
    type Seed = [u8; 8];

    /// Create a new `Mwc64X32`.
    ///
    /// The seed is the little-endian representation of the state, that is,
    /// of the integer *s* = *x* + *c* · 2³², where *x* is the digit and *c*
    /// is the carry. If *s* is not a valid state, that is, if *s* is zero or
    /// not smaller than the modulus *m* = 0xfff145f8 · 2³² − 1, the carry is
    /// set to one.
    #[inline]
    fn from_seed(seed: [u8; 8]) -> Mwc64X32 {
        // The seed is an explicit state: unlike in seed_from_u64() and
        // from_rng(), the carry is not mapped into the range of valid carries.
        let s = u64::from_le_bytes(seed);
        Mwc64X32 {
            x: s as u32,
            c: if s != 0 && s < MOD {
                (s >> 32) as u32
            } else {
                1
            },
        }
    }

    /// Seed an `Mwc64X32` from a `u64` as suggested by the reference
    /// implementation: the digit and the carry are initialized to
    /// valid values using the output of a SplitMix generator seeded with `seed`.
    fn seed_from_u64(seed: u64) -> Mwc64X32 {
        Mwc64X32::from_random_bytes(crate::common::splitmix_bytes::<8>(seed))
    }

    /// Create a new `Mwc64X32`, initializing the digit and the carry to valid
    /// values using the output of `rng`.
    fn from_rng<R: Rng + ?Sized>(rng: &mut R) -> Mwc64X32 {
        let mut seed = [0; 8];
        rng.fill_bytes(&mut seed);
        Mwc64X32::from_random_bytes(seed)
    }

    /// Create a new `Mwc64X32`, initializing the digit and the carry to valid
    /// values using the output of `rng`.
    fn try_from_rng<R: TryRng + ?Sized>(rng: &mut R) -> Result<Mwc64X32, R::Error> {
        let mut seed = [0; 8];
        rng.try_fill_bytes(&mut seed)?;
        Ok(Mwc64X32::from_random_bytes(seed))
    }
}

impl TryRng for Mwc64X32 {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let result = self.x ^ self.c;
        let t = MWC_A1 as u64 * self.x as u64 + self.c as u64;
        self.x = t as u32;
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

    // x = 0x89abcdef, c = 1
    const SEED: [u8; 8] = [0xef, 0xcd, 0xab, 0x89, 1, 0, 0, 0];

    #[test]
    fn reference() {
        let mut rng = Mwc64X32::from_seed(SEED);
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC64X_32.c
        let expected = [
            0x89abcdee, 0x124c08f7, 0x1dd9f62f, 0xe3dcdf5f, 0x6aadea99, 0x3b18a0d9, 0x8a555e1f,
            0x46d69f8e, 0x9edf4569, 0x85ccb634,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    fn reference_jumps() {
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC64X_32.c
        let jumped = |f: fn(&mut Mwc64X32)| {
            let mut rng = Mwc64X32::from_seed(SEED);
            f(&mut rng);
            rng
        };
        let expected = |x, c| Mwc64X32 { x, c };
        assert_eq!(jumped(Mwc64X32::jump), expected(0xf9132140, 0xd28b3488));
        assert_eq!(
            jumped(Mwc64X32::long_jump),
            expected(0x48cd3df2, 0x5b545e15)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(3, 70)),
            expected(0xf99803f0, 0x8856dbf7)
        );
        assert_eq!(
            jumped(|rng| rng.jump_ce(12345, 0)),
            expected(0x8304459c, 0xc3636383)
        );
        assert_eq!(
            jumped(|rng| rng.jump_n(0x9e3779b97f4a7c15)),
            expected(0x56b9861b, 0x0a7a33b3)
        );
    }

    #[test]
    fn reference_seed_from_u64() {
        let mut rng = Mwc64X32::seed_from_u64(42);
        assert_eq!(
            rng,
            Mwc64X32 {
                x: 0x2feb6e95,
                c: 0xbdcc466c
            }
        );
        // These values were produced with the reference implementation:
        // https://github.com/vigna/MWCPRNG/blob/main/prngs/MWC64X_32.c
        let expected = [0x922728f9, 0x0c1a2327, 0xd4d7b8a3, 0x528714b3];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn jump_ce_small_distances_match_stepping() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc64X32::from_seed(SEED);
            for _ in 0..d {
                a.next_u32();
            }
            let mut b = Mwc64X32::from_seed(SEED);
            b.jump_ce(d, 0);
            assert_eq!(a, b, "jump_ce({d}, 0)");
        }
        // c * 2ᵉ with e > 0, small enough to step literally: 3 * 2⁸ = 768.
        let mut a = Mwc64X32::from_seed(SEED);
        for _ in 0..3 * 256 {
            a.next_u32();
        }
        let mut b = Mwc64X32::from_seed(SEED);
        b.jump_ce(3, 8);
        assert_eq!(a, b, "jump_ce(3, 8)");
    }

    #[test]
    fn jump_ce_matches_predefined_jumps() {
        let mut a = Mwc64X32::from_seed(SEED);
        a.jump();
        let mut b = Mwc64X32::from_seed(SEED);
        b.jump_ce(1, 32);
        assert_eq!(a, b, "jump_ce(1, 32) == jump()");

        let mut a = Mwc64X32::from_seed(SEED);
        a.long_jump();
        let mut b = Mwc64X32::from_seed(SEED);
        b.jump_ce(1, 48);
        assert_eq!(a, b, "jump_ce(1, 48) == long_jump()");
    }

    #[test]
    fn jump_n_matches_jump_ce() {
        for &d in &[0, 1, 2, 3, 7, 64, 1000, 1_000_000] {
            let mut a = Mwc64X32::from_seed(SEED);
            a.jump_ce(d, 0);
            let mut b = Mwc64X32::from_seed(SEED);
            b.jump_n(d);
            assert_eq!(a, b, "jump_n({d})");
        }
        let mut a = Mwc64X32::from_seed(SEED);
        a.jump();
        let mut b = Mwc64X32::from_seed(SEED);
        b.jump_n(1 << 32);
        assert_eq!(a, b, "jump_n(2^32) == jump()");
    }

    #[test]
    fn from_rng_maps_carry() {
        use crate::common::ConstRng;
        // Zero bytes give carry one; all-one bytes give the largest carry
        // produced by the mapping, a - 2.
        assert_eq!(
            Mwc64X32::from_rng(&mut ConstRng(0)),
            Mwc64X32 { x: 0, c: 1 }
        );
        let largest = Mwc64X32 {
            x: u32::MAX,
            c: MWC_A1 - 2,
        };
        assert_eq!(Mwc64X32::from_rng(&mut ConstRng(0xff)), largest);
        assert_eq!(Mwc64X32::try_from_rng(&mut ConstRng(0xff)), Ok(largest));
    }

    #[test]
    fn invalid_seeds() {
        let from_word = |s: u64| Mwc64X32::from_seed(s.to_le_bytes());
        // The zero state and the states not smaller than m get carry one.
        assert_eq!(from_word(0), Mwc64X32 { x: 0, c: 1 });
        assert_eq!(from_word(MOD), Mwc64X32 { x: u32::MAX, c: 1 });
        assert_eq!(from_word(u64::MAX), Mwc64X32 { x: u32::MAX, c: 1 });
        // The other states, including those with zero or maximum carry, are
        // left unchanged.
        assert_eq!(
            from_word(MOD - 1),
            Mwc64X32 {
                x: u32::MAX - 1,
                c: MWC_A1 - 1
            }
        );
        assert_eq!(from_word(1), Mwc64X32 { x: 1, c: 0 });
    }

    #[test]
    fn state_roundtrip() {
        let mut rng = Mwc64X32::seed_from_u64(42);
        rng.next_u32();
        let clone = Mwc64X32::from_seed(rng.state());
        assert_eq!(clone, rng);
    }
}
