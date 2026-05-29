// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#![allow(non_camel_case_types)]
#![allow(unused_parens)]

pub mod metadata;
pub mod revision_tree;

use lore_macro::VariantTypeSize;
use serde::Deserialize;
use serde::Serialize;

use crate::auth::LoreAuthUrlEventData;
use crate::auth::userinfo::LoreAuthIdentityEventData;
use crate::auth::userinfo::LoreAuthUserInfoEventData;
use crate::auth::userinfo::LoreAuthUserTokenEventData;
use crate::branch::LoreBranchCreateEventData;
use crate::branch::LoreBranchDeleteEventData;
use crate::branch::LoreBranchDiffBeginEventData;
use crate::branch::LoreBranchDiffChangeBeginEventData;
use crate::branch::LoreBranchDiffChangeEndEventData;
use crate::branch::LoreBranchDiffChangeEventData;
use crate::branch::LoreBranchDiffConflictBeginEventData;
use crate::branch::LoreBranchDiffConflictEndEventData;
use crate::branch::LoreBranchDiffConflictEventData;
use crate::branch::LoreBranchDiffEndEventData;
use crate::branch::LoreBranchListBeginEventData;
use crate::branch::LoreBranchListEndEventData;
use crate::branch::LoreBranchListEntryEventData;
use crate::branch::LoreBranchProtectEventData;
use crate::branch::LoreBranchUnprotectEventData;
use crate::branch::info::LoreBranchInfoEventData;
use crate::branch::latest::LoreBranchLatestListEntryEventData;
use crate::branch::merge::LoreBranchMergeAbortBeginEventData;
use crate::branch::merge::LoreBranchMergeAbortEndEventData;
use crate::branch::merge::LoreBranchMergeConflictFileEventData;
use crate::branch::merge::LoreBranchMergeIntoFileBeginEventData;
use crate::branch::merge::LoreBranchMergeIntoFileEndEventData;
use crate::branch::merge::LoreBranchMergeIntoFileEventData;
use crate::branch::merge::LoreBranchMergeIntoFragmentBeginEventData;
use crate::branch::merge::LoreBranchMergeIntoFragmentEndEventData;
use crate::branch::merge::LoreBranchMergeIntoFragmentProgressEventData;
use crate::branch::merge::LoreBranchMergeIntoRevisionEventData;
use crate::branch::merge::LoreBranchMergeIntoSyncBeginEventData;
use crate::branch::merge::LoreBranchMergeIntoSyncEndEventData;
use crate::branch::merge::LoreBranchMergeResolveFileEventData;
use crate::branch::merge::LoreBranchMergeResolveRevisionEventData;
use crate::branch::merge::LoreBranchMergeStartBeginEventData;
use crate::branch::merge::LoreBranchMergeStartEndEventData;
use crate::branch::merge::LoreBranchMergeUnresolveFileEventData;
use crate::branch::merge::LoreBranchMergeUnresolveRevisionEventData;
use crate::branch::push::LoreBranchPushBranchCreateBeginEventData;
use crate::branch::push::LoreBranchPushBranchCreateEndEventData;
use crate::branch::push::LoreBranchPushEventData;
use crate::branch::push::LoreBranchPushFragmentBeginEventData;
use crate::branch::push::LoreBranchPushFragmentEndEventData;
use crate::branch::push::LoreBranchPushFragmentProgressEventData;
use crate::branch::push::LoreBranchPushRevisionPushBeginEventData;
use crate::branch::push::LoreBranchPushRevisionPushEndEventData;
use crate::branch::push::LoreBranchPushRevisionPushUpdateEventData;
use crate::branch::push::LoreBranchPushRevisionUpdateBeginEventData;
use crate::branch::push::LoreBranchPushRevisionUpdateEndEventData;
use crate::branch::reset::LoreBranchResetEventData;
use crate::commit::LoreRevisionCommitBeginEventData;
use crate::commit::LoreRevisionCommitEndEventData;
use crate::commit::LoreRevisionCommitProgressEventData;
use crate::commit::LoreRevisionCommitRevisionEventData;
use crate::dependency::LoreDependencyResolveBeginEventData;
use crate::dependency::LoreDependencyResolveEndEventData;
use crate::dependency::LoreDependencyResolveItemEventData;
use crate::dependency::LoreFileDependencyAddBeginEventData;
use crate::dependency::LoreFileDependencyAddEndEventData;
use crate::dependency::LoreFileDependencyAddEntryEventData;
use crate::dependency::LoreFileDependencyListBeginEventData;
use crate::dependency::LoreFileDependencyListEndEventData;
use crate::dependency::LoreFileDependencyListEntryEventData;
use crate::dependency::LoreFileDependencyListFileEndEventData;
use crate::dependency::LoreFileDependencyListFileEventData;
use crate::dependency::LoreFileDependencyRemoveBeginEventData;
use crate::dependency::LoreFileDependencyRemoveEndEventData;
use crate::dependency::LoreFileDependencyRemoveEntryEventData;
use crate::event::revision_tree::LoreRevisionTreeAddCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeChildEventData;
use crate::event::revision_tree::LoreRevisionTreeCloseCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeCommitCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeDeleteCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeLoadedEventData;
use crate::event::revision_tree::LoreRevisionTreeMetadataGetCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeMetadataSetCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeModifyCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeMoveCompleteEventData;
use crate::event::revision_tree::LoreRevisionTreeNodeInfoEventData;
use crate::event::revision_tree::LoreRevisionTreeNodePathEventData;
use crate::event::revision_tree::LoreRevisionTreeResolvePathCompleteEventData;
use crate::file::diff::LoreFileDiffEventData;
use crate::file::dump::LoreFileDumpEventData;
use crate::file::hash::LoreFileHashEventData;
use crate::file::history::LoreFileHistoryEventData;
use crate::file::info::LoreFileInfoEventData;
use crate::file::obliterate::LoreFileObliterateEventData;
use crate::file::reset::LoreFileResetBeginEventData;
use crate::file::reset::LoreFileResetEndEventData;
use crate::file::reset::LoreFileResetFileEventData;
use crate::file::reset::LoreFileResetProgressEventData;
use crate::file::unstage::LoreFileUnstageBeginEventData;
use crate::file::unstage::LoreFileUnstageEndEventData;
use crate::file::unstage::LoreFileUnstageFileEventData;
use crate::file::unstage::LoreFileUnstageProgressEventData;
use crate::file::unstage::LoreFileUnstageRevisionEventData;
use crate::file::write::LoreFileWriteEventData;
use crate::filter::LoreFilterExcludeEventData;
use crate::find::LoreRevisionFindEventData;
use crate::immutable::LoreFragmentWriteEventData;
use crate::instance::LoreBranchMultipleInstanceEventData;
use crate::instance::LoreRepositoryInstanceEventData;
use crate::interface::LoreError;
use crate::interface::LoreEventCallback;
use crate::interface::LoreEventCallbackConfig;
use crate::interface::LoreMetadata;
use crate::interface::LoreString;
use crate::layer::LoreLayerAddEventData;
use crate::layer::LoreLayerEntryEventData;
use crate::layer::LoreLayerRemoveEventData;
use crate::layer::LoreLayerStagedEntryEventData;
use crate::link::LoreLinkChangeEventData;
use crate::link::LoreLinkEntryEventData;
use crate::link::list::LoreLinkStagedEntryEventData;
use crate::lock::file::acquire::LoreLockFileAcquireEventData;
use crate::lock::file::acquire::LoreLockFileAcquireIgnoreEventData;
use crate::lock::file::query::LoreLockFileQueryBeginEventData;
use crate::lock::file::query::LoreLockFileQueryEventData;
use crate::lock::file::release::LoreLockFileReleaseEventData;
use crate::lock::file::release::LoreLockFileReleaseNotFoundEventData;
use crate::lock::file::status::LoreLockFileStatusBeginEventData;
use crate::lock::file::status::LoreLockFileStatusEventData;
use crate::lore::execution_context;
use crate::metadata::Metadata;
use crate::metadata::MetadataError;
use crate::metadata::MetadataType;
use crate::metadata::clear::LoreMetadataClearFileEventData;
use crate::metadata::clear::LoreMetadataClearRevisionEventData;
use crate::notification::LoreNotificationBranchCreatedEventData;
use crate::notification::LoreNotificationBranchDeletedEventData;
use crate::notification::LoreNotificationBranchPushedEventData;
use crate::notification::LoreNotificationResourceLockedEventData;
use crate::notification::LoreNotificationResourceUnlockedEventData;
use crate::notification::LoreNotificationSubscribedEventData;
use crate::notification::LoreNotificationUnsubscribedEventData;
use crate::path::LorePathIgnoreEventData;
use crate::repository::LoreBranchSwitchBeginEventData;
use crate::repository::LoreBranchSwitchEndEventData;
use crate::repository::LoreRepositoryConfigGetEventData;
use crate::repository::LoreRepositoryDumpBeginEventData;
use crate::repository::LoreRepositoryDumpEndEventData;
use crate::repository::clone::LoreRepositoryCloneBeginEventData;
use crate::repository::clone::LoreRepositoryCloneEndEventData;
use crate::repository::clone::LoreRepositoryCloneProgressEventData;
use crate::repository::create::LoreRepositoryCreateEventData;
use crate::repository::info::LoreRepositoryDataEventData;
use crate::repository::list::LoreRepositoryListEntryEventData;
use crate::repository::status::LoreRepositoryStatusFileEventData;
use crate::repository::status::LoreRepositoryStatusRevisionEventData;
use crate::repository::store::LoreRepositoryStoreImmutableQueryEventData;
use crate::repository::verify::LoreRepositoryVerifyFragmentEventData;
use crate::repository::verify::LoreRepositoryVerifyFragmentMatchEventData;
use crate::repository::verify::LoreRepositoryVerifyFragmentRemoteEventData;
use crate::repository::verify::LoreRepositoryVerifyStateBeginEventData;
use crate::repository::verify::LoreRepositoryVerifyStateEndEventData;
use crate::revision::LoreRevisionResolveEventData;
use crate::revision::bisect::LoreRevisionBisectEventData;
use crate::revision::cherry_pick::LoreCherryPickAbortBeginEventData;
use crate::revision::cherry_pick::LoreCherryPickAbortEndEventData;
use crate::revision::cherry_pick::LoreCherryPickConflictFileEventData;
use crate::revision::cherry_pick::LoreCherryPickResolveFileEventData;
use crate::revision::cherry_pick::LoreCherryPickResolveRevisionEventData;
use crate::revision::cherry_pick::LoreCherryPickStartBeginEventData;
use crate::revision::cherry_pick::LoreCherryPickStartEndEventData;
use crate::revision::cherry_pick::LoreCherryPickUnresolveFileEventData;
use crate::revision::cherry_pick::LoreCherryPickUnresolveRevisionEventData;
use crate::revision::diff::LoreRevisionDiffFileEventData;
use crate::revision::history::LoreRevisionHistoryEntryEventData;
use crate::revision::history::LoreRevisionHistoryEventData;
use crate::revision::info::LoreRevisionInfoDeltaEventData;
use crate::revision::info::LoreRevisionInfoEventData;
use crate::revision::restore::LoreRevisionRestoreFileBeginEventData;
use crate::revision::restore::LoreRevisionRestoreFileEndEventData;
use crate::revision::restore::LoreRevisionRestoreFileEventData;
use crate::revision::restore::LoreRevisionRestoreFragmentBeginEventData;
use crate::revision::restore::LoreRevisionRestoreFragmentEndEventData;
use crate::revision::restore::LoreRevisionRestoreFragmentProgressEventData;
use crate::revision::restore::LoreRevisionRestoreRevisionEventData;
use crate::revision::restore::LoreRevisionRestoreSyncBeginEventData;
use crate::revision::restore::LoreRevisionRestoreSyncEndEventData;
use crate::revision::revert::LoreRevertAbortBeginEventData;
use crate::revision::revert::LoreRevertAbortEndEventData;
use crate::revision::revert::LoreRevertConflictFileEventData;
use crate::revision::revert::LoreRevertResolveFileEventData;
use crate::revision::revert::LoreRevertResolveRevisionEventData;
use crate::revision::revert::LoreRevertStartBeginEventData;
use crate::revision::revert::LoreRevertStartEndEventData;
use crate::revision::revert::LoreRevertUnresolveFileEventData;
use crate::revision::revert::LoreRevertUnresolveRevisionEventData;
use crate::revision::sync::LoreRevisionSyncFileEventData;
use crate::revision::sync::LoreRevisionSyncProgressEventData;
use crate::revision::sync::LoreRevisionSyncRevisionEventData;
use crate::revision::sync::LoreRevisionSyncTargetEventData;
use crate::shared_store::LoreSharedStoreCreateEventData;
use crate::shared_store::LoreSharedStoreInfoEventData;
use crate::stage::LoreFileStageBeginEventData;
use crate::stage::LoreFileStageEndEventData;
use crate::stage::LoreFileStageFileEventData;
use crate::stage::LoreFileStageProgressEventData;
use crate::stage::LoreFileStageRevisionEventData;
use crate::state::LoreRepositoryStateDumpEventData;
use crate::state::LoreRepositoryStateDumpNodeEventData;
use crate::store::event::LoreStorageCopyItemCompleteEventData;
use crate::store::event::LoreStorageGetDataEventData;
use crate::store::event::LoreStorageGetHeaderEventData;
use crate::store::event::LoreStorageGetItemCompleteEventData;
use crate::store::event::LoreStorageGetMetadataItemCompleteEventData;
use crate::store::event::LoreStorageObliterateItemCompleteEventData;
use crate::store::event::LoreStorageOpenedEventData;
use crate::store::event::LoreStoragePutItemCompleteEventData;
use crate::store::event::LoreStorageUploadItemCompleteEventData;

