// Copyright 2025 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand_core::le::read_u32_into;
use rand_core::{RngCore, SeedableRng};
use rand_core::impls::{fill_bytes_via_next, next_u64_via_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
/// An sfc32 random number generator.
///
/// Good performance and statistical quality, but not cryptographically secure
/// and has a large difference between its worst-case and maximum period.
///
/// This implementation is derived ultimately from
/// [`the PractRand RNG test suite`](https://pracrand.sourceforge.net/) by
/// Chris Doty-Humphrey.
pub struct Sfc32 {
    a: u32,
    b: u32,
    c: u32,
    weyl: u32,
}

const BARREL_SHIFT: u32 = 21;
const RSHIFT: u32 = 9;
const LSHIFT: u32 = 3;
// WEYL_INC is always 1 in the PractRand implementation, but some other
// implementations use it to provide a stream facility. This is not yet
// implemented here, though.
const WEYL_INC: u32 = 1;

impl RngCore for Sfc32 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let old_b = self.b;
        let old_c = self.c;
        let old_weyl = self.weyl;

        let result = self.a.wrapping_add(old_b).wrapping_add(old_weyl);
        self.a = old_b ^ (old_b >> RSHIFT);
        self.b = old_c.wrapping_add(old_c << LSHIFT);
        self.c = result.wrapping_add(old_c.rotate_left(BARREL_SHIFT));
        self.weyl = old_weyl.wrapping_add(WEYL_INC);
        result
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        next_u64_via_u32(self)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }
}

// PracRand uses different mixing step counts for different types of seeds.
// Here, just use one of the larger values always.
const SEED_MIXING_STEPS: u32 = 15;

impl SeedableRng for Sfc32 {
    type Seed = [u8; 12];

    /// Create a new `Sfc32`.
    fn from_seed(seed: [u8; 12]) -> Sfc32 {
        let mut s = [0; 3];
        read_u32_into(&seed, &mut s);

        let mut rng = Sfc32 {
            a: s[0],
            b: s[1],
            c: s[2],
            weyl: WEYL_INC
        };

        for _ in 0..SEED_MIXING_STEPS {
            rng.next_u32();
        }

        rng
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_seed() {
        let reference_rng = Sfc32 {
            a: 0x09BC85E1,
            b: 0x5A96CB07,
            c: 0xB53C149C,
            weyl: 0x10
        };
        let test_rng = Sfc32::seed_from_u64(1);

        assert_eq!(test_rng, reference_rng)
    }

    #[test]
    fn reference() {
        // These values were produced with the reference implementation:
        // https://pracrand.sourceforge.net/
        let mut rng = Sfc32::from_seed([0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0]);
        let expected: [u32; 16] = [
            0x03B80BB8,
            0xA87DBC7E,
            0x1787178C,
            0x4C7B7234,
            0xC65DADE2,
            0x2C692349,
            0xF52C2153,
            0xDF098072,
            0x9D49B03C,
            0x9562381A,
            0xC9B41738,
            0x64B75E54,
            0x36CE9B32,
            0xF106947E,
            0x0AFC726B,
            0x549BBC87
        ];

        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }
}
