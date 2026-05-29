// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use lore_revision::interface::LoreGlobalArgs;

use crate::args::InvokableLoreArgs;
use crate::interface::LoreEventCallback;
use crate::interface::LoreEventCallbackConfig;
use crate::remote::call::service_call;

pub(crate) fn run_synchronously<
    ArgsType: InvokableLoreArgs + Clone + Send + 'static,
    Handler: Fn(LoreGlobalArgs, ArgsType, LoreEventCallback) -> Fut,
    Fut: Future<Output = i32> + Send + 'static,
>(
    globals: &LoreGlobalArgs,
    args: &ArgsType,
    callback: LoreEventCallbackConfig,
    handler: Handler,
) -> i32 {
    let callback = lore_revision::event::convert_event_callback(callback);
    let globals = globals.clone();
    let args = args.clone();
    crate::runtime().block_on(handler(globals, args, callback))
}

pub(crate) fn run_asynchronously<
    ArgsType: InvokableLoreArgs + Clone + Send + 'static,
    Handler: Fn(LoreGlobalArgs, ArgsType, LoreEventCallback) -> Fut,
    Fut: Future<Output = i32> + Send + 'static,
>(
    globals: &LoreGlobalArgs,
    args: &ArgsType,
    callback: LoreEventCallbackConfig,
    handler: Handler,
) {
    let callback = lore_revision::event::convert_event_callback(callback);
    let globals = globals.clone();
    let args = args.clone();
    drop(lore_base::lore_spawn!(handler(globals, args, callback)));
}

pub(crate) async fn dispatch_call<
    ArgsType: InvokableLoreArgs + Clone + Send + 'static,
    Handler: Fn(LoreGlobalArgs, ArgsType, LoreEventCallback) -> Fut,
    Fut: Future<Output = i32> + Send + 'static,
>(
    globals: LoreGlobalArgs,
    args: ArgsType,
    callback: LoreEventCallback,
    handler: Handler,
) -> i32 {
    if let Ok(environment_value) = std::env::var("LORE_USE_SERVICE")
        && !environment_value.is_empty()
    {
        service_call(globals, args, callback).await
    } else {
        handler(globals, args, callback).await
    }
}
