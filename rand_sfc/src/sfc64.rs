// Copyright 2025 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rand_core::le::read_u64_into;
use rand_core::{RngCore, SeedableRng};
use rand_core::impls::fill_bytes_via_next;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
/// An sfc64 random number generator.
///
/// Good performance and statistical quality, but not cryptographically secure
/// and has a large difference between its worst-case and maximum period.
/// sfc64 has a longer period than sfc32 and should perform as well or better
/// on 64-bit processors, though Chris Doty-Humphrey believes its statistical
/// properties are similar to sfc32.
///
/// This implementation is derived ultimately from
/// [`the PractRand RNG test suite`](https://pracrand.sourceforge.net/) by
/// Chris Doty-Humphrey.
pub struct Sfc64 {
    a: u64,
    b: u64,
    c: u64,
    weyl: u64,
}

const BARREL_SHIFT: u32 = 24;
const RSHIFT: u32 = 11;
const LSHIFT: u32 = 3;
// WEYL_INC is always 1 in the PractRand implementation, but some other
// implementations use it to provide a stream facility. This is not yet
// implemented here, though.
const WEYL_INC: u64 = 1;

impl RngCore for Sfc64 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let old_b = self.b;
        let old_c = self.c;
        let old_weyl = self.weyl;

        let result = self.a.wrapping_add(old_b).wrapping_add(old_weyl);
        self.a = old_b ^ (old_b >> RSHIFT);
        self.b = old_c.wrapping_add(old_c << LSHIFT);
        self.c = result.wrapping_add(old_c.rotate_left(BARREL_SHIFT));
        self.weyl = self.weyl.wrapping_add(WEYL_INC);
        result
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }
}

// PracRand uses different mixing step counts for different types of seeds.
// Here, just use one of the larger values always.
const SEED_MIXING_STEPS: u32 = 18;

impl SeedableRng for Sfc64 {
    type Seed = [u8; 24];

    /// Create a new `Sfc64`.
    fn from_seed(seed: [u8; 24]) -> Sfc64 {
        let mut s = [0; 3];
        read_u64_into(&seed, &mut s);

        let mut rng = Sfc64 {
            a: s[0],
            b: s[1],
            c: s[2],
            weyl: WEYL_INC
        };

        for _ in 0..SEED_MIXING_STEPS {
            rng.next_u64();
        }
        rng
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_seed() {
        let reference_rng = Sfc64 {
            a: 0xFAFE87BC868CA702,
            b: 0x143212184DB2E2BB,
            c: 0xA4C7E95D7B898700,
            weyl: 0x13
        };
        let test_rng = Sfc64::seed_from_u64(1);

        assert_eq!(test_rng, reference_rng)
    }

    #[test]
    fn reference() {
        // These values were produced with the reference implementation:
        // https://pracrand.sourceforge.net/
        let mut rng = Sfc64::from_seed([
            1, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0,
        ]);

        let expected: [u64; 16] = [
            0xAD6FDC729FEEF3C1,
            0x2A20433D733F77D5,
            0x0310E21369647420,
            0x331A176BC71DCABC,
            0x53118F35C2494D94,
            0xA3A99DE7E77E16BF,
            0xA7B1B70A3E59A1FF,
            0x8E1127B28667EB3C,
            0x3FC589DC124CF6E8,
            0x81E0EAAACEB81D81,
            0x79F534652D262DF6,
            0x87F70C8214E186C5,
            0x67AF9C007B825917,
            0x5134AEC9998D8629,
            0x205AA24994068634,
            0x1C762918DBA3E139
        ];

        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }
}
