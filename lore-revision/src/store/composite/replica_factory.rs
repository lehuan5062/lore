// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

use lore_base::lore_spawn;
use lore_error_set::prelude::*;
use tokio::task::JoinError;
use tokio::task::JoinSet;

use crate::cluster::peer::PeerInfo;
use crate::store::composite::ReplicationTarget;

#[error_set]
pub enum MakeReplicaTargetsError {}

/// Paired read and write replication targets produced from a single `PeerInfo`.
/// The read target is typically QUIC-backed and the write target is gRPC-backed.
#[derive(Debug)]
pub struct ReplicaTargets {
    pub read: Option<ReplicationTarget>,
    pub write: Option<ReplicationTarget>,
}

#[async_trait::async_trait]
pub trait ReplicaFactory: Debug + Send + Sync {
    /// Take a `PeerInfo` and convert to paired `ReplicaTargets`
    async fn make_replica_target(
        &self,
        peer_info: &PeerInfo,
    ) -> Result<ReplicaTargets, Box<dyn std::error::Error + Send + Sync>>;

    /// Take a set of `PeerInfo` and build replica targets in parallel.
    /// Intermittent connection issues might mean that 1 peer connection fails while many others succeed.
    /// It is up to the caller to decide what to do with the failures
    async fn make_replica_targets(
        self: Arc<Self>,
        infos: &HashSet<PeerInfo>,
    ) -> Result<Vec<Result<ReplicaTargets, MakeReplicaTargetsError>>, JoinError>
    where
        Self: 'static,
    {
        let mut build_targets_set = JoinSet::new();
        for info in infos {
            let info = info.clone();
            let builder = self.clone();
            lore_spawn!(build_targets_set, async move {
                builder.make_replica_target(&info).await.map_err(|error| {
                    MakeReplicaTargetsError::internal(format!(
                        "failed to make peer {info}: {error}"
                    ))
                })
            });
        }
        let mut output = Vec::with_capacity(build_targets_set.len());
        while let Some(join_result) = build_targets_set.join_next().await {
            output.push(join_result?);
        }
        Ok(output)
    }
}
