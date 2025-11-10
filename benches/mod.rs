// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(test)]
#![allow(non_snake_case)]

extern crate test;

use std::mem::size_of;
use test::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use criterion_cycles_per_byte::CyclesPerByte;
use rand_core::{RngCore, SeedableRng};

use rand_hc::Hc128Rng;
use rand_isaac::{Isaac64Rng, IsaacRng};
use rand_sfc::{Sfc32, Sfc64};
use rand_xorshift::XorShiftRng;
use rand_xoshiro::{
    SplitMix64, Xoroshiro64Star, Xoroshiro64StarStar, Xoroshiro128Plus, Xoroshiro128StarStar,
    Xoshiro128Plus, Xoshiro128PlusPlus, Xoshiro128StarStar, Xoshiro256Plus, Xoshiro256PlusPlus,
    Xoshiro256StarStar,
};

const RAND_BENCH_N: u64 = 1000;
const BYTES_LEN: usize = 1024;

fn gen_bytes(c: &mut Criterion<CyclesPerByte>) {
    let mut g = c.benchmark_group("gen_bytes");
    g.throughput(Throughput::Bytes(BYTES_LEN as u64 * RAND_BENCH_N));

    let mut master = Hc128Rng::seed_from_u64(0);

    macro_rules! gen_bytes {
        ($fnn:expr, $gen:expr) => {
            g.bench_function($fnn, |b| {
                let mut rng = $gen;
                let mut buf = [0u8; BYTES_LEN];
                b.iter(|| {
                    for _ in 0..RAND_BENCH_N {
                        rng.fill_bytes(&mut buf);
                        black_box(buf);
                    }
                })
            });
        };
    }

    gen_bytes!("xorshift", XorShiftRng::from_rng(&mut master));
    gen_bytes!(
        "xoshiro256starstar",
        Xoshiro256StarStar::from_rng(&mut master)
    );
    gen_bytes!("xoshiro256plus", Xoshiro256Plus::from_rng(&mut master));
    gen_bytes!(
        "xoshiro256plusplus",
        Xoshiro256PlusPlus::from_rng(&mut master)
    );
    gen_bytes!(
        "xoshiro128starstar",
        Xoshiro128StarStar::from_rng(&mut master)
    );
    gen_bytes!("xoshiro128plus", Xoshiro128Plus::from_rng(&mut master));
    gen_bytes!(
        "xoshiro128plusplus",
        Xoshiro128PlusPlus::from_rng(&mut master)
    );
    gen_bytes!(
        "xoroshiro128starstar",
        Xoroshiro128StarStar::from_rng(&mut master)
    );
    gen_bytes!("xoroshiro128plus", Xoroshiro128Plus::from_rng(&mut master));
    gen_bytes!(
        "xoroshiro64starstar",
        Xoroshiro64StarStar::from_rng(&mut master)
    );
    gen_bytes!("xoroshiro64star", Xoroshiro64Star::from_rng(&mut master));
    gen_bytes!("splitmix64", SplitMix64::from_rng(&mut master));
    gen_bytes!("hc128", Hc128Rng::from_rng(&mut master));
    gen_bytes!("isaac", IsaacRng::from_rng(&mut master));
    gen_bytes!("isaac64", Isaac64Rng::from_rng(&mut master));
    gen_bytes!("sfc32", Sfc32::from_rng(&mut master));
    gen_bytes!("sfc64", Sfc64::from_rng(&mut master));
}

// Save a dependency on Rand:
trait Generate {
    fn generate<R: RngCore>(rng: &mut R) -> Self;
}

impl Generate for u32 {
    #[inline]
    fn generate<R: RngCore>(rng: &mut R) -> Self {
        rng.next_u32()
    }
}

impl Generate for u64 {
    #[inline]
    fn generate<R: RngCore>(rng: &mut R) -> Self {
        rng.next_u64()
    }
}

