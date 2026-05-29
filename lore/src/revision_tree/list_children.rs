// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_list_children` — stream the children of a directory
//! node as per-entry events terminated by `Complete`.

use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_list_children`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeListChildrenArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub parent_node_id: NodeID,
}
