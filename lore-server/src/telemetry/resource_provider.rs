// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Resource detector provider trait for telemetry.
//!
//! This module provides the [`ResourceDetectorProvider`] trait that abstracts
//! environment-specific resource detection for OpenTelemetry. The base server
//! passes `None` when no environment-specific detection is needed, while
//! derived server binaries implement this trait to provide detectors such as
//! cloud infrastructure/deployment resource detection.

use opentelemetry_sdk::resource::ResourceDetector;
use tokio::runtime::Handle;

/// Trait for providing resource detectors to the telemetry system.
///
/// Derived crates implement this to provide environment-specific detectors
/// (e.g., cloud infrastructure/orchestration metadata) that are
/// added to the OpenTelemetry [`Resource`](opentelemetry_sdk::resource::Resource).
///
/// # Example
///
/// ```
/// use lore_server::telemetry::ResourceDetectorProvider;
/// use opentelemetry_sdk::resource::ResourceDetector;
/// use tokio::runtime::Handle;
///
/// struct MyDetectorProvider;
///
/// impl ResourceDetectorProvider for MyDetectorProvider {
///     fn detectors(&self, _runtime_handle: Handle) -> Vec<Box<dyn ResourceDetector>> {
///         vec![]
///     }
/// }
/// ```
pub trait ResourceDetectorProvider: Send + Sync {
    /// Returns a list of resource detectors to add to the telemetry resource.
    ///
    /// # Arguments
    /// * `runtime_handle` - Tokio runtime handle for detectors that perform
    ///   async operations (e.g., querying instance metadata endpoints).
    fn detectors(&self, runtime_handle: Handle) -> Vec<Box<dyn ResourceDetector>>;
}
