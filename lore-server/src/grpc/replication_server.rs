// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use lore_proto::rpc::replication_service_server::ReplicationServiceServer;
use lore_storage::ImmutableStore;
use lore_telemetry::grpc_tower_layer::GrpcMetricsLayer;
use tonic::transport::Certificate;
use tonic::transport::Identity;
use tonic::transport::Server;
use tonic::transport::ServerTlsConfig;
use tracing::info;

use crate::correlation::layer::CorrelationIdLayerBuilder;
use crate::correlation::layer::TraceLayerConfig;
use crate::grpc;
use crate::grpc::replication_service::LoreReplicationService;
use crate::grpc::tower::tracing::LoreTracingLayer;

// Why Tower, why?
// Just try to make this type alias match the 'router' type in GrpcServerBuilder.
// Copy and paste from the rust compiler for sanity
type GrpcRouter = tonic::transport::server::Router<
    tower::layer::util::Stack<
        tower::ServiceBuilder<
            tower::layer::util::Stack<
                lore_telemetry::grpc_tower_layer::GrpcMetricsLayer,
                tower::layer::util::Identity,
            >,
        >,
        tower::layer::util::Stack<
            grpc::tower::tracing::LoreTracingLayer,
            tower::layer::util::Stack<
                tower::layer::util::Stack<
                    tower_http::trace::TraceLayer<
                        tower_http::classify::SharedClassifier<
                            tower_http::classify::GrpcErrorsAsFailures,
                        >,
                        crate::correlation::span::MakeCorrelationIdSpan,
                    >,
                    crate::correlation::layer::CorrelationIdLayer,
                >,
                tower::layer::util::Identity,
            >,
        >,
    >,
>;

#[derive(Debug, Default)]
pub struct GrpcReplicationServerBuilder<State>(State);

pub struct WantsImmutableStore(());

impl GrpcReplicationServerBuilder<WantsImmutableStore> {
    pub fn new() -> Self {
        Self(WantsImmutableStore(()))
    }

    pub fn with_local_immutable_store(
        self,
        immutable_store: Arc<dyn ImmutableStore>,
    ) -> anyhow::Result<GrpcReplicationServerBuilder<WantsTlsConfig>> {
        if !immutable_store.is_local() {
            return Err(anyhow!("Immutable store must be a local store"));
        }

        Ok(GrpcReplicationServerBuilder(WantsTlsConfig {
            local_immutable_store: immutable_store,
        }))
    }
}

pub struct WantsTlsConfig {
    local_immutable_store: Arc<dyn ImmutableStore>,
}

impl GrpcReplicationServerBuilder<WantsTlsConfig> {
    /// Configure TLS. The replication endpoint only supports two modes:
    /// either all three of `cert_path`, `key_path`, `cert_chain_path` are
    /// supplied (mTLS) or all three are `None` (untrusted; the caller is
    /// responsible for having validated that this is acceptable, e.g. via
    /// `verify_client_certs = false`). Anything in between is rejected
    /// rather than silently downgrading to server-only TLS.
    pub fn with_tls_config(
        self,
        cert_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
        cert_chain_path: Option<PathBuf>,
    ) -> anyhow::Result<GrpcReplicationServerBuilder<WantsHttp2Config>> {
        let tls_config = match (cert_path, key_path, cert_chain_path) {
            (Some(cert_path), Some(key_path), Some(cert_chain_path)) => {
                info!("Loading TLS certs - cert: {cert_path:?} key: {key_path:?}");
                let identity =
                    Identity::from_pem(std::fs::read(cert_path)?, std::fs::read(key_path)?);

                info!("Using CA cert: {cert_chain_path:?}");
                let ca_cert = std::fs::read(cert_chain_path)?;

                Some(
                    ServerTlsConfig::new()
                        .identity(identity)
                        .client_ca_root(Certificate::from_pem(ca_cert)),
                )
            }
            (None, None, None) => None,
            (cert, key, chain) => {
                return Err(anyhow!(
                    "Replication TLS is partially configured: cert={}, key={}, cert_chain={}. \
                     Provide all three or none",
                    cert.is_some(),
                    key.is_some(),
                    chain.is_some(),
                ));
            }
        };

        Ok(GrpcReplicationServerBuilder(WantsHttp2Config {
            local_immutable_store: self.0.local_immutable_store,
            tls_config,
        }))
    }
}

pub struct WantsHttp2Config {
    local_immutable_store: Arc<dyn ImmutableStore>,
    tls_config: Option<ServerTlsConfig>,
}

impl GrpcReplicationServerBuilder<WantsHttp2Config> {
    pub fn with_http2_config(
        self,
        http2_keep_alive_interval: Option<Duration>,
        http2_keep_alive_timeout: Option<Duration>,
    ) -> anyhow::Result<GrpcReplicationServerBuilder<WantsAddress>> {
        let metrics_layer = tower::ServiceBuilder::new().layer(GrpcMetricsLayer::new());
        let mut server = Server::builder()
            .http2_keepalive_interval(http2_keep_alive_interval)
            .http2_keepalive_timeout(http2_keep_alive_timeout);

        if let Some(tls_config) = self.0.tls_config {
            server = server.tls_config(tls_config)?;
        }
        let tracing_levels = TraceLayerConfig::default();
        let router = server
            .layer(
                CorrelationIdLayerBuilder::new()
                    .with_grpc_tracer(tracing_levels)
                    .build(),
            )
            .layer(LoreTracingLayer {})
            .layer(metrics_layer)
            .add_service(ReplicationServiceServer::new(LoreReplicationService::new(
                self.0.local_immutable_store,
            )?));

        Ok(GrpcReplicationServerBuilder(WantsAddress { router }))
    }
}

pub struct WantsAddress {
    router: GrpcRouter,
}

impl GrpcReplicationServerBuilder<WantsAddress> {
    pub async fn serve(
        self,
        addr: SocketAddr,
        signal: impl Future<Output = ()>,
    ) -> anyhow::Result<()> {
        self.0.router.serve_with_shutdown(addr, signal).await?;
        Ok(())
    }
}
