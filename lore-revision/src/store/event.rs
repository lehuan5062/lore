// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Per-item event data for the content-addressed storage API.
//!
//! Each put/get/copy/obliterate/query/upload operation terminates an item
//! with an `*_ITEM_COMPLETE` event. `get` additionally emits a `HEADER` and
//! one or more `DATA` events for each item before the terminal. `open`
//! emits a single `OPENED` event on success before `Complete`.
//!
//! All event-data structs here are `#[repr(C)]` PODs carrying the item's
//! correlation `id`, the relevant addresses/partitions, and a
//! [`LoreErrorCode`] discriminator. The companion `LORE_EVENT_ERROR` event
//! (emitted alongside per-item failures) carries the human-readable detail.

use lore_base::types::Address;
use lore_base::types::Context;
use lore_base::types::Fragment;
use lore_base::types::Partition;
use serde::Deserialize;
use serde::Serialize;

use crate::event::LoreBytes;
use crate::event::LoreErrorCode;

/// Delivered on successful `lore_storage_open`. Carries the handle id the
/// caller must pass to subsequent ops against this store.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageOpenedEventData {
    pub handle_id: u64,
}

/// Terminal per-item event for `put` and `put_file`. On success
/// `error_code == None` and `address` is the computed content hash; on
/// failure `error_code` is populated and `address` is zero.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStoragePutItemCompleteEventData {
    pub id: u64,
    pub address: Address,
    pub error_code: LoreErrorCode,
}

/// Leading event for each regular `get` item. Reports the total
/// reassembled content size before any `GET_DATA` events arrive.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageGetHeaderEventData {
    pub id: u64,
    pub address: Address,
    pub size_content: u64,
}

/// Per-fragment (or single-buffer) payload event for `get`. The `bytes`
/// view is valid only during the callback invocation.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageGetDataEventData {
    pub id: u64,
    pub address: Address,
    pub offset: u64,
    pub bytes: LoreBytes,
}

/// Terminal per-item event for `get` and `get_file`. For `get_file` this
/// is emitted without any preceding `HEADER`/`DATA` events — the payload
/// is written directly to the filesystem.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageGetItemCompleteEventData {
    pub id: u64,
    pub address: Address,
    pub error_code: LoreErrorCode,
}

/// Terminal per-item event for `copy`. `source_partition` /
/// `target_partition` disambiguate the per-item source and target. The item's content hash is
/// preserved across the copy so only `source_address` is carried; `target_context` is the
/// destination tuple's context — the destination address is `(target_partition,
/// source_address.hash, target_context)`.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageCopyItemCompleteEventData {
    pub id: u64,
    pub source_partition: Partition,
    pub target_partition: Partition,
    pub source_address: Address,
    pub target_context: Context,
    pub error_code: LoreErrorCode,
}

/// Terminal per-item event for `obliterate`. `local_success` / `remote_success` report
/// whether the corresponding side completed without error. `local_skipped` / `remote_skipped`
/// report whether the corresponding side was suppressed up front by the handle's bound flags
/// (`globals.offline`/`local`/`remote`) — when a side is skipped, its `_success` flag is `0`
/// rather than a misleading `1`. `error_code` is populated if either side that DID run
/// failed.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageObliterateItemCompleteEventData {
    pub id: u64,
    pub address: Address,
    pub local_success: u8,
    pub remote_success: u8,
    pub local_skipped: u8,
    pub remote_skipped: u8,
    pub error_code: LoreErrorCode,
}

/// Terminal per-item event for `get_metadata`. On success `fragment` is
/// valid and `error_code == None`; on miss `error_code == ADDRESS_NOT_FOUND`.
/// Mirrors `LoreStorageGetItemCompleteEventData`'s shape minus the absence of
/// any preceding `GET_HEADER` / `GET_DATA` events — `get_metadata` carries no
/// payload bytes.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageGetMetadataItemCompleteEventData {
    pub id: u64,
    pub address: Address,
    pub fragment: Fragment,
    pub error_code: LoreErrorCode,
}

/// Terminal per-item event for `upload`. `already_durable` is 1 when the
/// item was already flagged durable and no upload was performed.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreStorageUploadItemCompleteEventData {
    pub id: u64,
    pub address: Address,
    pub already_durable: u8,
    pub error_code: LoreErrorCode,
}
