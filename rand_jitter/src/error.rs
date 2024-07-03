// Copyright 2018 Developers of the Rand project.
// Copyright 2013-2015 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::fmt;

/// Base code for all `JitterRng` errors
const ERROR_BASE: u32 = 0xAE53_0400;

/// An error that can occur when [`JitterRng::test_timer`] fails.
///
/// All variants have a value of 0xAE530400 = 2924676096 plus a small
/// increment (1 through 5).
///
/// [`JitterRng::test_timer`]: crate::JitterRng::test_timer
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(clippy::manual_non_exhaustive)]
//^ TODO: Replace with `#[non_exhaustive]` for Rust >= 1.40
pub enum TimerError {
    /// No timer available.
    NoTimer = ERROR_BASE + 1,
    /// Timer too coarse to use as an entropy source.
    CoarseTimer = ERROR_BASE + 2,
    /// Timer is not monotonically increasing.
    NotMonotonic = ERROR_BASE + 3,
    /// Variations of deltas of time too small.
    TinyVariations = ERROR_BASE + 4,
    /// Too many stuck results (indicating no added entropy).
    TooManyStuck = ERROR_BASE + 5,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl TimerError {
    fn description(&self) -> &'static str {
        match *self {
            TimerError::NoTimer => "no timer available",
            TimerError::CoarseTimer => "coarse timer",
            TimerError::NotMonotonic => "timer not monotonic",
            TimerError::TinyVariations => "time delta variations too small",
            TimerError::TooManyStuck => "too many stuck results",
            TimerError::__Nonexhaustive => unreachable!(),
        }
    }
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for TimerError {
    fn description(&self) -> &str {
        self.description()
    }
}
