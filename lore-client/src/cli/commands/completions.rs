// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::fs;
use std::io;
use std::path::Path;

use clap::Args;
use clap::CommandFactory;
use clap_complete::Shell;
use clap_complete::generate;

use crate::cli::LoreCli;

#[cfg(target_os = "windows")]
const FILENAME: &str = "lore.ps1";
#[cfg(target_os = "macos")]
const FILENAME: &str = "_lore";
#[cfg(target_os = "linux")]
const FILENAME: &str = "lore";

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate autocompletions for
    #[clap(value_name = "shell")]
    pub shell: Shell,
    /// Directory path to write the autocompletion script to
    #[clap(value_name = "path")]
    pub path: Option<String>,
}

pub fn handle_completions_commands(args: &CompletionsArgs) -> u8 {
    let mut cmd = LoreCli::command();

    let output_file = args
        .path
        .as_ref()
        .map(Path::new)
        .and_then(|path| fs::create_dir_all(path).ok().map(|_| path))
        .and_then(|path| fs::File::create(path.join(FILENAME)).ok());

    if let Some(mut file) = output_file {
        generate(args.shell, &mut cmd, "lore", &mut file);
    } else {
        generate(args.shell, &mut cmd, "lore", &mut io::stdout());
    }

    0u8
}
