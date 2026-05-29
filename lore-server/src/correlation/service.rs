// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::task::Context;
use std::task::Poll;

use http::HeaderName;
use http::HeaderValue;
use http::Request;
use http::Response;
use tower::Service;
use tracing::debug;
use tracing::warn;

use super::CorrelationId;

fn add_correlation_id_header(
    headers: &mut http::HeaderMap,
    header_name: HeaderName,
    correlation_id: &str,
) {
    match HeaderValue::from_str(correlation_id) {
        Ok(val) => {
            headers.insert(header_name, val);
        }
        Err(err) => {
            warn!(correlation_id = ?correlation_id, "Error creating header from correlation ID: {err}");
        }
    }
}

/// A `tower::Service` implementation that injects a `CorrelationId`
/// if it doesn't already exist.
#[derive(Debug, Clone)]
pub struct CorrelationIdService<S> {
    pub header_name: HeaderName,
    pub inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorrelationIdService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        if let Some(correlation_id) = request.headers().get(&self.header_name) {
            debug!(correlation_id = ?correlation_id, "Found existing correlation ID");
        } else {
            let correlation_id = CorrelationId::default();
            debug!(correlation_id = ?correlation_id, "Generated correlation ID");

            add_correlation_id_header(
                request.headers_mut(),
                self.header_name.clone(),
                &correlation_id,
            );
        }

        self.inner.call(request)
    }
}
