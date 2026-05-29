// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;

/// Default bounded channel capacity for discovery → execution pipeline.
pub const DEFAULT_WORK_CHANNEL_CAPACITY: usize = 200_000;

/// Statistics accumulated by the discovery (producer) side.
/// When the producer finishes and the channel drains, these are the final totals.
#[derive(Default)]
pub struct DiscoveryStats {
    pub total_files: AtomicU64,
    pub total_bytes: AtomicU64,
    pub complete: AtomicBool,
}
