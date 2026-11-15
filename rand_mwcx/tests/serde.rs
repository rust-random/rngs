#![cfg(feature = "serde")]

use rand_core::{Rng, SeedableRng};
use rand_mwcx::{Mwc64X32, Mwc128X, Mwc128X32, Mwc192X};

macro_rules! serde_rng {
    ($rng:ident) => {
        let mut rng = $rng::seed_from_u64(0);

        let buf = postcard::to_allocvec(&rng).expect("Could not serialize");

        let mut deserialized: $rng = postcard::from_bytes(&buf).expect("Could not deserialize");

        for _ in 0..16 {
            assert_eq!(rng.next_u64(), deserialized.next_u64());
        }
    };
}

#[test]
fn test_mwc128x() {
    serde_rng!(Mwc128X);
}

#[test]
fn test_mwc192x() {
    serde_rng!(Mwc192X);
}

#[test]
fn test_mwc64x32() {
    serde_rng!(Mwc64X32);
}

#[test]
fn test_mwc128x32() {
    serde_rng!(Mwc128X32);
}
