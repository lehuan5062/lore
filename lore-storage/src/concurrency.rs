// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic;

use lore_error_set::prelude::*;
use tokio::sync::Semaphore;
use tokio::sync::SemaphorePermit;

use crate::compress::FRAGMENT_SIZE_THRESHOLD;

/// Minimum fragment size for chunking (32 KiB).
pub const FRAGMENT_SIZE_MINIMUM: usize = 32 * 1024;

pub use lore_base::types::FRAGMENT_SIZE_EXPECTED;

/// Default file count concurrency limit when not configured.
pub const FILE_COUNT_LIMIT_DEFAULT: usize = 10000;

// Fragment concurrency is budgeted in KiB units. The total budget allows ~4000
// maximum-size (256 KiB) fragments in flight simultaneously (1 GiB total).
// Each fragment acquires max(ceil(content_size / 1024), FRAGMENT_MINIMUM_COST_KIB)
// permits, so small fragments are capped at FRAGMENT_MINIMUM_COST_KIB to also
// bound the fragment count (~265k for tiny fragments).
pub const FRAGMENT_BUDGET_KIB: usize = 1024 * 1024; // 1 GiB
pub const FRAGMENT_MINIMUM_COST_KIB: u32 = 4;
const FRAGMENT_MAXIMUM_COST_KIB: u32 = FRAGMENT_SIZE_THRESHOLD as u32;

static FILE_COUNT_LIMITER: OnceLock<Semaphore> = OnceLock::<Semaphore>::new();
static FRAGMENT_LIMITER: OnceLock<Arc<Semaphore>> = OnceLock::new();
static COMPRESS_LIMITER: OnceLock<Option<Arc<Semaphore>>> = OnceLock::new();

/// When true, load operations enforce repository isolation.
pub static LOCAL_ISOLATION: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// Configured file count limit. Set via [`configure`] before first use.
static FILE_COUNT_LIMIT_CONFIG: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

/// Configured compress task limit. Set via [`configure_compress_limiter`] before first use.
static COMPRESS_LIMIT_CONFIG: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

/// Configure the file count concurrency limit.
///
/// Must be called before the first call to [`file_count_limiter`] or
/// [`file_count_limit_acquire`]; later calls have no effect because the
/// semaphore is initialised on first use.
pub fn configure(file_count_limit: usize) {
    FILE_COUNT_LIMIT_CONFIG.store(file_count_limit, atomic::Ordering::Relaxed);
}

/// Configure the compress task concurrency limit.
///
/// A limit of 0 (default) disables the limiter. Must be called before the
/// first call to [`compress_limit_acquire`]; later calls have no effect.
pub fn configure_compress_limiter(limit: usize) {
    COMPRESS_LIMIT_CONFIG.store(limit, atomic::Ordering::Relaxed);
}

/// Return the global compress limiter, creating it on first access.
/// Returns `None` if no limit was configured (limit == 0).
fn compress_limiter() -> &'static Option<Arc<Semaphore>> {
    COMPRESS_LIMITER.get_or_init(|| {
        let limit = COMPRESS_LIMIT_CONFIG.load(atomic::Ordering::Relaxed);
        if limit > 0 {
            lore_base::lore_debug!("Compress task limit set to {limit}");
            Some(Arc::new(Semaphore::new(limit)))
        } else {
            None
        }
    })
}

/// Acquire a permit from the compress limiter if one is configured.
/// Returns `None` if no compress limit is active.
pub async fn compress_limit_acquire() -> Option<SemaphorePermit<'static>> {
    if let Some(semaphore) = compress_limiter().as_deref() {
        semaphore.acquire().await.ok()
    } else {
        None
    }
}

/// Return the global file-count semaphore, creating it on first access.
pub fn file_count_limiter() -> &'static Semaphore {
    FILE_COUNT_LIMITER.get_or_init(|| {
        Semaphore::new({
            let mut limit = FILE_COUNT_LIMIT_CONFIG.load(atomic::Ordering::Relaxed);
            if limit == 0 {
                limit = FILE_COUNT_LIMIT_DEFAULT;
            }
            lore_base::lore_debug!("File parallel count limit set to {limit}");
            limit
        })
    })
}

/// Acquire a permit from the file-count limiter.
pub async fn file_count_limit_acquire() -> Result<SemaphorePermit<'static>, SemaphoreError> {
    file_count_limiter()
        .acquire()
        .await
        .internal("Failed to acquire file limit permit")
        .map_err(SemaphoreError::from)
}

/// Return the global fragment-budget semaphore, creating it on first access.
pub fn fragment_limiter() -> &'static Semaphore {
    fragment_limiter_arc()
}

/// Return a cloneable owning handle to the global fragment-budget semaphore.
///
/// Permits acquired from this handle via [`Semaphore::acquire_many_owned`] share
/// the same budget as permits acquired from [`fragment_limiter`]; they can be
/// moved into spawned tasks and released independently.
pub fn fragment_limiter_owned() -> Arc<Semaphore> {
    Arc::clone(fragment_limiter_arc())
}

