// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use lore_base::runtime::LORE_CONTEXT;
use lore_base::types::Hash;
use lore_base::types::KeyType;
use lore_revision::lore::RepositoryId;
use lore_storage::MutableStore;
use lore_storage::StoreError;
use tracing::debug;
use tracing::info;
use tracing::warn;
use zerocopy::IntoBytes;

use crate::correlation::CorrelationId;
use crate::protocol::attribute_map::AttributeMap;
use crate::protocol::attribute_map::get_user_id_from_context;
use crate::protocol::storage::messages::LoreResponse;
use crate::protocol::storage::messages::Message;
use crate::protocol::storage::messages::MessageHandleError;
use crate::protocol::storage::messages::MessageParseError;
use crate::protocol::storage::messages::Response;
use crate::util::setup_execution;

#[derive(Clone, Debug, PartialEq)]
pub struct MutableLoad {
    pub key: Hash,
    pub key_type: KeyType,
}

impl MutableLoad {
    pub fn parse(bytes: Bytes) -> Result<Self, MessageParseError> {
        if bytes.len() < size_of::<Hash>() + 1 {
            return Err(MessageParseError::InvalidFieldLength);
        }

        let key = Hash::from(&bytes[..size_of::<Hash>()]);
        let key_type = KeyType::try_from(bytes[size_of::<Hash>()])
            .map_err(|_err| MessageParseError::InvalidFieldLength)?;

        Ok(Self { key, key_type })
    }
}

pub async fn handle_mutable_load(
    key: Hash,
    key_type: KeyType,
    repository: RepositoryId,
    correlation_id: String,
    user_id: String,
    mutable_store: Arc<dyn MutableStore>,
) -> Result<LoreResponse, MessageHandleError> {
    let execution = setup_execution(module_path!(), correlation_id, user_id);

    debug!(
        "Handling mutable_load for key: {} key_type: {:?} in repository: {}",
        key, key_type, repository
    );

    LORE_CONTEXT
        .scope(execution, async move {
            match mutable_store.load(repository, key, key_type).await {
                Ok(value) => {
                    debug!("Found mutable value for key: {}", key);
                    Ok(LoreResponse::MutableLoad(MutableLoadResponse { value }))
                }
                Err(StoreError::SlowDown(_)) => Err(MessageHandleError::SlowDown),
                Err(StoreError::AddressNotFound(_)) => {
                    info!("Mutable key not found: {}", key);
                    Err(MessageHandleError::MutableDataNotFound(key))
                }
                Err(err) => {
                    warn!(error = ?err, "Failed to load mutable key: {}", key);
                    Err(MessageHandleError::StoreFailure)
                }
            }
        })
        .await
}

#[async_trait]
impl Message for MutableLoad {
    async fn handle_mutable(
        &self,
        context: Arc<AttributeMap>,
        mutable_store: Arc<dyn MutableStore>,
    ) -> Result<LoreResponse, MessageHandleError> {
        let repository = *context
            .get_or::<RepositoryId, MessageHandleError>(MessageHandleError::NotConnected)?;
        let user_id = get_user_id_from_context(&context);
        let correlation_id = context.get::<CorrelationId>().unwrap_or_default();
        handle_mutable_load(
            self.key,
            self.key_type,
            repository,
            correlation_id.to_string(),
            user_id,
            mutable_store,
        )
        .await
    }
}

#[derive(Debug, PartialEq)]
pub struct MutableLoadResponse {
    pub value: Hash,
}

impl Response for MutableLoadResponse {
    fn data(&self) -> Vec<Bytes> {
        vec![Bytes::copy_from_slice(self.value.as_bytes())]
    }
}

#[cfg(test)]
mod tests {
    use lore_base::runtime::LORE_CONTEXT;
    use lore_base::types::KeyType;
    use rand::random;

    use super::*;
    use crate::store::test_store_create;

    #[test]
    fn test_parse() {
        let key = Hash::hash_buffer(b"test-key");
        let mut bytes = bytes::BytesMut::with_capacity(size_of::<Hash>() + 1);
        bytes.extend_from_slice(key.as_bytes());
        bytes.extend_from_slice(&[KeyType::BranchMetadata as u8]);
        let result = MutableLoad::parse(bytes.freeze()).unwrap();
        assert_eq!(result.key, key);
        assert_eq!(result.key_type, KeyType::BranchMetadata);
    }

    #[test]
    fn test_parse_invalid_length() {
        let bytes = Bytes::from_static(&[0u8; 16]);
        assert_eq!(
            MutableLoad::parse(bytes),
            Err(MessageParseError::InvalidFieldLength)
        );
    }

