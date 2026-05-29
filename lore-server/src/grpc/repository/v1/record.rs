// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_base::types::Hash;
use lore_proto::lore::model::v1 as model_v1;
use lore_revision::lore::RepositoryId;
use lore_revision::repository::RepositoryMetadata;

/// Build a v1 `Repository` response record from already-loaded metadata.
///
/// `metadata_hash` is the current repository metadata pointer; callers
/// that have just performed a mutation pass the new pointer, callers
/// returning a "what was here" record (e.g. `RepositoryDelete`) pass the
/// pre-mutation pointer.
pub(super) fn build_repository(
    id: RepositoryId,
    metadata: &RepositoryMetadata,
    metadata_hash: Hash,
) -> model_v1::Repository {
    model_v1::Repository {
        id: id.into(),
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        default_branch_id: metadata.default_branch.into(),
        default_branch_name: metadata.default_branch_name.clone(),
        creator: metadata.creator.clone(),
        created: metadata.created,
        metadata: metadata_hash.into(),
    }
}
