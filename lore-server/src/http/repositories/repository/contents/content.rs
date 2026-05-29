// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub mod get_repository_content;
pub mod presign_repository_content;

use std::collections::HashMap;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing;
use tracing::Instrument;
use tracing::info_span;

use crate::http::server::ServerState;

async fn trace(
    Path(path_params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Response {
    let content_address = path_params.get("address");
    let span = info_span!("http_repository_content", content_address);
    next.run(request).instrument(span.or_current()).await
}

pub fn create_router<S>(shared_state: ServerState) -> Router<S> {
    Router::new()
        .route("/", routing::get(get_repository_content::handler))
        .route(
            "/presign",
            routing::post(presign_repository_content::handler),
        )
        .layer(middleware::from_fn(trace))
        .with_state(shared_state.into())
}
