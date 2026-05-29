// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use lore_base::lore_spawn;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;

use crate::cluster::topology::Topology;
use crate::lore_error;
use crate::lore_info;
use crate::store::composite::CompositeStore;

/// Listens out for when the Topology broadcasts refreshes
/// then calls into `CompositeStore` and instructs it to update its state
#[derive(Debug)]
pub struct TopologyRefreshSubscription {
    task: JoinHandle<()>,
}

impl TopologyRefreshSubscription {
    pub fn new(topology: Arc<dyn Topology + Send + Sync>, store: Arc<CompositeStore>) -> Self {
        let weak_store = Arc::downgrade(&store);
        let mut subscription = topology.subscribe_to_peer_refreshes();

        let task = lore_spawn!({
            async move {
                loop {
                    let change_event = match subscription.recv().await {
                        Ok(change_event) => change_event,
                        Err(error) => {
                            lore_info!("topology refresh receive error {error:?}");
                            match error {
                                RecvError::Closed => {
                                    lore_info!("stopping topology refresh subscription");
                                    break;
                                }
                                RecvError::Lagged(_) => {
                                    continue;
                                }
                            };
                        }
                    };
                    if let Some(cluster) = weak_store.upgrade() {
                        if let Err(error) = cluster.topology_peers_refreshed(change_event).await {
                            lore_error!("error doing peer refresh: {error:?}");
                        }
                    } else {
                        lore_info!("cluster dropped - stopping subscription");
                        break;
                    }
                }
            }
        });
        Self { task }
    }
}

impl Drop for TopologyRefreshSubscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}
