// Copyright 2025 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// A decent-quality 64 bit linear congruential generator used to extend seeds.
pub fn seed_extender_lcg (state: u64) -> u64 {
    let multiplier: u64 = 14647171131086947261;

    state.wrapping_mul(multiplier).wrapping_add(multiplier)
}