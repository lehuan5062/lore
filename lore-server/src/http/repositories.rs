// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub mod repository;

use axum::Router;

use crate::http::server::ServerState;

pub fn create_router<S>(shared_state: ServerState) -> Router<S> {
    let repository_router = repository::create_router(shared_state.clone());

    Router::new()
        .nest("/{repository_id}", repository_router)
        .with_state(shared_state)
}
