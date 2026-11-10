//! Universal time and tick provider for Kōgnitara.
//! Designed to run on bare-metal kernels (such as Amentys OS in `no_std`)
//! as well as standard operating system environments.

#[cfg(not(feature = "std"))]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "std"))]
static GLOBAL_TICK_COUNTER: AtomicU64 = AtomicU64::new(1_000_000);

/// Returns current monotonic epoch timestamp or logical tick counter.
pub fn current_timestamp() -> i64 {
    #[cfg(feature = "std")]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(1_700_000_000)
    }

    #[cfg(not(feature = "std"))]
    {
        GLOBAL_TICK_COUNTER.fetch_add(1, Ordering::Relaxed) as i64
    }
}

/// Allows the Amentys OS kernel or custom runtime to advance or set the logical tick.
#[cfg(not(feature = "std"))]
pub fn advance_kernel_ticks(ticks: u64) {
    GLOBAL_TICK_COUNTER.fetch_add(ticks, Ordering::Relaxed);
}
