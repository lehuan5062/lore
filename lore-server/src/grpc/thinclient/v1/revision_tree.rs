// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::pin::Pin;
use std::sync::Arc;

use lore_base::lore_spawn;
use lore_base::runtime::LORE_CONTEXT;
use lore_base::types::Hash;
use lore_proto::lore::model::v1 as model_v1;
use lore_proto::lore::thin_client::v1 as thin_client_v1;
use lore_proto::lore::thin_client::v1::RevisionTreeRequest;
use lore_proto::lore::thin_client::v1::RevisionTreeResponse;
use lore_proto::lore::thin_client::v1::revision_tree_response::Payload;
use lore_revision::repository::RepositoryContext;
use lore_revision::revision::tree;
use lore_revision::util::path::RelativePath;
use lore_telemetry::tracing::fields::REPOSITORY_ID;
use lore_telemetry::tracing::fields::REVISION;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tracing::Instrument;
use tracing::debug;
use tracing::warn;

use super::helpers::node_flags_to_node_type;
use super::helpers::resolve_to_identifier;
use crate::grpc::extract_correlation_id;
use crate::grpc::get_repository;
use crate::grpc::get_user_id;
use crate::grpc::warn_error_to_status;
use crate::util::setup_execution;

type RevisionTreeStream =
    Pin<Box<dyn Stream<Item = Result<RevisionTreeResponse, Status>> + Send + 'static>>;

/// `lore.thin_client.v1.ThinClientService.RevisionTree` handler.
///
/// Server-streams a `RevisionTreeHeader` first (echoing the resolved
/// revision identifier + signature), then one `TreeNode` per entry at
/// or under the optional `path_prefix`, bounded by `max_depth` when
/// set. The header is always emitted before the first node — failures
/// during resolution surface as a non-OK `Status` from the unary part
/// of the call, before the stream begins.
#[tracing::instrument(name = "RevisionTree::v1::handle", skip_all)]
pub async fn handler(
    request: Request<RevisionTreeRequest>,
    immutable_store: Arc<dyn lore_storage::ImmutableStore>,
    mutable_store: Arc<dyn lore_storage::MutableStore>,
) -> Result<Response<RevisionTreeStream>, Status> {
    let repository_id = get_repository(request.metadata())?;
    let user_id = get_user_id(request.extensions());
    let correlation_id = extract_correlation_id(&request).unwrap_or_default();
    let req = request.into_inner();

    let Some(query) = req.query else {
        return Err(Status::invalid_argument(
            "RevisionTreeRequest.query must be set (identifier or signature)",
        ));
    };

    let path = match req.path_prefix.as_deref() {
        Some(s) if !s.is_empty() => RelativePath::new_from_initial_path(s)
            .map_err(|err| Status::invalid_argument(format!("invalid path_prefix: {err}")))?,
        _ => RelativePath::new(),
    };
    let max_depth = req.max_depth.map_or(usize::MAX, |d| d as usize);

    let execution = setup_execution(module_path!(), correlation_id, user_id);
    let repository = Arc::new(RepositoryContext::new_server_context(
        immutable_store,
        mutable_store,
        repository_id,
    ));

    LORE_CONTEXT
        .scope(execution, async move {
            // Resolve up-front so the unary part of the call can surface
            // NotFound / Internal before the stream opens.
            let (signature, identifier) = resolve_to_identifier(&repository, query.into()).await?;

            let (tx, rx) = mpsc::channel(64);
            let header = thin_client_v1::RevisionTreeHeader {
                identifier: Some(identifier),
                signature: signature.into(),
            };

            lore_spawn!(
                async move {
                    stream_tree(repository, signature, path, max_depth, header, tx).await;
                }
                .in_current_span()
            );

            let stream: RevisionTreeStream = Box::pin(ReceiverStream::from(rx));
            Ok(Response::new(stream))
        })
        .await
}

