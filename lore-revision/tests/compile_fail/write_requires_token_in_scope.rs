// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_revision::repository::RepositoryContext;

// A read-only command callback receives only `&RepositoryContext` — no
// `RepositoryWriteToken` is bound in this scope. Attempting to reach
// `write_mutable_store` must fail to compile because the token argument
// cannot be named.
async fn read_only_callback_cannot_write(repo: &RepositoryContext) {
    let _ = repo.write_mutable_store(&token);
}

fn main() {}
