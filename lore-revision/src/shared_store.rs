// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::path::Path;
use std::path::PathBuf;

use lore_error_set::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::error::LoreErrorExt;
use crate::error::LoreResultExt;
use crate::errors::*;
use crate::event::EventError;
use crate::event::LoreEvent;
use crate::global;
use crate::global::GlobalConfig;
use crate::global::save_config;
use crate::interface::LoreArray;
use crate::interface::LoreError;
use crate::interface::LoreString;
use crate::lore::RepositoryId;
use crate::lore::execution_context;
use crate::protocol;
use crate::repository::RepositoryConfig;
use crate::repository::StoreConfig;
use crate::store::immutable;
use crate::store::immutable::ImmutableStoreCreateOptions;
use crate::store::immutable::ImmutableStoreSettings;
use crate::store::mutable;
use crate::util;
use crate::util::url::normalize_remote_url;

#[error_set]
pub enum SharedStoreError {
    SharedStoreNotFound,
    AddressNotFound,
    Disconnected,
    Maintenance,
    NoRemote,
    NotAuthenticated,
    NotAuthorized,
    NotFound,
    NotSupported,
    Oversized,
    PayloadNotFound,
    SlowDown,
}

impl EventError for SharedStoreError {
    fn translated(&self) -> LoreError {
        LoreError::Internal
    }

