use crate::DartEmbedder;

use wasmtime::{Caller, Result};

/// Supported frequencies for timer ticks.
pub enum StopwatchFrequency {
    /// The stopwatch timer runs at 1kHz, meaning that one tick corresponds to
    /// 1 millisecond.
    KiloHertz,
    /// The stopwatch timer runs at 1MHz, meaning that one tick corresponds to
    /// 1 microsecond.
    MegaHertz,
}

pub fn func_monotonic_clock_frequency<E: DartEmbedder>() -> i32 {
    match E::MONOTONIC_FREQUENCY {
        StopwatchFrequency::KiloHertz => 1_000,
        StopwatchFrequency::MegaHertz => 1_000_000,
    }
}

pub fn func_monotonic_clock_ticks<E: DartEmbedder>(mut caller: Caller<'_, E>) -> Result<i64> {
    let embedder = caller.data_mut();
    embedder.monotonic_ticks()
}
