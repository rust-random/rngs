# rand_jitter

[![Test Status](https://github.com/rust-random/rngs/workflows/Tests/badge.svg?event=push)](https://github.com/rust-random/rngs/actions)
[![Latest version](https://img.shields.io/crates/v/rand_jitter.svg)](https://crates.io/crates/rand_jitter)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://img.shields.io/badge/api-master-yellow.svg)](https://rust-random.github.io/rand/rand_jitter)
[![API](https://docs.rs/rand_jitter/badge.svg)](https://docs.rs/rand_jitter)

Non-physical true random number generator based on timing jitter.

Note that this RNG is not suited for use cases where cryptographic security is
required (also see [this
discussion](https://github.com/rust-random/rand/issues/699)).

This crate depends on [rand_core](https://crates.io/crates/rand_core) and is
part of the [Rand project](https://github.com/rust-random/rand).

This crate aims to support all of Rust's `std` platforms with a system-provided
entropy source. Unlike other Rand crates, this crate does not support `no_std`
(handling this gracefully is a current discussion topic).

Links:

-   [API documentation (master)](https://rust-random.github.io/rand/rand_jitter)
-   [API documentation (docs.rs)](https://docs.rs/rand_jitter)
-   [Changelog](https://github.com/rust-random/rngs/blob/master/rand_jitter/CHANGELOG.md)

## Features

This crate has optional `std` support which is *disabled by default*;
this feature is required to provide the `JitterRng::new` function;
without `std` support a timer must be supplied via `JitterRng::new_with_timer`.

## Quality testing

`JitterRng::new()` has build-in, but limited, quality testing, however
before using `JitterRng` on untested hardware, or after changes that could
effect how the code is optimized (such as a new LLVM version), it is
recommend to run the much more stringent
[NIST SP 800-90B Entropy Estimation Suite](https://github.com/usnistgov/SP800-90B_EntropyAssessment).

Use the following code using `timer_stats` to collect the data:

```rust,no_run
use rand_jitter::JitterRng;

use std::error::Error;
use std::fs::File;
use std::io::Write;

fn get_nstime() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    // The correct way to calculate the current time is
    // `dur.as_secs() * 1_000_000_000 + dur.subsec_nanos() as u64`
    // But this is faster, and the difference in terms of entropy is
    // negligible (log2(10^9) == 29.9).
    dur.as_secs() << 30 | dur.subsec_nanos() as u64
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = JitterRng::new_with_timer(get_nstime);

    // 1_000_000 results are required for the
    // NIST SP 800-90B Entropy Estimation Suite
    const ROUNDS: usize = 1_000_000;
    let mut deltas_variable: Vec<u8> = Vec::with_capacity(ROUNDS);
    let mut deltas_minimal: Vec<u8> = Vec::with_capacity(ROUNDS);

    for _ in 0..ROUNDS {
        deltas_variable.push(rng.timer_stats(true) as u8);
        deltas_minimal.push(rng.timer_stats(false) as u8);
    }

    // Write out after the statistics collection loop, to not disturb the
    // test results.
    File::create("jitter_rng_var.bin")?.write(&deltas_variable)?;
    File::create("jitter_rng_min.bin")?.write(&deltas_minimal)?;
    Ok(())
}
```

This will produce two files: `jitter_rng_var.bin` and `jitter_rng_min.bin`.
Run the Entropy Estimation Suite in three configurations, as outlined below.
Every run has two steps. One step to produce an estimation, another to
validate the estimation.

1. Estimate the expected amount of entropy that is at least available with
   each round of the entropy collector. This number should be greater than
   the amount estimated with `64 / test_timer()`.
   ```sh
   python noniid_main.py -v jitter_rng_var.bin 8
   restart.py -v jitter_rng_var.bin 8 <min-entropy>
   ```
2. Estimate the expected amount of entropy that is available in the last 4
   bits of the timer delta after running noice sources. Note that a value of
   `3.70` is the minimum estimated entropy for true randomness.
   ```sh
   python noniid_main.py -v -u 4 jitter_rng_var.bin 4
   restart.py -v -u 4 jitter_rng_var.bin 4 <min-entropy>
   ```
3. Estimate the expected amount of entropy that is available to the entropy
   collector if both noise sources only run their minimal number of times.
   This measures the absolute worst-case, and gives a lower bound for the
   available entropy.
   ```sh
   python noniid_main.py -v -u 4 jitter_rng_min.bin 4
   restart.py -v -u 4 jitter_rng_min.bin 4 <min-entropy>
   ```

## License

`rand_jitter` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