pub fn convert_event_callback(callback: LoreEventCallbackConfig) -> LoreEventCallback {
    if let Some(func) = callback.func {
        Some(Box::new(move |event: &LoreEvent| unsafe {
            func(event, callback.user_context);
        }))
    } else {
        None
    }
}

pub trait EventError: std::fmt::Display {
    // The error to expose to the user. Defaults to `LoreError::Internal` —
    // the right answer for any error_set whose handleable variants are all
    // mapped to opaque internal events; override for sets that surface
    // user-actionable variants like `LoreError::NotFound`.
    fn translated(&self) -> LoreError {
        LoreError::Internal
    }

    // The underlying error message as generated by URC library
    fn inner(&self) -> String {
        self.to_string()
    }
}

// TODO(vri): Implement with a union to enable command-specific progress events
#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreProgressEventData {
    pub _unused: u32,
}

/// Borrowed byte slice handed to callbacks.
///
/// The pointer is valid only for the duration of the callback that receives
/// it; callers must copy the bytes if they need them beyond that scope.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LoreBytes {
    pub ptr: *const core::ffi::c_void,
    pub len: usize,
}

// SAFETY: `LoreBytes` is a borrowed view; the referenced bytes live in a
// buffer owned by the emitter whose lifetime contract is "valid for the
// duration of the callback". Passing a view between threads within that
// lifetime is sound — matches the equivalent contract on `LoreString`.
unsafe impl Send for LoreBytes {}
unsafe impl Sync for LoreBytes {}