async fn stream_tree(
    repository: Arc<RepositoryContext>,
    signature: Hash,
    path: RelativePath,
    max_depth: usize,
    header: thin_client_v1::RevisionTreeHeader,
    tx: mpsc::Sender<Result<RevisionTreeResponse, Status>>,
) {
    // Emit header first. If the client has already dropped, just bail.
    if tx
        .send(Ok(RevisionTreeResponse {
            payload: Some(Payload::Header(header)),
        }))
        .await
        .is_err()
    {
        debug!("RevisionTree receiver dropped before header");
        return;
    }

    let result = match tree(repository.clone(), signature, path, max_depth).await {
        Ok(result) => result,
        Err(err) => {
            let status = if err.is_invalid_path() {
                Status::invalid_argument("Cannot calculate tree for path that is not a directory")
            } else if err.is_node_not_found() {
                Status::not_found("A node in the tree could not be found")
            } else {
                warn!(
                    {REPOSITORY_ID} = %repository.id, {REVISION} = %signature, ?err,
                    "Failed to walk revision tree",
                );
                warn_error_to_status(&err, |e| Status::internal(e.to_string()))
            };
            let _ = tx.send(Err(status)).await;
            return;
        }
    };

    let mut emitted: u64 = 0;
    for tree_path in result.paths {
        let node = thin_client_v1::TreeNode {
            path: tree_path.path.to_string(),
            node_type: node_flags_to_node_type(tree_path.flags) as i32,
            address: tree_path.address.map(|address| model_v1::Address {
                hash: address.hash.into(),
                context: address.context.into(),
            }),
        };
        if tx
            .send(Ok(RevisionTreeResponse {
                payload: Some(Payload::Node(node)),
            }))
            .await
            .is_err()
        {
            debug!(emitted, "RevisionTree receiver dropped mid-stream");
            return;
        }
        emitted += 1;
    }

    debug!(emitted, "RevisionTree complete");
}

#[cfg(test)]
mod test {
    use lore_base::runtime::LORE_CONTEXT;
    use lore_proto::lore::thin_client::v1::revision_tree_request::Query;
    use lore_revision::branch;
    use lore_revision::branch::DEFAULT_HISTORY_STEP_SIZE;
    use lore_revision::lore::BranchId;
    use lore_revision::lore::RepositoryId;
    use lore_revision::metadata::Metadata;
    use lore_revision::node::Node;
    use lore_revision::node::NodeFlags;
    use lore_revision::node::ROOT_NODE;
    use lore_revision::state;
    use lore_storage::hash::hash_string;
    use lore_transport::grpc::REPOSITORY_ID_KEY;
    use rand::random;
    use tokio_stream::StreamExt;
    use tonic::Request;

    use super::*;
    use crate::grpc::get_write_token;
    use crate::grpc::handlers::branch_push;
    use crate::store::test_store_create;

    fn make_request(
        repository: RepositoryId,
        query: Query,
        path_prefix: Option<String>,
        max_depth: Option<u32>,
    ) -> Request<RevisionTreeRequest> {
        let mut request = Request::new(RevisionTreeRequest {
            query: Some(query),
            path_prefix,
            max_depth,
        });
        request.metadata_mut().insert_bin(
            REPOSITORY_ID_KEY,
            tonic::metadata::BinaryMetadataValue::from_bytes(repository.data()),
        );
        request
    }

    /// Pushes a fresh branch and one revision for each of `revisions`; each
    /// revision's state contains `revisions[i]` as File nodes at the root.
    /// Returns the branch id and the revision signatures in push order.
    async fn push_branch_with_revisions(
        repository: &Arc<RepositoryContext>,
        revisions: &[&[&str]],
    ) -> (BranchId, Vec<Hash>) {
        let write_token = get_write_token();
        let branch_id = BranchId::from(uuid::Uuid::now_v7());
        branch::create(
            repository.clone(),
            &write_token,
            branch_id,
            "test-branch",
            branch::default_category(),
            "creator",
            1,
            vec![],
            false,
            false,
        )
        .await
        .expect("create branch");

        let mut signatures = Vec::with_capacity(revisions.len());
        let mut parent = Hash::default();
        for (idx, files) in revisions.iter().enumerate() {
            let mut metadata = Metadata::new();
            metadata.set_branch(branch_id).expect("set branch");
            let metadata_hash = metadata
                .serialize(repository.clone())
                .await
                .expect("serialize metadata");

            let state = state::State::new();
            state.set_parent_self(parent);
            state.set_revision_number((idx + 1) as u64);
            state.set_metadata_hash(metadata_hash);
            for name in *files {
                let node = Node {
                    flags: NodeFlags::File.bits(),
                    name_hash: hash_string(name),
                    ..Default::default()
                };
                state
                    .node_add(repository.clone(), ROOT_NODE, node, name)
                    .await
                    .expect("node_add");
            }
            let serialized = state
                .serialize(repository.clone(), &write_token)
                .await
                .expect("serialize state");
            let signature = branch_push::push(
                repository.clone(),
                branch_id,
                serialized,
                true,
                true,
                false,
                DEFAULT_HISTORY_STEP_SIZE,
                crate::grpc::server::RevisionListAcceleration::default(),
            )
            .await
            .expect("push")
            .revision;
            signatures.push(signature);
            parent = signature;
        }
        (branch_id, signatures)
    }

