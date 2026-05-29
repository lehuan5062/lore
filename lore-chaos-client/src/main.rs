// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
mod chaos;
mod chaos_main;
mod cli;
mod lore;
mod operations;
mod parallel;
mod probability;
mod tracing;

fn main() {
    chaos_main::chaos_main();
}
