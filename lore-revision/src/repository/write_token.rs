// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Per-repository-path write mutex registry backing [`RepositoryWriteToken`].
//!
//! Each write-mode command holds a guard on a path-keyed
//! `tokio::sync::Mutex<()>`; as long as the guard (and therefore the token
//! that owns it) is alive, no other in-process write command for the same
//! repository can run. Read-only commands never touch this registry.
//!
//! The registry lives here in `lore-revision` so
//! [`RepositoryWriteToken::acquire`](super::RepositoryWriteToken::acquire) is
//! the only way to obtain a guard — callers cannot separately take the mutex
//! and forget to mint the token, or vice versa.
//!
//! Server-side code stays outside this system: its `RepositoryContext`
//! instances carry `write_token: None` and reach the mutable store via the
//! feature-gated raw accessor, relying on per-bucket `RwLock`s for
//! concurrency instead of the client dispatcher's per-repo write mutex.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;

static WRITE_MUTEXES: OnceLock<RwLock<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>> =
    OnceLock::new();

pub(crate) fn write_mutex_for_path(path: &Path) -> Arc<tokio::sync::Mutex<()>> {
    let mutexes = WRITE_MUTEXES.get_or_init(|| RwLock::new(HashMap::new()));
    if let Some(existing) = mutexes.read().unwrap().get(path) {
        return existing.clone();
    }
    mutexes
        .write()
        .unwrap()
        .entry(path.to_path_buf())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}
