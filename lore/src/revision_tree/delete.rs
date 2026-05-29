// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! `lore_revision_tree_delete` — mark a node and its transitive children as
//! deleted within the handle's in-progress revision. Subsequent reads in the
//! same handle do not observe the deleted subtree.

use lore_revision::node::NodeID;
use serde::Deserialize;
use serde::Serialize;

use crate::revision_tree::handle::LoreRevisionTree;

/// Arguments for `lore_revision_tree_delete`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LoreRevisionTreeDeleteArgs {
    pub id: u64,
    pub handle: LoreRevisionTree,
    pub node_id: NodeID,
}
