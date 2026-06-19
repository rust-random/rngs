// Copyright 2018-2026 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Minimal arithmetic in F₂[*x*] / (charpoly), the ring in which advancing a
//! linear generator by one step is multiplication by *x*. Used to compute the
//! jump polynomial for the `jump_ce` and `jump_n` methods.
//!
//! `charpoly` holds the `deg` low coefficients of the characteristic polynomial
//! (`deg = 64 * charpoly.len()`); the leading `x^deg` term is implicit. All
//! supported generators have a state that is a whole number of `u64` words, so
//! `deg` is a multiple of 64.

/// Coefficient `k` of the polynomial stored in `a`.
#[inline]
fn bit(a: &[u64], k: usize) -> u64 {
    (a[k / 64] >> (k & 63)) & 1
}

/// `a <- a · x mod charpoly`, in place. Requires `a.len() == charpoly.len()`.
pub(crate) fn mul_x(a: &mut [u64], charpoly: &[u64]) {
    let mut carry = 0;
    for word in a.iter_mut() {
        let next_carry = *word >> 63;
        *word = (*word << 1) | carry;
        carry = next_carry;
    }
    // All supported degrees are multiples of 64, so the x^deg coefficient is
    // exactly the carry out of the top word.
    if carry != 0 {
        for (w, &p) in a.iter_mut().zip(charpoly) {
            *w ^= p;
        }
    }
}

/// `a <- a · b mod charpoly`.
fn mul_mod(a: &mut [u64], b: &[u64], charpoly: &[u64]) {
    let w = charpoly.len();
    let deg = 64 * w;
    let mut r = [0; 8];
    for k in (0..deg).rev() {
        mul_x(&mut r[..w], charpoly);
        if bit(a, k) != 0 {
            for i in 0..w {
                r[i] ^= b[i];
            }
        }
    }
    a[..w].copy_from_slice(&r[..w]);
}

/// `out <- x^(c · 2^e) mod charpoly`, by square-and-multiply. Requires
/// `out.len() == charpoly.len()`.
pub(crate) fn jump_poly_ce(c: u64, e: u32, charpoly: &[u64], out: &mut [u64]) {
    let w = charpoly.len();
    for o in out.iter_mut() {
        *o = 0;
    }
    out[0] = 1; // out = 1
    for k in (0..64).rev() {
        // out = out^2
        let mut sq = [0; 8];
        sq[..w].copy_from_slice(&out[..w]);
        mul_mod(out, &sq[..w], charpoly);
        if (c >> k) & 1 != 0 {
            mul_x(out, charpoly);
        }
    }
    // out = (x^c)^(2^e) = x^(c · 2^e)
    for _ in 0..e {
        let mut sq = [0; 8];
        sq[..w].copy_from_slice(&out[..w]);
        mul_mod(out, &sq[..w], charpoly);
    }
}

/// `out <- x^n mod charpoly`, where `n = jump[0] + jump[1] · 2^64 + …` is the
/// little-endian integer held in the `N` words of `jump`. Square-and-multiply
/// over the bits of `n`, from the most significant down. Requires
/// `out.len() == charpoly.len()`.
pub(crate) fn jump_poly_n<const N: usize>(jump: &[u64; N], charpoly: &[u64], out: &mut [u64]) {
    let w = charpoly.len();
    for o in out.iter_mut() {
        *o = 0;
    }
    out[0] = 1; // out = 1
    for k in (0..N * 64).rev() {
        // out = out^2
        let mut sq = [0; 8];
        sq[..w].copy_from_slice(&out[..w]);
        mul_mod(out, &sq[..w], charpoly);
        if (jump[k >> 6] >> (k & 63)) & 1 != 0 {
            mul_x(out, charpoly);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{jump_poly_ce, jump_poly_n, mul_x};

    // xoshiro256 characteristic polynomial.
    const P: [u64; 4] = [
        0x9d116f2bb0f0f001,
        0x0280002bcefd1a5e,
        0x04b4edcf26259f85,
        0x0003c03c3f3ecb19,
    ];

    // x^c mod p by applying mul_x c times to the constant 1.
    fn x_pow(c: u64) -> [u64; 4] {
        let mut a = [1, 0, 0, 0];
        for _ in 0..c {
            mul_x(&mut a, &P);
        }
        a
    }

    #[test]
    fn jump_poly_ce_matches_repeated_mul_x_for_e0() {
        for &c in &[0, 1, 2, 3, 7, 64, 1000, 5000] {
            let mut out = [0; 4];
            jump_poly_ce(c, 0, &P, &mut out);
            assert_eq!(out, x_pow(c), "jump_poly_ce({c}, 0)");
        }
    }

    #[test]
    fn jump_poly_ce_matches_repeated_mul_x_for_powers_of_two() {
        // jump_poly_ce(1, e) == x^(2^e); compare to 2^e applications of mul_x.
        for e in 0..=12 {
            let mut out = [0; 4];
            jump_poly_ce(1, e, &P, &mut out);
            assert_eq!(out, x_pow(1 << e), "jump_poly_ce(1, {e})");
        }
    }

    #[test]
    fn x_pow_zero_is_one() {
        let mut out = [0; 4];
        jump_poly_ce(0, 0, &P, &mut out);
        assert_eq!(out, [1, 0, 0, 0]);
    }

    #[test]
    fn jump_poly_ce_consistency_under_current_build() {
        // x^300 * x^45 == x^345.
        let mut g = [0; 4];
        jump_poly_ce(300, 0, &P, &mut g);
        for _ in 0..45 {
            mul_x(&mut g, &P);
        }
        let mut expect = [0; 4];
        jump_poly_ce(345, 0, &P, &mut expect);
        assert_eq!(g, expect);
    }

    #[test]
    fn jump_poly_ce_c_and_e_combined() {
        // jump_poly_ce(c, e) == x^(c * 2^e); exercises the combined exponent path.
        for &(c, e) in &[(3, 8), (5, 4), (7, 10)] {
            let mut out = [0; 4];
            jump_poly_ce(c, e, &P, &mut out);
            assert_eq!(out, x_pow(c << e), "jump_poly_ce({c}, {e})");
        }
    }

    #[test]
    fn jump_poly_n_matches_ce_single_word() {
        // jump_poly_n(&[n]) == jump_poly_ce(n, 0) for a single-word distance.
        for &n in &[0, 1, 2, 3, 7, 64, 1000, 5000] {
            let mut got = [0; 4];
            jump_poly_n(&[n], &P, &mut got);
            let mut expect = [0; 4];
            jump_poly_ce(n, 0, &P, &mut expect);
            assert_eq!(got, expect, "jump_poly_n(&[{n}])");
        }
    }

    #[test]
    fn jump_poly_n_matches_ce_multi_word() {
        // n = c * 2^64 is the two-word integer [0, c]; compare to ce(c, 64).
        for &c in &[1, 3, 5000] {
            let mut got = [0; 4];
            jump_poly_n(&[0, c], &P, &mut got);
            let mut expect = [0; 4];
            jump_poly_ce(c, 64, &P, &mut expect);
            assert_eq!(got, expect, "jump_poly_n(&[0, {c}])");
        }
    }
}
