// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use lore_error_set::prelude::*;
use tokio::sync::broadcast::Receiver;

use crate::cluster::peer::PeerInfo;

#[error_set]
pub enum RefreshLoopError {}

/// Understands an underlying infrastructure, abstracting the implementation of different
/// platforms/vendors into a common connection info
#[async_trait]
pub trait Topology: std::fmt::Debug {
    /// Does this topology support running a refresh loop?
    fn supports_refresh_loop(&self) -> bool {
        true
    }

    /// Runs auto refresh logic in a loop. Any changes get broadcast. Up to the implementation what
    /// happens under the hood to achieve this.
    async fn refresh_loop(self: Arc<Self>) -> Result<(), RefreshLoopError>;

    /// If there are any refreshes to the underlying infrastructure, this receiver should receive
    /// a notification
    fn subscribe_to_peer_refreshes(self: Arc<Self>) -> Receiver<HashSet<PeerInfo>>;
}
