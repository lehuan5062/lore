// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::any::Any;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use lore_base::runtime::LORE_CONTEXT;
use lore_base::types::Address;
use lore_base::types::FragmentFlags;
use lore_base::types::HealResult;
use lore_revision::lore::RepositoryId;
use lore_storage::ImmutableStore;
use lore_storage::LocalImmutableStore;
use lore_storage::StoreError;
use lore_storage::StoreMatch;
use lore_storage::decompress;
use tracing::debug;
use tracing::info;
use tracing::warn;
use zerocopy::FromBytes;

use crate::correlation::CorrelationId;
use crate::protocol::attribute_map::AttributeMap;
use crate::protocol::attribute_map::get_user_id_from_context;
use crate::protocol::storage::messages::LoreResponse;
use crate::protocol::storage::messages::Message;
use crate::protocol::storage::messages::MessageHandleError;
use crate::protocol::storage::messages::MessageParseError;
use crate::protocol::storage::messages::Response;
use crate::util::setup_execution;

#[derive(Debug, PartialEq, FromBytes)]
#[repr(C)]
pub struct Verify {
    pub address: Address,
    pub heal: u8,
}

impl Display for Verify {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self.address)
    }
}

impl Verify {
    pub fn parse(bytes: Bytes) -> Result<Self, MessageParseError>
    where
        Self: Sized,
    {
        Verify::read_from_bytes(bytes.as_ref()).map_err(|_e| MessageParseError::InvalidFieldLength)
    }
}

pub async fn handle_verify(
    address: Address,
    heal_flag: u8,
    repository: RepositoryId,
    correlation_id: String,
    user_id: String,
    local_store: Arc<dyn ImmutableStore>,
    immutable_store: Arc<dyn ImmutableStore>,
) -> Result<LoreResponse, MessageHandleError> {
    let execution = setup_execution(module_path!(), correlation_id, user_id);
    let heal = heal_flag != 0;

    debug!(%address, "Handling verify request for address");

    let match_requested = if address.context.is_zero() {
        StoreMatch::MatchPartition
    } else {
        StoreMatch::MatchFull
    };

    LORE_CONTEXT
        .scope(execution, async move {
            let concrete_local_store: Arc<LocalImmutableStore> =
                {
                    let any_store: Arc<dyn Any + Send + Sync> = local_store;
                    any_store
                        .downcast::<LocalImmutableStore>()
                        .map_err(|_err| MessageHandleError::StoreFailure)?
                };

            match concrete_local_store
                .verify_fragment(address, repository, match_requested, heal)
                .await
            {
                Ok(result) => {
                    info!(%address, "Verify result: {result:?}");

                    let mut did_zstd_decompress = false;
                    for m in &result.matches {
                        if FragmentFlags::from_bits_retain(m.data.flags)
                            .contains(FragmentFlags::PayloadCompressedZstd)
                        {
                            info!(%address, partition = %m.partition, "Attempting Zstd decompress");
                            let (fragment, compressed) = immutable_store
                                .clone()
                                .get(m.partition, m.address, StoreMatch::MatchFull)
                                .await
                                .map_err(|e| {
                                    warn!(%address, error = ?e, "Failed to get compressed fragment from store");
                                    MessageHandleError::StoreFailure
                                })?;
                            let (decompressed_fragment, decompressed) =
                                decompress(fragment, &compressed)
                                    .map_err(|e| {
                                        warn!(%address, error = ?e, "Failed to decompress zstd fragment");
                                        MessageHandleError::StoreFailure
                                    })?;
                            immutable_store
                                .clone()
                                .put(
                                    m.partition,
                                    m.address,
                                    decompressed_fragment,
                                    Some(decompressed.freeze()),
                                    true,
                                )
                                .await
                                .map_err(|e| {
                                    warn!(%address, error = ?e, "Failed to write decompressed fragment to durable store");
                                    MessageHandleError::StoreFailure
                                })?;
                            did_zstd_decompress = true;
                            info!(%address, partition = %m.partition, "Completed Zstd uncompress");
                        }
                    }

                    match result.verification_result {
                        Ok(()) =>
                            {
                                let corrupted;
                                let healed;
                                if did_zstd_decompress {
                                    corrupted = 1;
                                    healed = HealResult::Healed;
                                } else {
                                    corrupted = 0;
                                    healed = HealResult::NotAttempted;
                                }

                                Ok(LoreResponse::Verify(VerifyResponse {
                                    corrupted,
                                    healed,
                                }))
                            }
                        Err(err) => {
                            let healed = if result.healed {
                                HealResult::Healed
                            } else if heal {
                                warn!(%address, error = %err, "Attempted to heal while verifying fragment, but result indicated we did not heal?");
                                HealResult::Failed
                            } else {
                                HealResult::NotAttempted
                            };
                            Ok(LoreResponse::Verify(VerifyResponse { corrupted: 1, healed }))
                        }
                    }
                }
                Err(StoreError::AddressNotFound(_)) => {
                    info!(%address, "Fragment verification failed, fragment not found");
                    Err(MessageHandleError::FragmentNotFound)
                }
                Err(StoreError::SlowDown(_)) => Err(MessageHandleError::SlowDown),
                Err(e) => {
                    warn!(%address, error = ?e, "Fragment verification failed");
                    Err(MessageHandleError::StoreFailure)
                }
            }
        })
        .await
}

