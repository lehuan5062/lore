// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_error_set::prelude::*;

// Reexport the unimplemented OS generic module
#[cfg(not(target_os = "windows"))]
mod stub;

#[cfg(not(target_os = "windows"))]
mod os_specific {
    pub use super::stub::UdsListener;
    pub use super::stub::UdsStream;
    pub use super::stub::uds_supported;
}

// Reexport the windows specific module
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
mod os_specific {
    pub use super::windows::UdsListener;
    pub use super::windows::UdsStream;
    pub use super::windows::uds_supported;
}

// Reexport everything from the private OS specific networking module
pub use os_specific::*;

#[error_set]
pub enum UdsListenerError {}

#[error_set]
pub enum UdsAcceptError {}

#[error_set]
pub enum UdsConnectionError {}
