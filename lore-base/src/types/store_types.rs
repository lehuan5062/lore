// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyType {
    Untyped = 0,
    BranchMetadata = 1,
    BranchId = 2,
    BranchLatestPointer = 3,
    RepositoryMetadata = 4,
    RepositoryId = 5,
    Instance = 6,
}

impl TryFrom<u8> for KeyType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyType::Untyped),
            1 => Ok(KeyType::BranchMetadata),
            2 => Ok(KeyType::BranchId),
            3 => Ok(KeyType::BranchLatestPointer),
            4 => Ok(KeyType::RepositoryMetadata),
            5 => Ok(KeyType::RepositoryId),
            6 => Ok(KeyType::Instance),
            other => Err(other),
        }
    }
}

impl TryFrom<u32> for KeyType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > u8::MAX as u32 {
            return Err(value);
        }
        KeyType::try_from(value as u8).map_err(|_unknown| value)
    }
}
