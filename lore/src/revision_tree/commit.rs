// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_commit` — freeze the handle's tree, write the 320-
//! byte revision record, and atomically advance the target branch tip. The
//! options struct carries the `remote_write` flag (`u8`, 0 or 1, not
//! `bool`) selecting between local-only and remote-uploading commits.

use lore_base::types::BranchId;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Tuneables for `lore_revision_tree_commit`.
///
/// `remote_write` is `0` or `1`. The flag is encoded as `u8` rather than
/// `bool` because `bool` is not `#[repr(C)]`-stable across FFI; this
/// matches the existing storage API convention
/// (`LoreStoragePutItem::remote_write`).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeCommitOptions {
    pub remote_write: u8,
}

/// Arguments for `lore_revision_tree_commit`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeCommitArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub branch: BranchId,
    pub options: LoreRevisionTreeCommitOptions,
}
