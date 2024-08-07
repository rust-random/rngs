use rand_jitter::JitterRng;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
#[test]
fn test_jitter_init() {
    use rand_core::RngCore;

    // Because this is a debug build, measurements here are not representative
    // of the final release build.
    // Don't fail this test if initializing `JitterRng` fails because of a
    // bad timer (the timer from the standard library may not have enough
    // accuracy on all platforms).
    match JitterRng::new() {
        Ok(ref mut rng) => {
            // false positives are possible, but extremely unlikely
            assert!(rng.next_u32() | rng.next_u32() != 0);
        }
        Err(_) => {}
    }
}

#[test]
fn test_jitter_bad_timer() {
    fn bad_timer() -> u64 {
        0
    }
    let mut rng = JitterRng::new_with_timer(bad_timer);
    assert!(rng.test_timer().is_err());
}

#[test]
fn test_jitter_closure() {
    fn bad_timer() -> u64 {
        0
    }
    let at_start = bad_timer();
    let _ = JitterRng::new_with_timer(move || bad_timer() - at_start);
}