    fn inner(&self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SharedStoreConfig {
    pub remote_url: Option<String>,
    pub store_config: Option<StoreConfig>,
}

// Name of the shared store directory created inside the user provided directory
pub const SHARED_STORE_DIR: &str = "shared_store";
// Inside SHARED_STORE_DIR
pub const SHARED_STORE_CONFIG: &str = "global.toml";

pub fn find_existing_shared_store_in_dir(
    containing_dir_path: impl AsRef<Path>,
) -> Result<PathBuf, SharedStoreError> {
    // Try new name first
    let shared_store_path = containing_dir_path.as_ref().join(SHARED_STORE_DIR);
    if shared_store_path.exists() {
        return Ok(shared_store_path);
    }

    // Fall back to old name for backwards compatibility
    let global_store_path = containing_dir_path.as_ref().join("global_store");
    if global_store_path.exists() {
        return Ok(global_store_path);
    }

    // Neither exists, return error with new name
    Err(SharedStoreNotFound {
        path: shared_store_path.display().to_string(),
    }
    .into())
}

#[repr(C)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreSharedStoreCreateEventData {
    pub path: LoreString,
}

pub async fn create_shared_store(
    path: Option<PathBuf>,
    remote_url: String,
    make_default: bool,
) -> Result<(), SharedStoreError> {
    let execution = execution_context();
    let global_cli_args = execution.globals();

    if global_cli_args.offline == 0 {
        let identity = global_cli_args.identity().unwrap_or_default();
        protocol::connect(&remote_url, identity, RepositoryId::default())
            .await
            .internal(&format!("Failed to connect to remote URL {remote_url}"))?;
    }

    let directory_containing_shared_store = if let Some(path) = path {
        path
    } else {
        GlobalConfig::suggested_path_for_remote_url(&remote_url)
            .internal("failed to make default shared store path")?
    };
    let shared_store_path = directory_containing_shared_store.join(SHARED_STORE_DIR);
    let config_path = shared_store_path.join(SHARED_STORE_CONFIG);

    let shared_store_config = SharedStoreConfig {
        remote_url: Some(remote_url.clone()),
        store_config: Some(StoreConfig::global_default()),
    };

    if shared_store_path.exists() {
        if global_cli_args.force() {
            tokio::fs::remove_dir_all(&shared_store_path)
                .await
                .internal_with(|| {
                    format!("removing shared store at {}", shared_store_path.display())
                })?;
        } else {
            return SharedStoreError::internal(format!(
                "Found existing shared store at {}",
                shared_store_path.display()
            ))
            .emit();
        }
    }

    let options = shared_store_config
        .store_config
        .as_ref()
        .map_or(ImmutableStoreCreateOptions::none(), |config| {
            config.to_options()
        });

    let immutable_store = immutable::create(
        Some(&shared_store_path),
        options,
        false,
        ImmutableStoreSettings {
            allow_partial_fragment: true, /* Client store can have partial fragments */
            protect_local_fragment: true, /* Protect local fragments from eviction */
            verify_write: shared_store_config
                .store_config
                .as_ref()
                .and_then(|config| config.verify_write)
                .unwrap_or_default(),
            ..Default::default()
        },
    )
    .await
    .forward::<SharedStoreError>("creating immutable store")?;

    mutable::create(
        Some(&shared_store_path),
        mutable::MutableStoreSettings::default(),
        immutable_store,
    )
    .await
    .forward::<SharedStoreError>("creating mutable store")?;

    LoreEvent::SharedStoreCreate(LoreSharedStoreCreateEventData {
        path: LoreString::from_str(&shared_store_path.display().to_string()),
    })
    .send();

    save_config(&shared_store_config, &config_path)
        .await
        .internal("saving shared store config")?;

    if make_default {
        let (mut global_config, lock) = GlobalConfig::load_locked()
            .await
            .internal("loading global config")?;
        global_config
            .set_default_path_for_remote_url(
                &remote_url,
                directory_containing_shared_store
                    .to_str()
                    .ok_or_else(|| SharedStoreError::internal("bad path"))?,
            )
            .internal("setting default shared store path")?;
        global_config
            .save(lock)
            .await
            .internal("saving global config")?;
    }

    Ok(())
}

fn remote_urls_equivalent(
    shared_store_url: &Option<String>,
    repository_url: &Option<String>,
) -> Result<bool, SharedStoreError> {
    if let Some(shared_store_url) = shared_store_url {
        if let Some(repository_url) = repository_url {
            Ok(normalize_remote_url(shared_store_url) == normalize_remote_url(repository_url))
        } else {
            Err(SharedStoreError::internal("No repository remote URL"))
        }
    } else {
        Err(SharedStoreError::internal("No shared store remote URL"))
    }
}

/// Given a `RepositoryConfig` either
/// - return `Ok(Some(path))` where path is the path of a shared store to use instead of .urc/immutable
/// - return `Ok(None)` indicating to use the local .urc/immutable store
/// - return `Err()`
pub async fn get_shared_store_path_for_repo(
    config: &RepositoryConfig,
) -> Result<Option<PathBuf>, SharedStoreError> {
    Ok(
        if let Some(shared_store_to_use_config) = config.shared_store_to_use.as_ref()
            && shared_store_to_use_config.use_shared_store.unwrap_or(false)
        {
            let directory_with_shared_store =
                if let Some(path) = &shared_store_to_use_config.shared_store_path {
                    util::path::make_absolute(path).emit_map_err(SharedStoreError::internal(
                        format!("bad shared store path {path}"),
                    ))?
                } else {
                    let global_config = GlobalConfig::load()
                        .await
                        .internal("loading global config")?;
                    global_config
                        .default_shared_store_directory_for_remote(
                            config
                                .remote_url
                                .as_ref()
                                .ok_or(SharedStoreError::internal("no remote url"))?,
                        )
                        .internal("getting shared store path")?
                };
            let shared_store_path = find_existing_shared_store_in_dir(directory_with_shared_store)?;
            let shared_store_config = global::load_config::<SharedStoreConfig>(
                shared_store_path.join(SHARED_STORE_CONFIG),
            )
            .await
            .map_err(|_err| SharedStoreNotFound {
                path: shared_store_path.display().to_string(),
            })?;
            if !remote_urls_equivalent(&shared_store_config.remote_url, &config.remote_url)? {
                return Err(SharedStoreError::internal(format!(
                    "Loading the shared store for a repo with remote url \"{}\" but the shared store had the remote url \"{}\"",
                    config.remote_url.clone().unwrap_or_default(),
                    shared_store_config.remote_url.unwrap_or_default(),
                )));
            }
            Some(shared_store_path)
        } else {
            None
        },
    )
}

#[repr(C)]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreSharedStoreInfoEventData {
    pub use_automatically: u8,
    pub remote_urls: LoreArray<LoreString>,
    pub paths: LoreArray<LoreString>,
    pub exists: LoreArray<u8>,
}
