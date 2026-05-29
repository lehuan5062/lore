// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub fn normalize_remote_url(remote_url: &str) -> &str {
    let url = remote_url.trim_end_matches('/');
    // Strip protocol specifier (e.g. "lore://", "urc://", "grpc://") so that
    // URLs differing only in scheme still match the same shared store entry.
    if let Some(rest) = url.split_once("://") {
        rest.1
    } else {
        url
    }
}
