// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::time::Duration;

use lore_error_set::prelude::*;
use lore_telemetry::ExporterConfig;
use lore_telemetry::MetricsConfig;
use lore_telemetry::TelemetryError;
use lore_telemetry::set_meter_provider;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use tokio::runtime::Handle;
use tracing::debug;

use super::resource::resource;
use super::resource_provider::ResourceDetectorProvider;

/// Initializes an OTLP meter provider for exporting metrics.
///
/// Creates a meter provider configured with:
/// - OTLP exporter using gRPC/tonic
/// - Periodic reader with configurable export interval
/// - Resource attributes from the environment and configuration
///
/// Also sets this provider as the global meter provider via `set_meter_provider`.
pub fn init_meter_provider(
    exporter_config: &ExporterConfig,
    metrics_config: &MetricsConfig,
    additional_labels: &Option<HashMap<String, String>>,
    runtime_handle: Handle,
    resource_detector_provider: Option<&dyn ResourceDetectorProvider>,
) -> Result<SdkMeterProvider, TelemetryError> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(exporter_config.endpoint.clone())
        .with_timeout(Duration::from_millis(exporter_config.timeout))
        .build()
        .internal("Failed to build OTLP metric exporter")?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_millis(metrics_config.sample_interval_millis))
        .build();

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource(
            additional_labels,
            runtime_handle,
            resource_detector_provider,
        ))
        .with_reader(reader)
        .build();

    debug!("Setting meter provider");
    set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}
