// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use zerocopy::FromBytes;
use zerocopy::Immutable;
use zerocopy::IntoBytes;

use super::BranchId;
use super::Hash;

#[derive(Clone, Debug, Default, PartialEq, FromBytes, IntoBytes, Immutable)]
pub struct BranchPoint {
    pub branch: BranchId,
    pub revision: Hash,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BranchMetadata {
    pub id: BranchId,
    pub name: String,
    pub category: String,
    pub latest: Hash,
    pub creator: String,
    pub created: u64,
    pub stack: Vec<BranchPoint>,
}

impl BranchMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BranchId,
        name: String,
        category: String,
        latest: Hash,
        creator: String,
        created: u64,
        stack: Vec<BranchPoint>,
    ) -> Self {
        Self {
            id,
            name,
            category,
            latest,
            creator,
            created,
            stack,
        }
    }
}
