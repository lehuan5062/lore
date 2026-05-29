// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::any::type_name;
use std::path::Path;
use std::time::Instant;

use lore_base::version::LORE_LIBRARY_VERSION;
use lore_revision::interface::LoreArray;
use lore_revision::lore_debug;
use lore_revision::util::path::RelativePath;

use crate::interface::LoreString;

#[allow(dead_code)]
pub fn convert_user_optional_path(
    repository_path: impl AsRef<Path>,
    path: &str,
) -> Result<RelativePath, lore_revision::util::path::PathError> {
    RelativePath::new_from_user_path(repository_path.as_ref(), path)
}

pub fn convert_user_paths(
    repository_path: impl AsRef<Path>,
    paths: LoreArray<LoreString>,
) -> Result<Vec<RelativePath>, lore_revision::util::path::PathError> {
    let mut relative_paths = Vec::with_capacity(paths.len());

    for path in paths.as_slice() {
        relative_paths.push(RelativePath::new_from_user_path(
            repository_path.as_ref(),
            path.as_str(),
        )?);
    }

    Ok(relative_paths)
}

pub fn log_command_info<T, A>(_caller: &T, args: &A)
where
    A: std::fmt::Debug,
{
    lore_debug!(
        "Executing command: {} {}",
        type_name::<T>(),
        LORE_LIBRARY_VERSION.as_str()
    );
    lore_debug!("Command arguments: {args:?}");
}

pub fn log_command_done<T>(_caller: &T, start: Instant) {
    lore_debug!(
        "Finished command: {} ({:.2}s)",
        type_name::<T>(),
        start.elapsed().as_secs_f32()
    );
}
