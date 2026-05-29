// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use clap::Args;
use clap::Subcommand;

use crate::println;
use crate::styling::CommonStyles;

#[derive(Args)]
pub struct LogfileArgs {
    #[command(subcommand)]
    pub command: LogfileCommands,
}

#[derive(Subcommand)]
pub enum LogfileCommands {
    /// Info
    Info,
}

// TODO(vri): Add some more useful information
fn handle_logfile_info() -> u8 {
    let logfile_path = lore::log_file_path();
    println!(
        "{}Location:{} {}",
        CommonStyles::HEADERS,
        anstyle::Reset,
        logfile_path.as_str()
    );

    0
}

pub fn handle_logfile_commands(cmd: &LogfileCommands) -> u8 {
    match cmd {
        LogfileCommands::Info => {
            return handle_logfile_info();
        }
    }
}
