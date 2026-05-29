// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#![allow(unreachable_code)]

use lore_revision::repository::RepositoryContext;

async fn read_handle_cannot_compare_and_swap(repo: &RepositoryContext) {
    let handle = repo.read_mutable_store();
    let _ = handle
        .compare_and_swap(todo!(), todo!(), todo!(), todo!(), todo!())
        .await;
}

fn main() {}
