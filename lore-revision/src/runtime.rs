// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
pub use lore_base::runtime::LORE_CONTEXT;
pub use lore_base::runtime::runtime;

use crate::interface::ExecutionContext;

/// Get the current `ExecutionContext` from the task-local. Panics if not set.
pub fn execution_context() -> std::sync::Arc<ExecutionContext> {
    lore_base::runtime::lore_context()
        .downcast::<ExecutionContext>()
        .expect("ExecutionContext not set on LORE_CONTEXT")
}

/// Get the current `ExecutionContext` if set, or `None`.
pub fn try_execution_context() -> Option<std::sync::Arc<ExecutionContext>> {
    lore_base::runtime::try_lore_context()?
        .downcast::<ExecutionContext>()
        .ok()
}
