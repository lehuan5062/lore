// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub mod contents;

use std::collections::HashMap;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use tracing::info_span;

use crate::http::server::ServerState;

async fn trace(
    Path(path_params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Response {
    let repository_id = path_params.get("repository_id");
    let span = info_span!("http_repository", repository_id);
    next.run(request).instrument(span.or_current()).await
}

pub fn create_router<S>(shared_state: ServerState) -> Router<S> {
    let contents_router = contents::create_router(shared_state.clone());

    Router::new()
        .nest("/content", contents_router)
        .layer(middleware::from_fn(trace))
        .with_state(shared_state)
}
