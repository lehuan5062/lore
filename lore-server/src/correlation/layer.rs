// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use http::HeaderName;
use lore_transport::grpc::CORRELATION_ID_HEADER;
use tower::Layer;
use tower::layer::util::Stack;
use tower_http::classify::GrpcCode;
use tower_http::classify::GrpcErrorsAsFailures;
use tower_http::classify::SharedClassifier;
use tower_http::trace::DefaultOnBodyChunk;
use tower_http::trace::DefaultOnEos;
use tower_http::trace::DefaultOnFailure;
use tower_http::trace::DefaultOnRequest;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::GrpcMakeClassifier;
use tower_http::trace::HttpMakeClassifier;
use tower_http::trace::TraceLayer;
use tracing::Level;

use super::service::CorrelationIdService;
use super::span::MakeCorrelationIdSpan;

/// Type aliases for the overly complicated `tower_http`
/// layer generics
type CorrelationHttpTraceLayer = TraceLayer<
    HttpMakeClassifier,
    MakeCorrelationIdSpan,
    DefaultOnRequest,
    DefaultOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    DefaultOnFailure,
>;

type CorrelationGrpcTraceLayer = TraceLayer<
    GrpcMakeClassifier,
    MakeCorrelationIdSpan,
    DefaultOnRequest,
    DefaultOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    DefaultOnFailure,
>;

/// Wraps a `CorrelationIdService` in a `Layer` implementation
/// to inject correlation IDs if they don't exist
#[derive(Debug, Clone)]
pub struct CorrelationIdLayer;

impl<S> Layer<S> for CorrelationIdLayer {
    type Service = CorrelationIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let header_name = HeaderName::from_static(CORRELATION_ID_HEADER);
        CorrelationIdService { header_name, inner }
    }
}

pub struct TraceLayerConfig {
    pub response: Level,
    pub failure: Level,
    pub grpc_codes_as_success: Vec<GrpcCode>,
}

impl Default for TraceLayerConfig {
    fn default() -> Self {
        Self {
            response: Level::DEBUG,
            failure: Level::WARN,
            grpc_codes_as_success: vec![GrpcCode::NotFound],
        }
    }
}

/// Helper to add a tracer layer that will add the request's
/// correlation ID to the main span
#[derive(Debug, Clone)]
pub struct CorrelationIdLayerBuilder<L> {
    layer: L,
}

impl Default for CorrelationIdLayerBuilder<CorrelationIdLayer> {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrelationIdLayerBuilder<CorrelationIdLayer> {
    pub const fn new() -> Self {
        Self {
            layer: CorrelationIdLayer {},
        }
    }
}

impl<L> CorrelationIdLayerBuilder<L> {
    pub fn build(self) -> L {
        self.layer
    }

    pub fn with_http_tracer(
        self,
    ) -> CorrelationIdLayerBuilder<Stack<CorrelationHttpTraceLayer, L>> {
        CorrelationIdLayerBuilder {
            layer: Stack::new(
                TraceLayer::new_for_http().make_span_with(MakeCorrelationIdSpan),
                self.layer,
            ),
        }
    }

    pub fn with_grpc_tracer(
        self,
        layer_config: TraceLayerConfig,
    ) -> CorrelationIdLayerBuilder<Stack<CorrelationGrpcTraceLayer, L>> {
        let grpc_errors_as_failures = layer_config
            .grpc_codes_as_success
            .into_iter()
            .fold(GrpcErrorsAsFailures::new(), |acc, code| {
                acc.with_success(code)
            });

        CorrelationIdLayerBuilder {
            layer: Stack::new(
                TraceLayer::new(SharedClassifier::new(grpc_errors_as_failures))
                    .on_response(DefaultOnResponse::default().level(layer_config.response))
                    .on_failure(DefaultOnFailure::default().level(layer_config.failure))
                    .make_span_with(MakeCorrelationIdSpan),
                self.layer,
            ),
        }
    }
}
