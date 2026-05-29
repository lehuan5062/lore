// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#![allow(unreachable_code)]

use lore_revision::repository::RepositoryContext;

async fn read_handle_cannot_store(repo: &RepositoryContext) {
    let handle = repo.read_mutable_store();
    let _ = handle
        .store(todo!(), todo!(), todo!(), todo!())
        .await;
}

fn main() {}