fn fragment_limiter_arc() -> &'static Arc<Semaphore> {
    FRAGMENT_LIMITER.get_or_init(|| Arc::new(Semaphore::new(FRAGMENT_BUDGET_KIB)))
}

/// Acquire an owned memory permit sized for a fragment buffer of `buffer_len`
/// bytes. The permit can be moved into a spawned task and is released when
/// dropped. Returns `None` if the fragment limiter has been closed.
pub async fn acquire_fragment_memory_permit(
    buffer_len: usize,
) -> Option<tokio::sync::OwnedSemaphorePermit> {
    fragment_limiter_owned()
        .acquire_many_owned(fragment_permit_count(buffer_len))
        .await
        .ok()
}

/// Compute the number of semaphore permits a fragment of `content_size` bytes
/// should acquire from the fragment limiter.
pub fn fragment_permit_count(content_size: usize) -> u32 {
    // Clamp before casting to u32 to avoid overflow for content sizes >= 4 TiB
    (content_size
        .div_ceil(1024)
        .min(FRAGMENT_MAXIMUM_COST_KIB as usize) as u32)
        .max(FRAGMENT_MINIMUM_COST_KIB)
}

#[error_set]
pub enum SemaphoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_permit_count_minimum() {
        // Very small content should be clamped to FRAGMENT_MINIMUM_COST_KIB
        assert_eq!(fragment_permit_count(0), FRAGMENT_MINIMUM_COST_KIB);
        assert_eq!(fragment_permit_count(1), FRAGMENT_MINIMUM_COST_KIB);
        assert_eq!(fragment_permit_count(1024), FRAGMENT_MINIMUM_COST_KIB);
    }

    #[test]
    fn fragment_permit_count_maximum() {
        // Very large content should be clamped to FRAGMENT_MAXIMUM_COST_KIB
        let huge = 1024 * 1024 * 1024; // 1 GiB
        assert_eq!(fragment_permit_count(huge), FRAGMENT_MAXIMUM_COST_KIB);
    }

    #[test]
    fn fragment_permit_count_mid_range() {
        // 100 KiB content -> ceil(100*1024/1024) = 100 permits
        let size = 100 * 1024;
        assert_eq!(fragment_permit_count(size), 100);
    }

    #[tokio::test]
    async fn acquire_fragment_memory_permit_sizes_by_buffer() {
        // Inspect the permit's own `num_permits()` so the test does not sample
        // the global semaphore's available_permits (which other concurrent
        // tests perturb).
        let permit_small = acquire_fragment_memory_permit(1).await.expect("small");
        assert_eq!(
            permit_small.num_permits(),
            FRAGMENT_MINIMUM_COST_KIB as usize,
            "1-byte buffer should cost FRAGMENT_MINIMUM_COST_KIB permits"
        );
        drop(permit_small);

        let permit_mid = acquire_fragment_memory_permit(100 * 1024)
            .await
            .expect("mid");
        assert_eq!(
            permit_mid.num_permits(),
            100,
            "100 KiB buffer should cost 100 permits"
        );
        drop(permit_mid);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fragment_memory_permit_saturation_does_not_deadlock() {
        // Use a dedicated Arc<Semaphore> for this stress test so we don't
        // perturb the global fragment_limiter (other tests sample it). The
        // permit-sizing logic is the same function (fragment_permit_count),
        // the only difference is which semaphore we acquire against.
        let semaphore = Arc::new(Semaphore::new(16 * FRAGMENT_MINIMUM_COST_KIB as usize));

        const N: usize = 100;
        let mut handles = Vec::with_capacity(N);
        for _ in 0..N {
            let semaphore = Arc::clone(&semaphore);
            handles.push(lore_base::lore_spawn!(async move {
                let permit_count = fragment_permit_count(1);
                let p = semaphore
                    .acquire_many_owned(permit_count)
                    .await
                    .expect("acquire");
                drop(p);
            }));
        }
        for h in handles {
            h.await.expect("join");
        }

        assert_eq!(
            semaphore.available_permits(),
            16 * FRAGMENT_MINIMUM_COST_KIB as usize,
            "all permits must be released after the stress burst"
        );
    }

    #[tokio::test]
    async fn fragment_limiter_owned_shares_budget_with_borrowed() {
        // The two handles MUST reference the same underlying Semaphore so
        // permits acquired from one count against the other's budget. Assert
        // pointer equality directly instead of sampling the budget (which
        // other concurrent tests perturb).
        let borrowed: *const Semaphore = fragment_limiter();
        let owned_arc = fragment_limiter_owned();
        let owned: *const Semaphore = Arc::as_ptr(&owned_arc);
        assert_eq!(
            borrowed, owned,
            "fragment_limiter and fragment_limiter_owned must share the same semaphore"
        );
    }
}
