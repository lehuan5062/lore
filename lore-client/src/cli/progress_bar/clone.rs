// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Three-phase clone progress indicator.
//!
//! File counts drive `progress`/`max` so the bar counter reads
//! `file_complete/file_count`. Byte totals go into the status message. The
//! three phases (spinner → growing determinate → final determinate) are
//! selected from the `(file_count, discovery_complete)` pair below.

use super::ProgressBar;
use crate::util::format_bytes_to_string;

pub fn apply_clone_progress(
    file_count: u64,
    file_complete: u64,
    bytes_transferred: u64,
    bytes_total: u64,
    discovery_complete: u8,
    bar: &ProgressBar,
) {
    if file_count <= 1 {
        // Spinner: file count not yet known.
        let bytes_msg = if bytes_transferred > 0 {
            format!(
                "Cloning ... {}/{}",
                format_bytes_to_string(bytes_transferred),
                format_bytes_to_string(bytes_total)
            )
        } else {
            String::from("Cloning ...")
        };
        bar.set_message(bytes_msg);
    } else {
        // Determinate: promote the spinner by setting max, and reflect whether
        // the total can still grow.
        bar.set_max_progress(file_count);
        bar.set_progress(file_complete);
        bar.set_growing(discovery_complete == 0);
        bar.set_message(format!(
            "{}/{}",
            format_bytes_to_string(bytes_transferred),
            format_bytes_to_string(bytes_total)
        ));
    }
}
