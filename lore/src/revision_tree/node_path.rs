// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_node_path` — reconstruct the full UTF-8 path for a
//! `NodeID` by walking parent pointers. Iteration costs scale with depth;
//! per-child listings deliberately skip this work to keep their memory flat.

use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_node_path`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeNodePathArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub node_id: NodeID,
}
