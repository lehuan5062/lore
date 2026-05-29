// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::error;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

/// Guard for cleanly shutting down observability providers on object drop.
///
/// This guard holds references to the various OpenTelemetry providers and
/// the tracing appender worker guard. When dropped, it ensures all providers
/// are properly shut down and any buffered telemetry data is flushed.
#[derive(Debug)]
pub struct TelemetryGuard {
    pub guard: WorkerGuard,
    pub logger_provider: Option<SdkLoggerProvider>,
    pub meter_provider: Option<SdkMeterProvider>,
    pub tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(meter_provider) = &self.meter_provider {
            info!("Shutting down meter provider");
            if let Err(err) = meter_provider.shutdown() {
                error!("Error shutting down meter provider: {err:?}");
            }
        }

        if let Some(tracer_provider) = &self.tracer_provider {
            info!("Shutting down tracer provider");
            if let Err(err) = tracer_provider.shutdown() {
                error!("Error shutting down tracer provider: {err:?}");
            }
        }

        if let Some(logger_provider) = &self.logger_provider {
            info!("Shutting down logger provider");
            if let Err(err) = logger_provider.shutdown() {
                error!("Error shutting down logger provider: {err:?}");
            }
        }
    }
}
