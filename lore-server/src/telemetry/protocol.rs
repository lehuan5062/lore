// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum StorageProtocol {
    StorageV0,
    StorageV1,
    StorageV4,
    Replication,
}

impl StorageProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StorageV0 => "storage.v0",
            Self::StorageV1 => "storage.v1",
            Self::StorageV4 => "storage.v4",
            Self::Replication => "replication",
        }
    }
}

impl fmt::Display for StorageProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Transport {
    Grpc,
    Quic,
}

impl Transport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grpc => "grpc",
            Self::Quic => "quic",
        }
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_strings() {
        assert_eq!(StorageProtocol::StorageV0.as_str(), "storage.v0");
        assert_eq!(StorageProtocol::StorageV1.as_str(), "storage.v1");
        assert_eq!(StorageProtocol::StorageV4.as_str(), "storage.v4");
        assert_eq!(StorageProtocol::Replication.as_str(), "replication");
        assert_eq!(Transport::Grpc.as_str(), "grpc");
        assert_eq!(Transport::Quic.as_str(), "quic");
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", StorageProtocol::StorageV0), "storage.v0");
        assert_eq!(format!("{}", StorageProtocol::Replication), "replication");
        assert_eq!(format!("{}", Transport::Grpc), "grpc");
        assert_eq!(format!("{}", Transport::Quic), "quic");
    }
}
