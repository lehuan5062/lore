// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::LazyLock;

pub static LORE_LIBRARY_VERSION: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_LORE_REVISION_NUMBER")
    )
});

pub static LORE_LIBRARY_VERSION_CSTR: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "+",
    env!("VERGEN_LORE_REVISION_NUMBER"),
    "\0"
);