    /// Push a branch with one revision laying out a small tree:
    /// `top.txt` and `subdir/inner.txt`.
    async fn push_branch_with_subdir(repository: &Arc<RepositoryContext>) -> (BranchId, Hash) {
        let write_token = get_write_token();
        let branch_id = BranchId::from(uuid::Uuid::now_v7());
        branch::create(
            repository.clone(),
            &write_token,
            branch_id,
            "test-branch",
            branch::default_category(),
            "creator",
            1,
            vec![],
            false,
            false,
        )
        .await
        .expect("create branch");

        let mut metadata = Metadata::new();
        metadata.set_branch(branch_id).expect("set branch");
        let metadata_hash = metadata
            .serialize(repository.clone())
            .await
            .expect("serialize metadata");

        let state = state::State::new();
        state.set_parent_self(Hash::default());
        state.set_revision_number(1);
        state.set_metadata_hash(metadata_hash);

        // Top-level file.
        let top = Node {
            flags: NodeFlags::File.bits(),
            name_hash: hash_string("top.txt"),
            ..Default::default()
        };
        state
            .node_add(repository.clone(), ROOT_NODE, top, "top.txt")
            .await
            .expect("node_add top.txt");

        // Top-level directory (no File/Link bits = directory).
        let subdir_node = Node {
            flags: NodeFlags::NoFlags.bits(),
            name_hash: hash_string("subdir"),
            ..Default::default()
        };
        let subdir_id = state
            .node_add(repository.clone(), ROOT_NODE, subdir_node, "subdir")
            .await
            .expect("node_add subdir");

        // File inside the subdirectory.
        let inner = Node {
            flags: NodeFlags::File.bits(),
            name_hash: hash_string("inner.txt"),
            ..Default::default()
        };
        state
            .node_add(repository.clone(), subdir_id, inner, "inner.txt")
            .await
            .expect("node_add inner.txt");

        let serialized = state
            .serialize(repository.clone(), &write_token)
            .await
            .expect("serialize state");
        let signature = branch_push::push(
            repository.clone(),
            branch_id,
            serialized,
            true,
            true,
            false,
            DEFAULT_HISTORY_STEP_SIZE,
            crate::grpc::server::RevisionListAcceleration::default(),
        )
        .await
        .expect("push")
        .revision;
        (branch_id, signature)
    }

    /// Pushes a fresh branch and one revision with `file_names` as File
    /// nodes at the root. Returns the branch id and the revision
    /// signature. Convenience wrapper around `push_branch_with_revisions`
    /// for tests that don't care about history depth.
    async fn push_branch_with_files(
        repository: &Arc<RepositoryContext>,
        file_names: &[&str],
    ) -> (BranchId, Hash) {
        let (branch_id, signatures) = push_branch_with_revisions(repository, &[file_names]).await;
        (branch_id, signatures[0])
    }

    async fn collect(
        response: Response<RevisionTreeStream>,
    ) -> Vec<Result<RevisionTreeResponse, Status>> {
        response.into_inner().collect().await
    }