fn gen_uint(c: &mut Criterion<CyclesPerByte>) {
    let mut master = Hc128Rng::seed_from_u64(1);

    macro_rules! gen_uint {
        ($g:expr, $fnn:expr, $ty:ty, $gen:expr) => {
            $g.bench_function($fnn, |b| {
                let mut rng = $gen;
                b.iter(|| {
                    let mut accum: $ty = 0;
                    for _ in 0..RAND_BENCH_N {
                        accum = accum.wrapping_add(<$ty as Generate>::generate(&mut rng));
                    }
                    accum
                });
            });
        };
    }

    {
        let mut g = c.benchmark_group("gen_u32");
        g.throughput(Throughput::Bytes(size_of::<u32>() as u64 * RAND_BENCH_N));

        gen_uint!(g, "xorshift", u32, XorShiftRng::from_rng(&mut master));
        gen_uint!(
            g,
            "xoshiro256starstar",
            u32,
            Xoshiro256StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro256plus",
            u32,
            Xoshiro256Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro256plusplus",
            u32,
            Xoshiro256PlusPlus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128starstar",
            u32,
            Xoshiro128StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128plus",
            u32,
            Xoshiro128Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128plusplus",
            u32,
            Xoshiro128PlusPlus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro128starstar",
            u32,
            Xoroshiro128StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro128plus",
            u32,
            Xoroshiro128Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro64starstar",
            u32,
            Xoroshiro64StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro64star",
            u32,
            Xoroshiro64Star::from_rng(&mut master)
        );
        gen_uint!(g, "splitmix64", u32, SplitMix64::from_rng(&mut master));
        gen_uint!(g, "hc128", u32, Hc128Rng::from_rng(&mut master));
        gen_uint!(g, "isaac", u32, IsaacRng::from_rng(&mut master));
        gen_uint!(g, "isaac64", u32, Isaac64Rng::from_rng(&mut master));
        gen_uint!(g, "sfc32", u32, Sfc32::from_rng(&mut master));
        gen_uint!(g, "sfc64", u32, Sfc64::from_rng(&mut master));
    }

    {
        let mut g = c.benchmark_group("gen_u64");
        g.throughput(Throughput::Bytes(size_of::<u64>() as u64 * RAND_BENCH_N));

        gen_uint!(g, "xorshift", u64, XorShiftRng::from_rng(&mut master));
        gen_uint!(
            g,
            "xoshiro256starstar",
            u64,
            Xoshiro256StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro256plus",
            u64,
            Xoshiro256Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro256plusplus",
            u64,
            Xoshiro256PlusPlus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128starstar",
            u64,
            Xoshiro128StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128plus",
            u64,
            Xoshiro128Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoshiro128plusplus",
            u64,
            Xoshiro128PlusPlus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro128starstar",
            u64,
            Xoroshiro128StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro128plus",
            u64,
            Xoroshiro128Plus::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro64starstar",
            u64,
            Xoroshiro64StarStar::from_rng(&mut master)
        );
        gen_uint!(
            g,
            "xoroshiro64star",
            u64,
            Xoroshiro64Star::from_rng(&mut master)
        );
        gen_uint!(g, "splitmix64", u64, SplitMix64::from_rng(&mut master));
        gen_uint!(g, "hc128", u64, Hc128Rng::from_rng(&mut master));
        gen_uint!(g, "isaac", u64, IsaacRng::from_rng(&mut master));
        gen_uint!(g, "isaac64", u64, Isaac64Rng::from_rng(&mut master));
        gen_uint!(g, "sfc32", u64, Sfc32::from_rng(&mut master));
        gen_uint!(g, "sfc64", u64, Sfc64::from_rng(&mut master));
    }
}

fn init(c: &mut Criterion) {
    let mut g = c.benchmark_group("init");
    let mut master = Hc128Rng::seed_from_u64(2);

    macro_rules! init_gen {
        ($fnn:expr, $gen:ident) => {
            g.bench_function($fnn, |b| {
                let mut rng = XorShiftRng::from_rng(&mut master);
                b.iter(|| $gen::from_rng(black_box(&mut rng)))
            });
        };
    }

    init_gen!("xorshift", XorShiftRng);
    init_gen!("xoshiro256starstar", Xoshiro256StarStar);
    init_gen!("xoshiro256plus", Xoshiro256Plus);
    init_gen!("xoshiro256plusplus", Xoshiro256PlusPlus);
    init_gen!("xoshiro128starstar", Xoshiro128StarStar);
    init_gen!("xoshiro128plus", Xoshiro128Plus);
    init_gen!("xoshiro128plusplus", Xoshiro128PlusPlus);
    init_gen!("xoroshiro128starstar", Xoroshiro128StarStar);
    init_gen!("xoroshiro128plus", Xoroshiro128Plus);
    init_gen!("xoroshiro64starstar", Xoroshiro64StarStar);
    init_gen!("xoroshiro64star", Xoroshiro64Star);
    init_gen!("splitmix64", SplitMix64);
    init_gen!("hc128", Hc128Rng);
    init_gen!("isaac", IsaacRng);
    init_gen!("isaac64", Isaac64Rng);
    init_gen!("sfc32", Sfc32);
    init_gen!("sfc64", Sfc64);
}

criterion_group! {
    name = gen_benches;
    config = Criterion::default().with_measurement(CyclesPerByte);
    targets =
    gen_uint,
    gen_bytes,
}

criterion_group! {
    name = standard_benches;
    config = Criterion::default();
    targets =
    init,
}

criterion_main!(gen_benches, standard_benches);
