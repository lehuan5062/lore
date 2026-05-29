// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore::interface::LoreLogConfig;
use lore::interface::LoreLogEventData;
use lore::interface::LoreLogLevel;
use lore::log::log_level;

use crate::cli::LoreCli;
use crate::cli::LoreCliError;
use crate::eprintln;
use crate::println;
use crate::styling::LogStyles;

pub fn log_config_from_args(args: &LoreCli) -> Result<LoreLogConfig, LoreCliError> {
    let mut config = LoreLogConfig {
        file: 1,
        file_max_count: 7,
        file_rolling: 1,
        level: LoreLogLevel::Info,
        ..Default::default()
    };

    // Check and set the log level
    if let Some(level) = &args.level {
        config.level = log_level_from_string(level)?;
    }

    // Check and set the debug mode
    if args.debug {
        config.level = LoreLogLevel::Debug;
    }

    Ok(config)
}

pub fn log_level_from_string(level_string: &str) -> Result<LoreLogLevel, LoreCliError> {
    let level = match level_string.to_lowercase().as_str() {
        "trace" => LoreLogLevel::Trace,
        "debug" => LoreLogLevel::Debug,
        "info" => LoreLogLevel::Info,
        "warn" => LoreLogLevel::Warn,
        "error" => LoreLogLevel::Error,
        _ => return Err(LoreCliError::ParseLogLevel(level_string.to_string())),
    };

    Ok(level)
}

pub fn handle_log_event(event: &LoreLogEventData) {
    handle_log_event_with(event);
}

pub fn handle_log_event_with(event: &LoreLogEventData) {
    if event.level < log_level() {
        return;
    }

    // Info messages that currently are output don't need a level
    if event.level == LoreLogLevel::Info {
        println!("{}", event.message.as_str());
        return;
    }

    let log_message = format!("[{}] {}", event.level, event.message.as_str());

    let color = LogStyles::from_level(event.level);

    if event.level == LoreLogLevel::Error {
        eprintln!("{}{}{}", color, log_message.as_str(), anstyle::Reset);
    } else {
        println!("{}{}{}", color, log_message.as_str(), anstyle::Reset);
    }
}