    #[tokio::test]
    async fn unset_query_returns_invalid_argument() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let mut request = Request::new(RevisionTreeRequest {
                query: None,
                path_prefix: None,
                max_depth: None,
            });
            request.metadata_mut().insert_bin(
                REPOSITORY_ID_KEY,
                tonic::metadata::BinaryMetadataValue::from_bytes(repository.data()),
            );
            let err = match handler(request, immutable_store, mutable_store).await {
                Ok(_) => panic!("unset query should fail"),
                Err(err) => err,
            };
            assert_eq!(err.code(), tonic::Code::InvalidArgument);
        }))
        .await;
    }

    #[tokio::test]
    async fn signature_query_emits_header_then_nodes() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (branch_id, signature) =
                push_branch_with_files(&repository_context, &["a.txt", "b.txt"]).await;

            let response = handler(
                make_request(repository, Query::Signature(signature.into()), None, None),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let items: Vec<_> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .collect();

            // First message is the header echoing the resolved
            // identifier and signature.
            let header = match &items[0].payload {
                Some(Payload::Header(h)) => h,
                other => panic!("expected header first, got {other:?}"),
            };
            assert_eq!(Hash::from(header.signature.as_ref()), signature);
            let identifier = header.identifier.as_ref().expect("identifier");
            assert_eq!(identifier.number, 1);
            assert_eq!(BranchId::from(&identifier.branch_id), branch_id);

            // Remaining messages are TreeNode payloads.
            let nodes: Vec<&thin_client_v1::TreeNode> = items[1..]
                .iter()
                .map(|item| match &item.payload {
                    Some(Payload::Node(n)) => n,
                    other => panic!("expected node payload, got {other:?}"),
                })
                .collect();
            // The walk emits both files (in server-natural order).
            assert!(nodes.iter().any(|n| n.path == "a.txt"));
            assert!(nodes.iter().any(|n| n.path == "b.txt"));
            assert!(
                nodes
                    .iter()
                    .all(|n| n.node_type == thin_client_v1::NodeType::File as i32)
            );
        }))
        .await;
    }

    #[tokio::test]
    async fn identifier_query_with_zero_resolves_latest() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (branch_id, signature) =
                push_branch_with_files(&repository_context, &["only.txt"]).await;

            let response = handler(
                make_request(
                    repository,
                    Query::Identifier(model_v1::RevisionIdentifier {
                        branch_id: branch_id.into(),
                        number: 0,
                    }),
                    None,
                    None,
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let items: Vec<_> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .collect();
            let header = match &items[0].payload {
                Some(Payload::Header(h)) => h,
                other => panic!("expected header, got {other:?}"),
            };
            assert_eq!(Hash::from(header.signature.as_ref()), signature);
            assert_eq!(header.identifier.as_ref().unwrap().number, 1);
        }))
        .await;
    }

    #[tokio::test]
    async fn empty_revision_emits_header_only() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (_branch, signature) = push_branch_with_files(&repository_context, &[]).await;

            let response = handler(
                make_request(repository, Query::Signature(signature.into()), None, None),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let items: Vec<_> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .collect();
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0].payload, Some(Payload::Header(_))));
        }))
        .await;
    }

    #[tokio::test]
    async fn unknown_signature_returns_not_found() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let bogus = Hash::from(random::<[u8; 32]>());
            let err = match handler(
                make_request(repository, Query::Signature(bogus.into()), None, None),
                immutable_store,
                mutable_store,
            )
            .await
            {
                Ok(_) => panic!("unknown signature should fail"),
                Err(err) => err,
            };
            assert_eq!(err.code(), tonic::Code::NotFound);
        }))
        .await;
    }

    #[tokio::test]
    async fn path_prefix_pointing_at_file_returns_invalid_argument_on_stream() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (_branch, signature) =
                push_branch_with_files(&repository_context, &["file.txt"]).await;

            let response = handler(
                make_request(
                    repository,
                    Query::Signature(signature.into()),
                    Some("file.txt".into()),
                    None,
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("unary part succeeds");

            // The header is emitted first; the error follows on the stream.
            let items: Vec<_> = collect(response).await;
            assert!(matches!(
                items[0].as_ref().unwrap().payload,
                Some(Payload::Header(_))
            ));
            let err = items[1].as_ref().expect_err("expected error item");
            assert_eq!(err.code(), tonic::Code::InvalidArgument);
        }))
        .await;
    }

    #[tokio::test]
    async fn path_prefix_pointing_at_missing_returns_not_found_on_stream() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (_branch, signature) = push_branch_with_files(&repository_context, &[]).await;

            let response = handler(
                make_request(
                    repository,
                    Query::Signature(signature.into()),
                    Some("missing".into()),
                    None,
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("unary part succeeds");

            let items: Vec<_> = collect(response).await;
            assert!(matches!(
                items[0].as_ref().unwrap().payload,
                Some(Payload::Header(_))
            ));
            let err = items[1].as_ref().expect_err("expected error item");
            assert_eq!(err.code(), tonic::Code::NotFound);
        }))
        .await;
    }

    #[tokio::test]
    async fn identifier_query_with_concrete_number_returns_that_revision() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            // Two revisions: rev 1 with "first.txt", rev 2 with both files.
            let (branch_id, signatures) = push_branch_with_revisions(
                &repository_context,
                &[&["first.txt"], &["first.txt", "second.txt"]],
            )
            .await;

            // Querying (branch, 1) MUST resolve to revision 1 — the
            // earlier signature, not the latest. Exercises the
            // revision::resolve(`branch@N`) path.
            let response = handler(
                make_request(
                    repository,
                    Query::Identifier(model_v1::RevisionIdentifier {
                        branch_id: branch_id.into(),
                        number: 1,
                    }),
                    None,
                    None,
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let items: Vec<_> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .collect();
            let header = match &items[0].payload {
                Some(Payload::Header(h)) => h,
                other => panic!("expected header, got {other:?}"),
            };
            assert_eq!(header.identifier.as_ref().unwrap().number, 1);
            assert_eq!(Hash::from(header.signature.as_ref()), signatures[0]);
        }))
        .await;
    }

    #[tokio::test]
    async fn path_prefix_at_subdirectory_emits_descendants() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (_branch, signature) = push_branch_with_subdir(&repository_context).await;

            // Walking from `subdir` should not see `top.txt`.
            let response = handler(
                make_request(
                    repository,
                    Query::Signature(signature.into()),
                    Some("subdir".into()),
                    None,
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let nodes: Vec<thin_client_v1::TreeNode> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .filter_map(|item| match item.payload {
                    Some(Payload::Node(n)) => Some(n),
                    _ => None,
                })
                .collect();

            let paths: Vec<&str> = nodes.iter().map(|n| n.path.as_str()).collect();
            assert!(
                paths.contains(&"subdir/inner.txt"),
                "expected subdir/inner.txt in {paths:?}",
            );
            assert!(
                !paths.contains(&"top.txt"),
                "top.txt should be filtered out, got {paths:?}",
            );
        }))
        .await;
    }

    #[tokio::test]
    async fn max_depth_one_excludes_grandchildren() {
        let repository = random::<RepositoryId>();
        let (immutable_store, mutable_store, execution) =
            test_store_create().await.expect("test stores");

        Box::pin(LORE_CONTEXT.scope(execution, async move {
            let repository_context = Arc::new(RepositoryContext::new_server_context(
                immutable_store.clone(),
                mutable_store.clone(),
                repository,
            ));
            let (_branch, signature) = push_branch_with_subdir(&repository_context).await;

            // max_depth = 1 from root: direct children only. The subdir
            // entry is emitted, but `subdir/inner.txt` (a grandchild)
            // must not be.
            let response = handler(
                make_request(
                    repository,
                    Query::Signature(signature.into()),
                    None,
                    Some(1),
                ),
                immutable_store,
                mutable_store,
            )
            .await
            .expect("handler ok");

            let paths: Vec<String> = collect(response)
                .await
                .into_iter()
                .map(|r| r.expect("stream item"))
                .filter_map(|item| match item.payload {
                    Some(Payload::Node(n)) => Some(n.path),
                    _ => None,
                })
                .collect();
            assert!(
                paths.iter().any(|p| p == "top.txt"),
                "expected top.txt in {paths:?}",
            );
            assert!(
                paths.iter().any(|p| p == "subdir"),
                "expected subdir in {paths:?}",
            );
            assert!(
                paths.iter().all(|p| p != "subdir/inner.txt"),
                "max_depth=1 must exclude grandchildren, got {paths:?}",
            );
        }))
        .await;
    }
}
