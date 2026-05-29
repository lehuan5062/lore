// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::cli::CliArgs;

pub fn setup_tracing(args: &CliArgs) {
    let log_file_layer = std::fs::File::create(&args.log_file)
        .unwrap_or_else(|_| panic!("Failed to open log file {}", args.log_file));

    let log_layer = layer()
        .pretty()
        .with_writer(log_file_layer)
        .with_ansi(false)
        .with_filter(LevelFilter::INFO);

    let stdout_filter = if args.log_to_console {
        LevelFilter::INFO
    } else {
        LevelFilter::WARN
    };
    let stdout_layer = tracing_subscriber::fmt::layer().with_filter(stdout_filter);

    tracing_subscriber::registry()
        .with(log_layer)
        .with(stdout_layer)
        .init();
}
