// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use clap::Args;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(name = "lore-chaos-client")]
#[command(about = "A CLI client for Lore repository operations with chaos engineering")]
#[command(version = "0.2.0")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,

    /// Path to the repository to run chaos operations on
    #[arg(short, long, default_value = ".", global = true)]
    pub repository_path: String,

    #[arg(short, long, global = true)]
    pub offline: bool,

    /// Include everything from the log file in the console as well
    #[arg(long, global = true)]
    pub log_to_console: bool,

    /// Log file path (optional)
    #[arg(long, default_value = "chaos_client.log", global = true)]
    pub log_file: String,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Run commands on the repo from a single thread
    Chaos(ChaosCommandArgs),
    /// Run commands on the repo from multiple threads simultaneously
    Parallel(ParallelCommandArgs),
}

#[derive(Args, Debug)]
pub struct ChaosCommandArgs {
    /// Number of chaos iterations to run
    #[arg(short, long)]
    pub iterations: Option<u32>,

    /// Number of minutes to stop running operations after.
    #[arg(short, long)]
    pub time_limit_mins: Option<f32>,

    /// Dry run mode - show what would be done without making changes
    #[arg(short, long)]
    pub dry_run: bool,

    /// Require the user to press enter between URC commands
    #[arg(short, long)]
    pub user_progressed: bool,

    /// File to write all operations to as JSON. Can be combined with --dry-run to just make the file.
    #[arg(long)]
    pub write_operations: Option<String>,

    /// JSON file to read all operations from rather than using the RNG seed
    #[arg(long, conflicts_with_all = ["iterations", "seed", "dry_run", "write_operations"])]
    pub replay_operations: Option<String>,

    /// RNG seed
    #[arg(short, long)]
    pub seed: Option<u64>,
}

#[derive(Args, Debug)]
pub struct ParallelCommandArgs {
    #[arg(short, long, default_value = "1")]
    pub iterations: Option<u32>,

    #[arg(short, long)]
    pub time_limit_mins: Option<f32>,

    /// How many chaos runners to have in parallel
    #[arg(long, default_value = "2")]
    pub runners: u32,

    /// RNG seed
    #[arg(short, long)]
    pub seed: Option<u64>,
}
