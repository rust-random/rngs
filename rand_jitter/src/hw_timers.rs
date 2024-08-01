/// returns a 64 Bit timestamp with an optimized implementation
/// on x86 platforms, based on the rdtsc instruction
///
/// May use it instead of get_nstime, if your platform supports it.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn get_nstime_rdtsc() -> u64 {
    use core::arch::asm;

    let eax: u32;
    let edx: u32;

    // rdtsc should be preferred against rdtscp, as it
    // typically introduces a higher amount of clock jitter
    // due to missing dependencies on previous instructions
    // and loads becoming visible
    unsafe {
        asm!(
          "rdtsc",
          lateout("eax") eax,
          lateout("edx") edx,
          options(nomem, nostack)
        )
    }

    u64::from(edx) << 32 | u64::from(eax)
}
