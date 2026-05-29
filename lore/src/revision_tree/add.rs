// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_add` — add a leaf or empty directory child under a
//! parent node. `kind` is an opaque `u32` matching the `NodeKind` encoding
//! in `lore_revision::node` (FILE=1, DIRECTORY=2, LINK=3); the verb rejects
//! any other value with `LORE_ERROR_CODE_INVALID_ARGUMENTS`.

use lore_base::types::Address;
use lore_revision::interface::LoreString;
use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_add`.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeAddArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub parent_node_id: NodeID,
    pub name: LoreString,
    pub kind: u32,
    pub mode: u16,
    pub size: u64,
    pub address: Address,
}
