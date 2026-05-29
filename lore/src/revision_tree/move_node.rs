// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_move` — reparent and/or rename a node while
//! preserving its `file_id`, so the resulting revision graph records a
//! true move instead of a delete-plus-add pair. The Rust module is named
//! `move_node` because `move` is a reserved keyword; the C symbol stays
//! `lore_revision_tree_move`.

use lore_revision::interface::LoreString;
use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_move`.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeMoveArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub node_id: NodeID,
    pub destination_parent_id: NodeID,
    pub dst_name: LoreString,
}
