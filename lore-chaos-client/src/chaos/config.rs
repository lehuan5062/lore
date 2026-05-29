// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use lore::interface::LoreString;

use crate::cli::ChaosCommandArgs;
use crate::cli::CliArgs;

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub iterations: Option<u32>,
    pub time_limit: Option<Duration>,
    pub dry_run: bool,
    pub repo_path: String,
    pub user_progressed: bool,
    pub write_operations: Option<String>,
    pub replay_operations: Option<String>,
}

impl RunnerConfig {
    pub fn from_cli(args: &CliArgs, chaos_args: &ChaosCommandArgs) -> Self {
        Self {
            iterations: chaos_args.iterations,
            time_limit: chaos_args
                .time_limit_mins
                .map(|limit| Duration::from_secs_f32(limit * 60.0)),
            dry_run: chaos_args.dry_run,
            repo_path: args.repository_path.clone(),
            user_progressed: chaos_args.user_progressed,
            write_operations: chaos_args.write_operations.clone(),
            replay_operations: chaos_args.replay_operations.clone(),
        }
    }

    pub fn path_in_repo(&self, path: &Path) -> PathBuf {
        PathBuf::from(&self.repo_path).join(path)
    }

    pub fn lore_string_to_path_in_repo(&self, file: &LoreString) -> PathBuf {
        PathBuf::from(&self.repo_path).join(PathBuf::from(file.to_string()))
    }
}
