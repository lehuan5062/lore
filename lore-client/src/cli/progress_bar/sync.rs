// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore::interface::LoreRevisionSyncProgressEventData;

use super::ProgressBar;

pub fn apply_sync_progress_to_bar(
    progress_bar: &ProgressBar,
    data: &LoreRevisionSyncProgressEventData,
) {
    progress_bar.set_max_progress((data.file_delete_total + data.file_update_total) as u64);
    progress_bar.set_progress((data.file_delete + data.file_update) as u64);
    progress_bar.set_growing(data.discovery_complete == 0);
    progress_bar.set_message(format!(
        "{} automerged, {} conflicts",
        data.file_automerge, data.file_conflict,
    ));
}
