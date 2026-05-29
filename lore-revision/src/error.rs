// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use crate::lore::execution_context;
use crate::lore_debug;
use crate::lore_error;

pub trait LoreResultExt<T, ToErr: std::error::Error> {
    fn emit_map_err(self, err: ToErr) -> Result<T, ToErr>;
    fn debug_map_err(self, err: ToErr) -> Result<T, ToErr>;
}

impl<T, FromErr, ToErr> LoreResultExt<T, ToErr> for Result<T, FromErr>
where
    FromErr: std::error::Error,
    ToErr: std::error::Error,
{
    fn emit_map_err(self, to_err: ToErr) -> Result<T, ToErr> {
        self.map_err(|from_err| {
            if !execution_context()
                .failure
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                lore_error!("{to_err}: {from_err} | {from_err:?}");
            }
            to_err
        })
    }

    fn debug_map_err(self, to_err: ToErr) -> Result<T, ToErr> {
        self.map_err(|from_err| {
            if !execution_context()
                .failure
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                lore_debug!("{to_err}: {from_err} | {from_err:?}");
            }
            to_err
        })
    }
}

pub trait LoreErrorExt<T, E> {
    fn emit(self) -> Result<T, E>;
    fn debug(self) -> Result<T, E>;
}

impl<T, E> LoreErrorExt<T, E> for E
where
    E: std::error::Error,
{
    fn emit(self) -> Result<T, E> {
        if !execution_context()
            .failure
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            lore_error!("{self}");
        }
        Err(self)
    }

    fn debug(self) -> Result<T, E> {
        if !execution_context()
            .failure
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            lore_debug!("{self}");
        }
        Err(self)
    }
}
