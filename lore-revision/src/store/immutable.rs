// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
// Re-export all immutable store types from lore-storage for internal use
pub(crate) use lore_storage::local::immutable_store::*;

// Backward compatibility alias
pub(crate) type ImmutableStore = lore_storage::local::immutable_store::LocalImmutableStore;
