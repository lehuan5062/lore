// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use serde::Deserialize;
use serde::Serialize;

use crate::event::LoreEvent;
use crate::interface::LoreString;

#[repr(C)]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LorePathIgnoreEventData {
    pub path: LoreString,
}

pub async fn emit_path_ignore(path: &str) {
    LoreEvent::PathIgnore(LorePathIgnoreEventData { path: path.into() }).send();
}