#[async_trait]
impl Message for Verify {
    #[tracing::instrument(name = "Verify::handle", skip_all)]
    async fn handle_verify(
        &self,
        context: Arc<AttributeMap>,
        local_immutable_store: Arc<dyn ImmutableStore>,
        immutable_store: Arc<dyn ImmutableStore>,
    ) -> Result<LoreResponse, MessageHandleError> {
        let repository = *context
            .get_or::<RepositoryId, MessageHandleError>(MessageHandleError::NotConnected)?;
        let user_id = get_user_id_from_context(&context);
        let correlation_id = context.get::<CorrelationId>().unwrap_or_default();
        handle_verify(
            self.address,
            self.heal,
            repository,
            correlation_id.to_string(),
            user_id,
            local_immutable_store,
            immutable_store,
        )
        .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct VerifyResponse {
    pub corrupted: u8,
    pub healed: HealResult,
}

impl Response for VerifyResponse {
    fn data(&self) -> Vec<Bytes> {
        vec![Bytes::from(vec![self.corrupted, self.healed as u8])]
    }
}

#[cfg(test)]
mod tests {
    use lore_base::types::Context;
    use lore_base::types::Fragment;
    use lore_base::types::Hash;
    use lore_revision::fragment::generate_random;
    use lore_revision::interface::ExecutionContext;
    use lore_revision::interface::LoreGlobalArgs;
    use lore_revision::relay::EventDispatcher;
    use lore_storage::CompressionMode;
    use lore_storage::compress;
    use lore_storage::hash_slice;
    use lore_storage::local::immutable_store::ImmutableStoreSettings;
    use rand::distr::SampleString;
    use rand::random;
    use zerocopy::IntoBytes;

    use super::*;
    use crate::store::test_store_create;

    fn make_verify_bytes(address: Address, heal: bool) -> Bytes {
        let mut bytes = address.as_bytes().to_vec();
        bytes.push(if heal { 1 } else { 0 });
        Bytes::from(bytes)
    }

    fn generate_tempdir() -> std::path::PathBuf {
        let testname = format!(
            "lore-verify-test-{}",
            rand::distr::Alphanumeric
                .sample_string(&mut rand::rng(), 8)
                .as_str()
        );
        let mut dir = std::env::temp_dir();
        dir.push(testname);
        std::fs::create_dir_all(&dir).expect("Create test directory");
        std::fs::canonicalize(dir).expect("Canonicalize temporary test dir")
    }

    fn setup_test_execution() -> Arc<ExecutionContext> {
        Arc::new(ExecutionContext::new_client(
            LoreGlobalArgs::default(),
            EventDispatcher::no_dispatch(),
        ))
    }

    #[test]
    fn test_parse_valid() {
        let hash = random::<Hash>();
        let context = random::<Context>();
        let address = Address { hash, context };

        assert_eq!(
            Verify::parse(make_verify_bytes(address, false)),
            Ok(Verify { address, heal: 0 })
        );
    }

    #[test]
    fn test_parse_with_heal_flag() {
        let hash = random::<Hash>();
        let context = random::<Context>();
        let address = Address { hash, context };

        assert_eq!(
            Verify::parse(make_verify_bytes(address, true)),
            Ok(Verify { address, heal: 1 })
        );
    }

    #[test]
    fn test_parse_too_short() {
        // We expect 49 bytes total, so trying to parse less than that should fail...
        assert_eq!(
            Verify::parse(Bytes::from(vec![0u8; 10])),
            Err(MessageParseError::InvalidFieldLength)
        );
    }

    #[tokio::test]
    async fn test_handle_not_connected() {
        let hash = random::<Hash>();
        let context = random::<Context>();
        let address = Address { hash, context };

        // Empty context map - no repository set
        let context_map = Arc::new(AttributeMap::default());

        let (immutable_store, _mutable_store, execution) =
            test_store_create().await.expect("Failed to create stores");

        LORE_CONTEXT
            .scope(execution, async move {
                let message = Verify { address, heal: 0 };
                match message
                    .handle_verify(context_map, immutable_store.clone(), immutable_store)
                    .await
                {
                    Err(MessageHandleError::NotConnected) => (),
                    Err(e) => panic!("Expected NotConnected error, got {e:?}"),
                    Ok(_) => panic!("Expected NotConnected error, got Ok"),
                }
            })
            .await;
    }

    #[tokio::test]
    async fn test_handle_not_found() {
        let dir = generate_tempdir();
        let dir_cleanup = dir.clone();
        let execution = setup_test_execution();

        LORE_CONTEXT
            .scope(execution, async move {
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create store");

                let repository: RepositoryId = random();

                // Write a fragment first so the index infrastructure exists
                let (fragment, address, payload) = generate_random();

                store
                    .clone()
                    .put(repository, address, fragment, Some(payload), false)
                    .await
                    .expect("Failed to put fragment");

                store.clone().flush(true).await.expect("Failed to flush");

                // Now try to verify a non-existent fragment in the same bucket
                let mut nonexistent_hash = address.hash;
                nonexistent_hash.data_mut()[2] = nonexistent_hash.data()[2].wrapping_add(1);
                let nonexistent_address = Address {
                    hash: nonexistent_hash,
                    context: random(),
                };

                let context_map = Arc::new(AttributeMap::default());
                context_map.insert(repository);

                let message = Verify {
                    address: nonexistent_address,
                    heal: 0,
                };

                match message
                    .handle_verify(context_map, store.clone(), store)
                    .await
                {
                    Err(MessageHandleError::FragmentNotFound) => (),
                    Err(e) => panic!("Expected FragmentNotFound error, got {e:?}"),
                    Ok(_) => panic!("Expected FragmentNotFound error, got Ok"),
                }
            })
            .await;

        let _ = std::fs::remove_dir_all(&dir_cleanup);
    }

    #[tokio::test]
    async fn test_handle_success() {
        let dir = generate_tempdir();
        let dir_cleanup = dir.clone();
        let execution = setup_test_execution();

        LORE_CONTEXT
            .scope(execution, async move {
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create store");

                let repository: RepositoryId = random();
                let (fragment, address, payload) = generate_random();

                store
                    .clone()
                    .put(repository, address, fragment, Some(payload), false)
                    .await
                    .expect("Failed to put fragment");

                store.clone().flush(true).await.expect("Failed to flush");

                let context_map = Arc::new(AttributeMap::default());
                context_map.insert(repository);

                let message = Verify { address, heal: 0 };

                match message
                    .handle_verify(context_map, store.clone(), store)
                    .await
                {
                    Ok(LoreResponse::Verify(resp)) => {
                        assert_eq!(resp.corrupted, 0);
                        assert_eq!(resp.healed, HealResult::NotAttempted);
                    }
                    Ok(other) => panic!("Expected Verify response, got {other:?}"),
                    Err(e) => panic!("Expected success, got error: {e:?}"),
                }
            })
            .await;

        let _ = std::fs::remove_dir_all(&dir_cleanup);
    }

    fn corrupt_packfile(
        store_path: &std::path::Path,
        group_index: u8,
        pack_file: u32,
        pack_offset: u32,
    ) {
        use std::io::Read;
        use std::io::Seek;
        use std::io::SeekFrom;
        use std::io::Write;

        let pack_path = store_path.join(format!(
            "immutable/index/{group_index:02x}/pack/{pack_file}"
        ));

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&pack_path)
            .expect("Failed to open packfile for corruption");

        file.seek(SeekFrom::Start(pack_offset as u64))
            .expect("Failed to seek to pack_offset");
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)
            .expect("Failed to read from packfile");

        file.seek(SeekFrom::Start(pack_offset as u64))
            .expect("Failed to seek to pack_offset");
        file.write_all(&[0xFF; 16])
            .expect("Failed to write corruption bytes");
        file.sync_all().expect("Failed to sync packfile");
    }

