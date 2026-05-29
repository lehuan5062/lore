// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub mod content;
mod put_repository_content;

use axum::Router;
use axum::routing;

use crate::http::server::ServerState;

pub fn create_router<S>(shared_state: ServerState) -> Router<S> {
    let content_router = content::create_router(shared_state.clone());

    Router::new()
        .route("/", routing::put(put_repository_content::handler))
        .nest("/{address}", content_router)
        .with_state(shared_state.into())
}
