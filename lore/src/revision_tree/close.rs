// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_close` — release a handle acquired via
//! `lore_revision_tree_load`. Drain semantics mirror `lore_storage_close`:
//! unregister, mark invalid, await the in-flight counter, drop.

use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_close`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeCloseArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
}
