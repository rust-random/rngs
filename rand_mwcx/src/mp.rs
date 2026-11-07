// Copyright 2026 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Minimal modular arithmetic on multiple-precision natural numbers, used by
//! the jump functions.
//!
//! Numbers are little-endian arrays of `u64` words, and operands are assumed
//! to be smaller than the modulus. Multiplication is shift-and-add: it is
//! slow, but jumps are rare, and at these sizes a proper multiple-precision
//! library would not be much faster.

/// Return whether `a` ≥ `b`.
fn ge<const N: usize>(a: &[u64; N], b: &[u64; N]) -> bool {
    a.iter().rev().ge(b.iter().rev())
}

/// Return bit `i` of the little-endian integer held in `x`.
fn bit(x: &[u64], i: usize) -> bool {
    (x[i / 64] >> (i % 64)) & 1 != 0
}

/// Return the number of bits of the little-endian integer held in `x` (zero
/// for zero).
fn bit_len(x: &[u64]) -> usize {
    x.iter()
        .rposition(|&w| w != 0)
        .map_or(0, |i| 64 * (i + 1) - x[i].leading_zeros() as usize)
}

/// Set `a` to (`a` + `b`) mod `m`, assuming `a < m` and `b < m`.
fn add_mod<const N: usize>(a: &mut [u64; N], b: &[u64; N], m: &[u64; N]) {
    debug_assert!(!ge(a, m) && !ge(b, m));
    let mut carry = false;
    for (a, &b) in a.iter_mut().zip(b) {
        let (s, c0) = a.overflowing_add(b);
        let (s, c1) = s.overflowing_add(carry as u64);
        *a = s;
        carry = c0 | c1;
    }
    // The sum is smaller than 2m, so one subtraction suffices; if the sum
    // overflowed, the subtraction wraps around to the correct result.
    if carry || ge(a, m) {
        let mut borrow = false;
        for (a, &m) in a.iter_mut().zip(m) {
            let (d, b0) = a.overflowing_sub(m);
            let (d, b1) = d.overflowing_sub(borrow as u64);
            *a = d;
            borrow = b0 | b1;
        }
    }
}

/// Return whether 0 < `s` < `m`, that is, whether `s` is a valid state of an
/// MWC generator with modulus `m`.
pub(crate) fn is_valid<const N: usize>(s: &[u64; N], m: &[u64; N]) -> bool {
    s.iter().any(|&w| w != 0) && !ge(s, m)
}

/// Return `a` · `b` mod `m`, assuming `a < m`.
pub(crate) fn mul_mod<const N: usize>(a: &[u64; N], b: &[u64; N], m: &[u64; N]) -> [u64; N] {
    debug_assert!(!ge(a, m));
    let mut r = [0; N];
    let mut t = *a;
    for i in 0..bit_len(b) {
        if bit(b, i) {
            add_mod(&mut r, &t, m);
        }
        let u = t;
        add_mod(&mut t, &u, m);
    }
    r
}

/// Return `baseⁿ` mod `m`, where *n* is the little-endian integer held in
/// `exp` and `base < m`.
pub(crate) fn pow_mod<const N: usize>(base: &[u64; N], exp: &[u64], m: &[u64; N]) -> [u64; N] {
    debug_assert!(!ge(base, m));
    let mut r = [0; N];
    r[0] = 1;
    let mut b = *base;
    // Right-to-left square-and-multiply
    let bits = bit_len(exp);
    for i in 0..bits {
        if bit(exp, i) {
            r = mul_mod(&r, &b, m);
        }
        if i + 1 < bits {
            b = mul_mod(&b, &b, m);
        }
    }
    r
}

/// Return `base`^(`c` · 2^`e`) mod `m` where `base < m`.
pub(crate) fn pow_mod_ce<const N: usize>(
    base: &[u64; N],
    c: u64,
    e: u32,
    m: &[u64; N],
) -> [u64; N] {
    debug_assert!(!ge(base, m));
    let mut r = pow_mod(base, &[c], m);
    for _ in 0..e {
        r = mul_mod(&r, &r, m);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    // The Mersenne prime 2¹²⁷ − 1, which spans two words.
    const M: [u64; 2] = [u64::MAX, u64::MAX >> 1];

    #[test]
    fn small_values_match_u128() {
        let m = (M[1] as u128) << 64 | M[0] as u128;
        let words = |x: u128| [x as u64, (x >> 64) as u64];
        for &(a, b) in &[
            (0u128, 5u128),
            (1, m - 1),
            (m - 1, m - 1),
            (
                0x0123456789abcdef_fedcba9876543210,
                0x0f1e2d3c4b5a6978_8796a5b4c3d2e1f0,
            ),
        ] {
            // (a · b) mod m computed by Russian-peasant multiplication on u128,
            // which cannot overflow as m < 2¹²⁷.
            let (mut r, mut t, mut k) = (0u128, a % m, b);
            while k != 0 {
                if k & 1 != 0 {
                    r = (r + t) % m;
                }
                t = (t + t) % m;
                k >>= 1;
            }
            assert_eq!(mul_mod(&words(a), &words(b), &M), words(r));
        }
    }

    #[test]
    fn add_mod_overflow() {
        // m − 1 + m − 1 overflows 2¹²⁸ when m > 2¹²⁷.
        let m = [u64::MAX - 58, u64::MAX]; // 2¹²⁸ − 59
        let mut a = [u64::MAX - 59, u64::MAX];
        add_mod(&mut a, &[u64::MAX - 59, u64::MAX], &m);
        assert_eq!(a, [u64::MAX - 60, u64::MAX]);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn pow_mod_fermat() {
        // a^(m − 1) = 1 mod m for the prime m = 2¹²⁷ − 1.
        let exp = [M[0] - 1, M[1]];
        assert_eq!(pow_mod(&[3, 0], &exp, &M), [1, 0]);
        // 2¹²⁷ = 1 and 2¹²⁸ = 2 mod 2¹²⁷ − 1.
        assert_eq!(pow_mod_ce(&[2, 0], 127, 0, &M), [1, 0]);
        assert_eq!(pow_mod_ce(&[2, 0], 1, 7, &M), [2, 0]);
        assert_eq!(pow_mod(&[2, 0], &[0], &M), [1, 0]);
    }
}
