// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use lore_error_set::prelude::*;

use super::MetadataErrors;
use crate::errors::FileNotFound;
use crate::lore::Hash;
use crate::metadata::Metadata;
use crate::node;
use crate::node::NodeFileMetadata;
use crate::node::NodeFileMetadataBlock;
use crate::repository::RepositoryContext;
use crate::state;
use crate::util::path::RelativePath;

pub(super) async fn revision(
    repository: Arc<RepositoryContext>,
    signature: Hash,
) -> Result<Option<Metadata>, MetadataErrors> {
    let state = state::State::deserialize(repository.clone(), signature)
        .await
        .forward::<MetadataErrors>("deserializing state")?;

    let metadata_hash = state.metadata_hash();
    if !metadata_hash.is_zero() {
        let metadata = Metadata::deserialize(repository.clone(), metadata_hash)
            .await
            .forward::<MetadataErrors>("deserializing metadata")?;

        return Ok(Some(metadata));
    }

    Ok(None)
}

pub(super) async fn file(
    repository: Arc<RepositoryContext>,
    signature: Hash,
    path: &RelativePath,
) -> Result<Option<Metadata>, MetadataErrors> {
    let (state, node) = {
        let mut found_state = None;
        let mut found_node = None;

        let state = state::State::deserialize(repository.clone(), signature)
            .await
            .forward::<MetadataErrors>("deserializing state")?;

        if let Ok(node_link) = state
            .find_node_link(repository.clone(), path.as_str())
            .await
            && node_link.is_valid()
        {
            found_state = Some(state);
            found_node = Some(node_link.node);
        } else if let Ok(state_parent_self) =
            state::State::deserialize(repository.clone(), state.parent_self())
                .await
                .forward::<MetadataErrors>("deserializing parent self state")
            && let Ok(node_link) = state_parent_self
                .find_node_link(repository.clone(), path.as_str())
                .await
            && node_link.is_valid()
        {
            // Found in parent which is expected for deleted paths.
            found_state = Some(state_parent_self);
            found_node = Some(node_link.node);
        } else if let Ok(state_parent_other) =
            state::State::deserialize(repository.clone(), state.parent_other())
                .await
                .forward::<MetadataErrors>("deserializing parent other state")
            && let Ok(node_link) = state_parent_other
                .find_node_link(repository.clone(), path.as_str())
                .await
            && node_link.is_valid()
        {
            // Found in parent which is expected for deleted paths.
            found_state = Some(state_parent_other);
            found_node = Some(node_link.node);
        }

        (found_state, found_node)
    };

    if let (Some(state), Some(node)) = (state, node) {
        let metadata_node = node::node_to_file_metadata(node);
        let metadata_block_index = NodeFileMetadataBlock::index(metadata_node);
        let metadata_node_index = NodeFileMetadata::index(metadata_node);

        let metadata_block = state
            .block_file_metadata(repository.clone(), metadata_block_index)
            .await
            .forward::<MetadataErrors>("deserializing metadata block")?;

        let metadata_hash = {
            let metadata_block_reader = metadata_block.read();
            let node = metadata_block_reader.node(metadata_node_index);

            node.metadata
        };

        if metadata_hash.is_zero() {
            return Ok(None);
        } else {
            let metadata = Metadata::deserialize(repository.clone(), metadata_hash)
                .await
                .forward::<MetadataErrors>("deserializing metadata")?;

            return Ok(Some(metadata));
        }
    }

    Err(FileNotFound {
        resource: path.to_string(),
    }
    .into())
}
