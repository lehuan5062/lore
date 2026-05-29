// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use lore_error_set::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::change;
use crate::change::NodeChange;
use crate::diff;
use crate::errors::*;
use crate::event;
use crate::interface::LoreFileAction;
use crate::interface::LoreString;
use crate::lore::Address;
use crate::lore::Hash;
use crate::node::INVALID_NODE;
use crate::repository::RepositoryContext;
use crate::state::State;
use crate::util::collect_stream::collect_stream;
use crate::util::path::RelativePath;

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreRevisionDiffFileEventData {
    pub path: LoreString,
    pub action: LoreFileAction,
    pub old_is_file: u8,
    pub new_is_file: u8,
    pub old_address: Address,
    pub new_address: Address,
}

impl LoreRevisionDiffFileEventData {
    pub fn from_node_change(change: &NodeChange, old_is_file: bool, new_is_file: bool) -> Self {
        LoreRevisionDiffFileEventData {
            path: LoreString::from(&change.path),
            action: LoreFileAction::from(change.action),
            old_is_file: old_is_file.into(),
            new_is_file: new_is_file.into(),
            old_address: change.from.address,
            new_address: change.to.address,
        }
    }

    pub fn action_as_string_short(&self) -> &'static str {
        self.action.as_string_short()
    }
}

#[error_set]
pub enum DiffError {
    AddressNotFound,
    InvalidNodeHierarchy,
    InvalidPath,
    LinkNotFound,
    NodeNotFound,
    NotFound,
    Oversized,
    RevisionNotFound,
    WriteRequired,
    Disconnected,
    InvalidArguments,
    Maintenance,
    NoRemote,
    NotAuthenticated,
    NotAuthorized,
    NotConnected,
    NotSupported,
    PayloadNotFound,
    SlowDown,
    AlreadyLinked,
    BranchAdvanced,
    BranchAlreadyExists,
    BranchNotFound,
    Conflict,
    DeleteCurrent,
    DeleteDefault,
    DeleteProtected,
    Divergent,
    FileNotFound,
    IdenticalMetadata,
    LayerNotFound,
    LinkPathNotFound,
    LocalModifications,
    LockNotFound,
    LockNotOwned,
    MaxHistorySearchDepth,
    NotALayer,
    NotALink,
    NothingStaged,
    RepositoryAlreadyExists,
    RepositoryNotFound,
    SharedStoreNotFound,
    TokenNotFound,
    MissingIdentity,
}

impl crate::event::EventError for DiffError {}

/// Calculate the difference between two revisions, as the set of changes that describe
/// going from revision 'source' to revision 'target', optionally filtered by a set of paths
pub async fn diff(
    repository: Arc<RepositoryContext>,
    source: Hash,
    target: Hash,
    paths: Option<Vec<RelativePath>>,
) -> Result<(), DiffError> {
    let state_source = State::deserialize(repository.clone(), source)
        .await
        .forward::<DiffError>("deserializing source state")?;
    let state_target = State::deserialize(repository.clone(), target)
        .await
        .forward::<DiffError>("deserializing target state")?;

    let mut diff = collect_stream(|tx| {
        diff::diff_revision_paths(repository.clone(), state_source, state_target, paths, tx)
    })
    .await
    .forward::<DiffError>("diffing states")?;
    change::sort_by_path(&mut diff);
    for change in diff {
        let mut old_is_file = false;
        if change.from.node != INVALID_NODE {
            old_is_file = change
                .from
                .state
                .node(change.from.repository.clone(), change.from.node)
                .await
                .forward::<DiffError>("deserializing source state")?
                .is_file();
        }

        let mut new_is_file = false;
        if change.to.node != INVALID_NODE {
            new_is_file = change
                .to
                .state
                .node(change.to.repository.clone(), change.to.node)
                .await
                .forward::<DiffError>("deserializing target state")?
                .is_file();
        }

        event::LoreEvent::RevisionDiffFile(LoreRevisionDiffFileEventData::from_node_change(
            &change,
            old_is_file,
            new_is_file,
        ))
        .send();
    }

    Ok(())
}
