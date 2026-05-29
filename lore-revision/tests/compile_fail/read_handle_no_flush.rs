// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_revision::repository::RepositoryContext;

async fn read_handle_cannot_flush(repo: &RepositoryContext) {
    let handle = repo.read_mutable_store();
    let _ = handle.flush(true).await;
}

fn main() {}