impl LoreBytes {
    /// View the referenced bytes as a Rust slice.
    ///
    /// # Safety
    ///
    /// Caller must ensure the emitter's lifetime contract is still upheld
    /// at the call — i.e., the view was just received in a callback and
    /// has not outlived it. A zero-length or null view is always safe.
    pub unsafe fn as_slice(&self) -> &[u8] {
        if self.ptr.is_null() || self.len == 0 {
            &[]
        } else {
            // SAFETY: upheld by the caller's invocation precondition.
            unsafe { core::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
        }
    }
}

impl PartialEq for LoreBytes {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: `PartialEq` is only meaningfully called by the emitter
        // within the view's lifetime (e.g., event comparisons inside the
        // dispatcher). Zero-length / null is handled by `as_slice`.
        unsafe { self.as_slice() == other.as_slice() }
    }
}

impl serde::Serialize for LoreBytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // SAFETY: `Serialize` is driven by the callback path while the
        // view is still live.
        serializer.serialize_bytes(unsafe { self.as_slice() })
    }
}

impl<'de> serde::Deserialize<'de> for LoreBytes {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(
            "LoreBytes cannot be deserialized — it is a borrowed view",
        ))
    }
}

/// Small discriminator enum for per-item terminal events in the
/// content-addressed storage API.
///
/// Narrower than [`crate::interface::LoreError`] — events emitted per
/// put/get/copy/etc. item embed this code so a caller can branch on the
/// common cases cheaply without parsing the companion `LORE_EVENT_ERROR`
/// detail. Variants overlap with [`crate::interface::LoreError`] where they
/// share a meaning.
///
/// cbindgen:prefix-with-name
/// cbindgen:rename-all=ScreamingSnakeCase
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LoreErrorCode {
    None = 0,
    InvalidArguments = 1,
    AddressNotFound = 2,
    Internal = 3,
    SlowDown = 4,
}

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreErrorEventData {
    pub error_type: u32,
    pub error_inner: LoreString,
}