    #[tokio::test]
    async fn test_handle_corrupted_heal() {
        let dir = generate_tempdir();
        let dir_cleanup = dir.clone();
        let execution = setup_test_execution();

        LORE_CONTEXT
            .scope(execution, async move {
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir.clone()),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create store");

                let repository: RepositoryId = random();
                let (fragment, address, payload) = generate_random();

                store
                    .clone()
                    .put(repository, address, fragment, Some(payload), false)
                    .await
                    .expect("Failed to put fragment");

                store.clone().flush(true).await.expect("Failed to flush");

                let result = store
                    .clone()
                    .verify_fragment(address, repository, StoreMatch::MatchFull, false)
                    .await
                    .expect("verify_fragment failed");

                let pack_file = result.matches[0].data.pack_file;
                let pack_offset = result.matches[0].data.pack_offset;

                // Drop the store to release file handles before corrupting the packfile
                // (Windows does not allow writing to files opened by another handle without
                // FILE_SHARE_WRITE).
                drop(store);

                corrupt_packfile(&dir, address.hash.data()[0], pack_file, pack_offset);

                // Recreate the store so it reloads from disk
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir.clone()),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to recreate store");

                let context_map = Arc::new(AttributeMap::default());
                context_map.insert(repository);

                let message = Verify { address, heal: 1 };
                match message
                    .handle_verify(context_map, store.clone(), store)
                    .await
                {
                    Ok(LoreResponse::Verify(resp)) => {
                        assert_eq!(resp.corrupted, 1);
                        assert_eq!(resp.healed, HealResult::Healed);
                    }
                    Ok(other) => panic!("Expected Verify response, got {other:?}"),
                    Err(e) => panic!("Expected success with heal, got error: {e:?}"),
                }
            })
            .await;

        let _ = std::fs::remove_dir_all(&dir_cleanup);
    }

    #[tokio::test]
    async fn test_handle_corrupted_no_heal() {
        let dir = generate_tempdir();
        let dir_cleanup = dir.clone();
        let execution = setup_test_execution();

        LORE_CONTEXT
            .scope(execution, async move {
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir.clone()),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create store");

                let repository: RepositoryId = random();
                let (fragment, address, payload) = generate_random();

                store
                    .clone()
                    .put(repository, address, fragment, Some(payload), false)
                    .await
                    .expect("Failed to put fragment");

                store.clone().flush(true).await.expect("Failed to flush");

                let result = store
                    .clone()
                    .verify_fragment(address, repository, StoreMatch::MatchFull, false)
                    .await
                    .expect("verify_fragment failed");

                let pack_file = result.matches[0].data.pack_file;
                let pack_offset = result.matches[0].data.pack_offset;

                // Drop the store to release file handles before corrupting the packfile
                // (Windows does not allow writing to files opened by another handle without
                // FILE_SHARE_WRITE).
                drop(store);

                corrupt_packfile(&dir, address.hash.data()[0], pack_file, pack_offset);

                // Recreate the store so it reloads from disk
                let store = lore_storage::LocalImmutableStore::new(
                    Some(dir.clone()),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to recreate store");

                let context_map = Arc::new(AttributeMap::default());
                context_map.insert(repository);

                let message = Verify { address, heal: 0 };
                match message
                    .handle_verify(context_map, store.clone(), store)
                    .await
                {
                    Ok(LoreResponse::Verify(resp)) => {
                        assert_eq!(resp.corrupted, 1);
                        assert_eq!(resp.healed, HealResult::NotAttempted);
                    }
                    Ok(other) => panic!("Expected Verify response, got {other:?}"),
                    Err(e) => panic!("Expected success, got error: {e:?}"),
                }
            })
            .await;

        let _ = std::fs::remove_dir_all(&dir_cleanup);
    }

    /// Verifies that a zstd-compressed fragment in the durable store is decompressed and
    /// force-written back as a raw payload when `handle_verify` encounters it.
    #[tokio::test]
    async fn test_zstd_decompressed_in_durable_store() {
        let local_dir = generate_tempdir();
        let durable_dir = generate_tempdir();
        let local_dir_cleanup = local_dir.clone();
        let durable_dir_cleanup = durable_dir.clone();
        let execution = setup_test_execution();

        LORE_CONTEXT
            .scope(execution, async move {
                let local_store = lore_storage::LocalImmutableStore::new(
                    Some(local_dir),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create local store");

                let durable_store = lore_storage::LocalImmutableStore::new(
                    Some(durable_dir),
                    ImmutableStoreSettings::default(),
                )
                .await
                .expect("Failed to create durable store");

                let repository: RepositoryId = random();
                // Use a large repetitive payload so zstd can compress it efficiently.
                let content_vec: Vec<u8> = b"abcdefgh".iter().cycle().take(4096).copied().collect();
                let content = content_vec.as_slice();
                let hash = hash_slice(content);
                let address = Address {
                    hash,
                    context: random(),
                };

                let uncompressed_fragment = Fragment {
                    flags: 0,
                    size_payload: content.len() as u32,
                    size_content: content.len() as u64,
                };

                let (compressed_fragment, compressed_payload) =
                    compress(uncompressed_fragment, content, CompressionMode::Zstd)
                        .expect("Failed to compress payload");

                assert!(
                    FragmentFlags::from_bits_retain(compressed_fragment.flags)
                        .contains(FragmentFlags::PayloadCompressedZstd),
                    "Expected PayloadCompressedZstd flag after compression"
                );

                // Write the compressed fragment to both the local store (for verify_fragment)
                // and the durable store (for the decompression rewrite loop).
                local_store
                    .clone()
                    .put(
                        repository,
                        address,
                        compressed_fragment,
                        Some(compressed_payload.clone()),
                        false,
                    )
                    .await
                    .expect("Failed to put compressed fragment in local store");
                local_store
                    .clone()
                    .flush(true)
                    .await
                    .expect("Failed to flush local store");

                durable_store
                    .clone()
                    .put(
                        repository,
                        address,
                        compressed_fragment,
                        Some(compressed_payload),
                        false,
                    )
                    .await
                    .expect("Failed to put compressed fragment in durable store");
                durable_store
                    .clone()
                    .flush(true)
                    .await
                    .expect("Failed to flush durable store");

                let context_map = Arc::new(AttributeMap::default());
                context_map.insert(repository);

                let message = Verify { address, heal: 0 };
                match message
                    .handle_verify(context_map, local_store, durable_store.clone())
                    .await
                {
                    Ok(LoreResponse::Verify(resp)) => {
                        assert_eq!(resp.corrupted, 1);
                        assert_eq!(resp.healed, HealResult::Healed);
                    }
                    Ok(other) => panic!("Expected Verify response, got {other:?}"),
                    Err(e) => panic!("Expected success, got error: {e:?}"),
                }

                // The durable store should now hold the uncompressed payload.
                let (fragment_after, payload_after) = durable_store
                    .clone()
                    .get(repository, address, StoreMatch::MatchFull)
                    .await
                    .expect("Failed to get fragment from durable store after verify");

                assert!(
                    !FragmentFlags::from_bits_retain(fragment_after.flags)
                        .contains(FragmentFlags::PayloadCompressed),
                    "Expected compression flags to be cleared after decompression rewrite"
                );
                assert_eq!(
                    payload_after.as_ref(),
                    content,
                    "Decompressed payload should match original content"
                );
                assert_eq!(
                    hash_slice(&payload_after),
                    address.hash,
                    "Decompressed payload hash should match the fragment address hash"
                );
            })
            .await;

        let _ = std::fs::remove_dir_all(&local_dir_cleanup);
        let _ = std::fs::remove_dir_all(&durable_dir_cleanup);
    }
}
