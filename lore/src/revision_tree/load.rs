// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_load` — open a revision tree handle on a given
//! `(store, repository, revision_hash)` tuple. `revision_hash == 0` opens an
//! empty tree suitable for committing an initial revision. The verb returns
//! the new handle on the load-complete event; no per-call correlation `id`
//! is needed because the handle itself serves as the future correlation key.

use lore_base::types::Hash;
use lore_base::types::Partition;
use serde::Deserialize;
use serde::Serialize;

use crate::storage::handle::LoreStore;

/// Arguments for `lore_revision_tree_load`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeLoadArgs {
    pub store: LoreStore,
    pub repository: Partition,
    pub revision_hash: Hash,
}
