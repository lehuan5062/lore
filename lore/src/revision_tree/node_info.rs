// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_node_info` — fetch the per-node record for a single
//! `NodeID`. For the revision root the response additionally carries
//! Metadata-fragment-derived fields (timestamp, author, key set) via the
//! event's `root_info` slot.

use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_node_info`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeNodeInfoArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub node_id: NodeID,
}