    #[tokio::test]
    async fn test_handle_not_found() {
        let repository = random::<RepositoryId>();
        let key = Hash::hash_buffer(b"missing-key");

        let context = Arc::new(AttributeMap::default());
        context.insert(repository);

        let (_immutable_store, mutable_store, execution) =
            test_store_create().await.expect("Failed to create stores");

        let message = MutableLoad {
            key,
            key_type: KeyType::Untyped,
        };
        let result = LORE_CONTEXT
            .scope(execution, async move {
                message.handle_mutable(context, mutable_store).await
            })
            .await;

        assert!(matches!(
            result,
            Err(MessageHandleError::MutableDataNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_handle_round_trip() {
        let repository = random::<RepositoryId>();
        let key = Hash::hash_buffer(b"test-key");
        let value = Hash::hash_buffer(b"test-value");

        let context = Arc::new(AttributeMap::default());
        context.insert(repository);

        let (_immutable_store, mutable_store, execution) =
            test_store_create().await.expect("Failed to create stores");

        LORE_CONTEXT
            .scope(execution, async move {
                mutable_store
                    .clone()
                    .store(repository, key, value, KeyType::Untyped)
                    .await
                    .unwrap();

                let message = MutableLoad {
                    key,
                    key_type: KeyType::Untyped,
                };
                let result = message
                    .handle_mutable(context, mutable_store)
                    .await
                    .unwrap();

                assert_eq!(
                    result,
                    LoreResponse::MutableLoad(MutableLoadResponse { value })
                );
            })
            .await;
    }

    #[tokio::test]
    async fn test_handle_load_after_overwrite() {
        let repository = random::<RepositoryId>();
        let key = Hash::hash_buffer(b"overwrite-load-key");
        let first_value = Hash::hash_buffer(b"first");
        let second_value = Hash::hash_buffer(b"second");

        let context = Arc::new(AttributeMap::default());
        context.insert(repository);

        let (_immutable_store, mutable_store, execution) =
            test_store_create().await.expect("Failed to create stores");

        LORE_CONTEXT
            .scope(execution, async move {
                mutable_store
                    .clone()
                    .store(repository, key, first_value, KeyType::Untyped)
                    .await
                    .unwrap();
                mutable_store
                    .clone()
                    .store(repository, key, second_value, KeyType::Untyped)
                    .await
                    .unwrap();

                let message = MutableLoad {
                    key,
                    key_type: KeyType::Untyped,
                };
                let result = message
                    .handle_mutable(context, mutable_store)
                    .await
                    .unwrap();

                assert_eq!(
                    result,
                    LoreResponse::MutableLoad(MutableLoadResponse {
                        value: second_value
                    })
                );
            })
            .await;
    }

    #[tokio::test]
    async fn test_handle_load_independent_repositories() {
        let repo_a = random::<RepositoryId>();
        let repo_b = random::<RepositoryId>();
        let key = Hash::hash_buffer(b"shared-key");
        let value_a = Hash::hash_buffer(b"value-a");
        let value_b = Hash::hash_buffer(b"value-b");

        let (_immutable_store, mutable_store, execution) =
            test_store_create().await.expect("Failed to create stores");

        LORE_CONTEXT
            .scope(execution, async move {
                mutable_store
                    .clone()
                    .store(repo_a, key, value_a, KeyType::Untyped)
                    .await
                    .unwrap();
                mutable_store
                    .clone()
                    .store(repo_b, key, value_b, KeyType::Untyped)
                    .await
                    .unwrap();

                // Load from repo_a
                let context_a = Arc::new(AttributeMap::default());
                context_a.insert(repo_a);
                let msg_a = MutableLoad {
                    key,
                    key_type: KeyType::Untyped,
                };
                let result_a = msg_a
                    .handle_mutable(context_a, mutable_store.clone())
                    .await
                    .unwrap();
                assert_eq!(
                    result_a,
                    LoreResponse::MutableLoad(MutableLoadResponse { value: value_a })
                );

                // Load from repo_b
                let context_b = Arc::new(AttributeMap::default());
                context_b.insert(repo_b);
                let msg_b = MutableLoad {
                    key,
                    key_type: KeyType::Untyped,
                };
                let result_b = msg_b
                    .handle_mutable(context_b, mutable_store)
                    .await
                    .unwrap();
                assert_eq!(
                    result_b,
                    LoreResponse::MutableLoad(MutableLoadResponse { value: value_b })
                );
            })
            .await;
    }
}
