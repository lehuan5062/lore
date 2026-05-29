// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
// Bridge the connect() function to maintain the 3-argument signature.
// lore-transport's connect() takes max_connections as a 4th parameter.
pub async fn connect(
    remote_url: &str,
    identity: &str,
    repository: crate::lore::RepositoryId,
) -> Result<std::sync::Arc<lore_transport::Connection>, lore_transport::ProtocolError> {
    let max_connections = crate::lore::execution_context().globals().max_connections as usize;
    lore_transport::connect(remote_url, identity, repository, max_connections).await
}
