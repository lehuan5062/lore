// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::chaos::config::RunnerConfig;

pub fn add_or_modify(path: &Path, config: &RunnerConfig, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(config.path_in_repo(path))?;
    file.write_all(contents.as_bytes())
}

pub fn delete(path: &Path, config: &RunnerConfig) -> std::io::Result<()> {
    std::fs::remove_file(config.path_in_repo(path))
}
