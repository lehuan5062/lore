// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Server configuration for derived binaries.
//!
//! This module provides [`ServerConfig`], the configuration struct that derived
//! server binaries pass to [`server_main()`](crate::server::server_main) to
//! inject plugins, hooks, and resource detectors.

use crate::hooks::HookRegistrationContext;
use crate::hooks::HookRegistry;
use crate::plugins::PluginRegistry;
use crate::telemetry::resource_provider::ResourceDetectorProvider;

/// Callback type for hook registration.
///
/// Derived crates provide closures that register their hook factories with the
/// hook registry, using the registration context for runtime dependencies
/// (e.g., notification sender).
pub type HookRegistrationCallback =
    Box<dyn FnOnce(&mut HookRegistry, &HookRegistrationContext) + Send>;

/// Configuration passed to [`server_main()`](crate::server::server_main) by
/// derived binaries.
///
/// This struct allows derived crates to inject plugins, hooks, and resource
/// detectors without the server library needing compile-time knowledge of them.
///
/// # Example
///
/// ```
/// use lore_server::server_config::ServerConfig;
///
/// // Base server with no plugins
/// let config = ServerConfig::default();
/// ```
pub struct ServerConfig {
    /// Pre-populated plugin registry with all desired plugins registered.
    pub plugin_registry: PluginRegistry,

    /// Callbacks for registering hooks. Called after the notification system is
    /// initialized so that [`HookRegistrationContext`] has a valid notification
    /// sender.
    pub hook_registration_callbacks: Vec<HookRegistrationCallback>,

    /// Optional provider for telemetry resource detectors (e.g., AWS, Nomad).
    /// When `None`, no environment-specific resource detectors are added.
    pub resource_detector_provider: Option<Box<dyn ResourceDetectorProvider>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
            hook_registration_callbacks: vec![],
            resource_detector_provider: None,
        }
    }
}
