// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::time::Duration;

use crate::chaos::config::RunnerConfig;
use crate::cli::CliArgs;
use crate::cli::ParallelCommandArgs;

#[derive(Debug)]
pub struct ParallelConfig {
    pub runners: u32,
    pub chaos_config: RunnerConfig,
}

impl ParallelConfig {
    pub fn from_cli(args: &CliArgs, parallel_args: &ParallelCommandArgs) -> Self {
        Self {
            runners: parallel_args.runners,
            chaos_config: RunnerConfig {
                iterations: parallel_args.iterations,
                time_limit: parallel_args
                    .time_limit_mins
                    .map(|limit| Duration::from_secs_f32(limit * 60.0)),
                dry_run: false,
                repo_path: args.repository_path.clone(),
                user_progressed: false,
                write_operations: None,
                replay_operations: None,
            },
        }
    }
}
