// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Legacy urc.rpc service definitions kept on the server for backward
//! compatibility. The proto files (`storage.proto`, `revision.proto`,
//! `repository.proto`, `environment.proto`) live in `proto/` and are compiled
//! by this crate's `build.rs` into `generated/urc.rpc.rs`. The `urc.model`
//! messages they reference stay in `lore-proto`, redirected via `extern_path`
//! at build time.

#[rustfmt::skip]
#[allow(clippy::doc_markdown)]
#[path = "generated/urc.rpc.rs"]
pub mod rpc;
