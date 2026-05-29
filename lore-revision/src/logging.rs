// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub use lore_base::log::LoreLogLevel;

#[macro_export]
macro_rules! lore_trace {
    ($($args:tt)*) => { $crate::lore_base::lore_trace!($($args)*) };
}

#[macro_export]
macro_rules! lore_debug {
    ($($args:tt)+) => { $crate::lore_base::lore_debug!($($args)+) };
}

#[macro_export]
macro_rules! lore_info {
    ($($args:tt)+) => { $crate::lore_base::lore_info!($($args)+) };
}

#[macro_export]
macro_rules! lore_warn {
    ($($args:tt)+) => { $crate::lore_base::lore_warn!($($args)+) };
}

#[macro_export]
macro_rules! lore_error {
    ($($args:tt)+) => { $crate::lore_base::lore_error!($($args)+) };
}
