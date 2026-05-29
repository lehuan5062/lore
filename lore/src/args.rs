// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use enum_dispatch::enum_dispatch;
use lore_revision::interface::LoreGlobalArgs;

use crate::interface::LoreEventCallback;
use crate::remote::command::LoreCommand;

#[enum_dispatch]
pub trait LoreArgs {
    fn to_command(self) -> LoreCommand;
}

// Keep this method separate from the LoreArgs trait because the anonymous return type doesn't work with enum_dispatch.
pub(crate) trait InvokableLoreArgs: LoreArgs {
    // Calls the local implementation of the functionality associated with this arg type
    fn invoke_local(
        self,
        globals: LoreGlobalArgs,
        callback: LoreEventCallback,
    ) -> impl Future<Output = i32> + Send;
}
