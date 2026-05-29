// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_metadata_set` — record a `(key, value, format)`
//! triple on the in-progress revision's metadata. A subsequent set on the
//! same key overwrites the previous value in the same uncommitted handle
//! state. `format` is a `u32` matching the existing
//! `LoreRevisionMetadataSetArgs::formats` element type.

use lore_revision::interface::LoreString;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_metadata_set`.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeMetadataSetArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub key: LoreString,
    pub value: LoreString,
    pub format: u32,
}