impl LoreErrorEventData {
    pub fn from_inner_error(err: &impl EventError) -> Self {
        Self {
            error_type: err.translated() as u32,
            error_inner: LoreString::from(err.inner()),
        }
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreCompleteEventData {
    pub status: i32,
}

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreMetadataEventData {
    pub key: LoreString,
    pub value: LoreMetadata,
}

impl LoreMetadataEventData {
    pub fn new(key: &str, value: &[u8], value_type: MetadataType) -> Result<Self, MetadataError> {
        let key = LoreString::from(key);
        let value = match value_type {
            MetadataType::Address => LoreMetadata::Address(Metadata::to_address(value)?),
            MetadataType::Boolean => LoreMetadata::Boolean(Metadata::to_bool(value)? as u8),
            MetadataType::Context => LoreMetadata::Context(Metadata::to_context(value)?),
            MetadataType::Hash => LoreMetadata::Hash(Metadata::to_hash(value)?),
            MetadataType::Numeric => LoreMetadata::Numeric(Metadata::to_u64(value)?),
            MetadataType::String => {
                LoreMetadata::String(LoreString::from(Metadata::to_string(value).ok()))
            }
            MetadataType::Binary => return Err(MetadataError::internal("metadata type mismatch")),
        };

        Ok(LoreMetadataEventData { key, value })
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreLogEventData {
    pub level: lore_base::log::LoreLogLevel,
    pub category: u32,
    pub timestamp: u64,
    pub location: LoreString,
    pub message: LoreString,
}

#[repr(C)]
#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreEndEventData {
    pub unused: u32,
}

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreMaintenanceEventData {
    pub message: LoreString,
}

/// cbindgen:prefix-with-name
/// cbindgen:rename-all=ScreamingSnakeCase
#[repr(C, u32)]
#[derive(Clone, PartialEq, Serialize, Deserialize, VariantTypeSize)]
#[serde(tag = "tagName", content = "data", rename_all = "camelCase")]
pub enum LoreEvent {
    // Standard events
    Progress(LoreProgressEventData),
    Error(LoreErrorEventData),
    Complete(LoreCompleteEventData),
    Metadata(LoreMetadataEventData),
    Log(LoreLogEventData),
    End(LoreEndEventData),
    Maintenance(LoreMaintenanceEventData),
    // ... Specialized events
    AuthUrl(LoreAuthUrlEventData),
    AuthUserInfo(LoreAuthUserInfoEventData),
    AuthUserToken(LoreAuthUserTokenEventData),
    AuthIdentity(LoreAuthIdentityEventData),
    BranchCreate(LoreBranchCreateEventData),
    BranchMultipleInstance(LoreBranchMultipleInstanceEventData),
    BranchDelete(LoreBranchDeleteEventData),
    BranchListBegin(LoreBranchListBeginEventData),
    BranchListEntry(LoreBranchListEntryEventData),
    BranchListEnd(LoreBranchListEndEventData),
    BranchMergeAbortBegin(LoreBranchMergeAbortBeginEventData),
    BranchMergeAbortEnd(LoreBranchMergeAbortEndEventData),
    BranchInfo(LoreBranchInfoEventData),
    BranchDiffBegin(LoreBranchDiffBeginEventData),
    BranchDiffChangeBegin(LoreBranchDiffChangeBeginEventData),
    BranchDiffChange(LoreBranchDiffChangeEventData),
    BranchDiffChangeEnd(LoreBranchDiffChangeEndEventData),
    BranchDiffConflictBegin(LoreBranchDiffConflictBeginEventData),
    BranchDiffConflict(LoreBranchDiffConflictEventData),
    BranchDiffConflictEnd(LoreBranchDiffConflictEndEventData),
    BranchDiffEnd(LoreBranchDiffEndEventData),
    BranchLatestListEntry(LoreBranchLatestListEntryEventData),
    BranchMergeConflictFile(LoreBranchMergeConflictFileEventData),
    BranchMergeLinkSkipped(crate::branch::merge::LoreBranchMergeLinkSkippedEventData),
    BranchMergeUnresolveFile(LoreBranchMergeUnresolveFileEventData),
    BranchMergeUnresolveRevision(LoreBranchMergeUnresolveRevisionEventData),
    BranchMergeIntoFileBegin(LoreBranchMergeIntoFileBeginEventData),
    BranchMergeIntoFile(LoreBranchMergeIntoFileEventData),
    BranchMergeIntoFileEnd(LoreBranchMergeIntoFileEndEventData),
    BranchMergeIntoFragmentBegin(LoreBranchMergeIntoFragmentBeginEventData),
    BranchMergeIntoFragmentProgress(LoreBranchMergeIntoFragmentProgressEventData),
    BranchMergeIntoFragmentEnd(LoreBranchMergeIntoFragmentEndEventData),
    BranchMergeIntoRevision(LoreBranchMergeIntoRevisionEventData),
    BranchMergeIntoSyncBegin(LoreBranchMergeIntoSyncBeginEventData),
    BranchMergeIntoSyncEnd(LoreBranchMergeIntoSyncEndEventData),
    BranchMergeResolveFile(LoreBranchMergeResolveFileEventData),
    BranchMergeResolveRevision(LoreBranchMergeResolveRevisionEventData),
    BranchMergeStartBegin(LoreBranchMergeStartBeginEventData),
    BranchMergeStartEnd(LoreBranchMergeStartEndEventData),
    CherryPickStartBegin(LoreCherryPickStartBeginEventData),
    CherryPickStartEnd(LoreCherryPickStartEndEventData),
    CherryPickAbortBegin(LoreCherryPickAbortBeginEventData),
    CherryPickAbortEnd(LoreCherryPickAbortEndEventData),
    CherryPickConflictFile(LoreCherryPickConflictFileEventData),
    CherryPickUnresolveFile(LoreCherryPickUnresolveFileEventData),
    CherryPickUnresolveRevision(LoreCherryPickUnresolveRevisionEventData),
    CherryPickResolveFile(LoreCherryPickResolveFileEventData),
    CherryPickResolveRevision(LoreCherryPickResolveRevisionEventData),
    RevertStartBegin(LoreRevertStartBeginEventData),
    RevertStartEnd(LoreRevertStartEndEventData),
    RevertAbortBegin(LoreRevertAbortBeginEventData),
    RevertAbortEnd(LoreRevertAbortEndEventData),
    RevertResolveFile(LoreRevertResolveFileEventData),
    RevertResolveRevision(LoreRevertResolveRevisionEventData),
    RevertConflictFile(LoreRevertConflictFileEventData),
    RevertUnresolveFile(LoreRevertUnresolveFileEventData),
    RevertUnresolveRevision(LoreRevertUnresolveRevisionEventData),
    BranchProtect(LoreBranchProtectEventData),
    BranchPush(LoreBranchPushEventData),
    BranchPushRevisionUpdateBegin(LoreBranchPushRevisionUpdateBeginEventData),
    BranchPushRevisionUpdateEnd(LoreBranchPushRevisionUpdateEndEventData),
    BranchPushFragmentBegin(LoreBranchPushFragmentBeginEventData),
    BranchPushFragmentProgress(LoreBranchPushFragmentProgressEventData),
    BranchPushFragmentEnd(LoreBranchPushFragmentEndEventData),
    BranchPushBranchCreateBegin(LoreBranchPushBranchCreateBeginEventData),
    BranchPushBranchCreateEnd(LoreBranchPushBranchCreateEndEventData),
    BranchPushRevisionPushBegin(LoreBranchPushRevisionPushBeginEventData),
    BranchPushRevisionPushUpdate(LoreBranchPushRevisionPushUpdateEventData),
    BranchPushRevisionPushEnd(LoreBranchPushRevisionPushEndEventData),
    BranchReset(LoreBranchResetEventData),
    BranchSwitchBegin(LoreBranchSwitchBeginEventData),
    BranchSwitchEnd(LoreBranchSwitchEndEventData),
    BranchUnprotect(LoreBranchUnprotectEventData),
    FileInfo(LoreFileInfoEventData),
    FileDiff(LoreFileDiffEventData),
    FileHash(LoreFileHashEventData),
    FileHistory(LoreFileHistoryEventData),
    FileWrite(LoreFileWriteEventData),
    FileObliterate(LoreFileObliterateEventData),
    FileDump(LoreFileDumpEventData),
    FileDependencyAddBegin(LoreFileDependencyAddBeginEventData),
    FileDependencyAddEntry(LoreFileDependencyAddEntryEventData),
    FileDependencyAddEnd(LoreFileDependencyAddEndEventData),
    FileDependencyRemoveBegin(LoreFileDependencyRemoveBeginEventData),
    FileDependencyRemoveEntry(LoreFileDependencyRemoveEntryEventData),
    FileDependencyRemoveEnd(LoreFileDependencyRemoveEndEventData),
    FileDependencyListBegin(LoreFileDependencyListBeginEventData),
    FileDependencyListFile(LoreFileDependencyListFileEventData),
    FileDependencyListEntry(LoreFileDependencyListEntryEventData),
    FileDependencyListFileEnd(LoreFileDependencyListFileEndEventData),
    FileDependencyListEnd(LoreFileDependencyListEndEventData),
    FileResetBegin(LoreFileResetBeginEventData),
    FileResetProgress(LoreFileResetProgressEventData),
    FileResetEnd(LoreFileResetEndEventData),
    FileResetFile(LoreFileResetFileEventData),
    FilterExclude(LoreFilterExcludeEventData),
    FileStageBegin(LoreFileStageBeginEventData),
    FileStageProgress(LoreFileStageProgressEventData),
    FileStageEnd(LoreFileStageEndEventData),
    FileStageRevision(LoreFileStageRevisionEventData),
    FileStageFile(LoreFileStageFileEventData),
    FileUnstageBegin(LoreFileUnstageBeginEventData),
    FileUnstageProgress(LoreFileUnstageProgressEventData),
    FileUnstageEnd(LoreFileUnstageEndEventData),
    FileUnstageRevision(LoreFileUnstageRevisionEventData),
    FileUnstageFile(LoreFileUnstageFileEventData),
    FragmentWrite(LoreFragmentWriteEventData),
    LayerAdd(LoreLayerAddEventData),
    LayerEntry(LoreLayerEntryEventData),
    LayerRemove(LoreLayerRemoveEventData),
    LayerStagedEntry(LoreLayerStagedEntryEventData),
    LinkChange(LoreLinkChangeEventData),
    LinkEntry(LoreLinkEntryEventData),
    LockFileAcquire(LoreLockFileAcquireEventData),
    LockFileAcquireIgnore(LoreLockFileAcquireIgnoreEventData),
    LockFileStatusBegin(LoreLockFileStatusBeginEventData),
    LockFileStatus(LoreLockFileStatusEventData),
    LockFileQueryBegin(LoreLockFileQueryBeginEventData),
    LockFileQuery(LoreLockFileQueryEventData),
    LockFileRelease(LoreLockFileReleaseEventData),
    LockFileReleaseNotFound(LoreLockFileReleaseNotFoundEventData),
    MetadataClearFile(LoreMetadataClearFileEventData),
    MetadataClearRevision(LoreMetadataClearRevisionEventData),
    PathIgnore(LorePathIgnoreEventData),
    RepositoryCreate(LoreRepositoryCreateEventData),
    RepositoryCloneBegin(LoreRepositoryCloneBeginEventData),
    RepositoryCloneProgress(LoreRepositoryCloneProgressEventData),
    RepositoryCloneEnd(LoreRepositoryCloneEndEventData),
    DependencyResolveBegin(LoreDependencyResolveBeginEventData),
    DependencyResolveItem(LoreDependencyResolveItemEventData),
    DependencyResolveEnd(LoreDependencyResolveEndEventData),
    RepositoryData(LoreRepositoryDataEventData),
    RepositoryConfigGet(LoreRepositoryConfigGetEventData),
    RepositoryDumpBegin(LoreRepositoryDumpBeginEventData),
    RepositoryDumpEnd(LoreRepositoryDumpEndEventData),
    RepositoryListEntry(LoreRepositoryListEntryEventData),
    RepositoryInstance(LoreRepositoryInstanceEventData),
    RepositoryVerifyStateBegin(LoreRepositoryVerifyStateBeginEventData),
    RepositoryVerifyStateEnd(LoreRepositoryVerifyStateEndEventData),
    RepositoryVerifyFragment(LoreRepositoryVerifyFragmentEventData),
    RepositoryVerifyFragmentMatch(LoreRepositoryVerifyFragmentMatchEventData),
    RepositoryVerifyFragmentRemote(LoreRepositoryVerifyFragmentRemoteEventData),
    RepositoryStateDump(LoreRepositoryStateDumpEventData),
    RepositoryStateDumpNode(LoreRepositoryStateDumpNodeEventData),
    RepositoryStatusRevision(LoreRepositoryStatusRevisionEventData),
    RepositoryStatusFile(LoreRepositoryStatusFileEventData),
    RepositoryStoreImmutableQuery(LoreRepositoryStoreImmutableQueryEventData),
    RevisionCommitBegin(LoreRevisionCommitBeginEventData),
    RevisionCommitProgress(LoreRevisionCommitProgressEventData),
    RevisionCommitEnd(LoreRevisionCommitEndEventData),
    RevisionCommitRevision(LoreRevisionCommitRevisionEventData),
    RevisionInfo(LoreRevisionInfoEventData),
    RevisionInfoDelta(LoreRevisionInfoDeltaEventData),
    RevisionDiffFile(LoreRevisionDiffFileEventData),
    RevisionFind(LoreRevisionFindEventData),
    RevisionHistory(LoreRevisionHistoryEventData),
    RevisionHistoryEntry(LoreRevisionHistoryEntryEventData),
    RevisionRestoreFileBegin(LoreRevisionRestoreFileBeginEventData),
    RevisionRestoreFile(LoreRevisionRestoreFileEventData),
    RevisionRestoreFileEnd(LoreRevisionRestoreFileEndEventData),
    RevisionRestoreFragmentBegin(LoreRevisionRestoreFragmentBeginEventData),
    RevisionRestoreFragmentProgress(LoreRevisionRestoreFragmentProgressEventData),
    RevisionRestoreFragmentEnd(LoreRevisionRestoreFragmentEndEventData),
    RevisionRestoreRevision(LoreRevisionRestoreRevisionEventData),
    RevisionRestoreSyncBegin(LoreRevisionRestoreSyncBeginEventData),
    RevisionRestoreSyncEnd(LoreRevisionRestoreSyncEndEventData),
    RevisionResolve(LoreRevisionResolveEventData),
    RevisionSyncTarget(LoreRevisionSyncTargetEventData),
    RevisionSyncFile(LoreRevisionSyncFileEventData),
    RevisionSyncProgress(LoreRevisionSyncProgressEventData),
    RevisionSyncRevision(LoreRevisionSyncRevisionEventData),
    RevisionBisect(LoreRevisionBisectEventData),
    NotificationBranchCreated(LoreNotificationBranchCreatedEventData),
    NotificationBranchDeleted(LoreNotificationBranchDeletedEventData),
    NotificationBranchPushed(LoreNotificationBranchPushedEventData),
    NotificationResourceLocked(LoreNotificationResourceLockedEventData),
    NotificationResourceUnlocked(LoreNotificationResourceUnlockedEventData),
    NotificationSubscribed(LoreNotificationSubscribedEventData),
    NotificationUnsubscribed(LoreNotificationUnsubscribedEventData),
    SharedStoreCreate(LoreSharedStoreCreateEventData),
    SharedStoreInfo(LoreSharedStoreInfoEventData),
    LinkStagedEntry(LoreLinkStagedEntryEventData),
    // Content-addressed storage API
    StorageOpened(LoreStorageOpenedEventData),
    StoragePutItemComplete(LoreStoragePutItemCompleteEventData),
    StorageGetHeader(LoreStorageGetHeaderEventData),
    StorageGetData(LoreStorageGetDataEventData),
    StorageGetItemComplete(LoreStorageGetItemCompleteEventData),
    StorageGetMetadataItemComplete(LoreStorageGetMetadataItemCompleteEventData),
    StorageCopyItemComplete(LoreStorageCopyItemCompleteEventData),
    StorageObliterateItemComplete(LoreStorageObliterateItemCompleteEventData),
    StorageUploadItemComplete(LoreStorageUploadItemCompleteEventData),
    // Low-level memory-based revision control API
    RevisionTreeLoaded(LoreRevisionTreeLoadedEventData),
    RevisionTreeResolvePathComplete(LoreRevisionTreeResolvePathCompleteEventData),
    RevisionTreeChild(LoreRevisionTreeChildEventData),
    RevisionTreeNodeInfo(LoreRevisionTreeNodeInfoEventData),
    RevisionTreeNodePath(LoreRevisionTreeNodePathEventData),
    RevisionTreeAddComplete(LoreRevisionTreeAddCompleteEventData),
    RevisionTreeDeleteComplete(LoreRevisionTreeDeleteCompleteEventData),
    RevisionTreeModifyComplete(LoreRevisionTreeModifyCompleteEventData),
    RevisionTreeMoveComplete(LoreRevisionTreeMoveCompleteEventData),
    RevisionTreeMetadataSetComplete(LoreRevisionTreeMetadataSetCompleteEventData),
    RevisionTreeMetadataGetComplete(LoreRevisionTreeMetadataGetCompleteEventData),
    RevisionTreeCommitComplete(LoreRevisionTreeCommitCompleteEventData),
    RevisionTreeCloseComplete(LoreRevisionTreeCloseCompleteEventData),
}

impl LoreEvent {
    pub fn send(self) {
        execution_context().dispatcher.send(self);
    }

    pub fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe {
            let ptr = <*const Self>::from(self).cast::<u32>();
            if ptr.is_aligned() {
                *ptr
            } else {
                ptr.read_unaligned()
            }
        }
    }
}
