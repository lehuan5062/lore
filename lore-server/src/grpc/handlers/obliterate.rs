// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use lore_base::runtime::LORE_CONTEXT;
use lore_base::types::Address;
use lore_proto::ObliterateRequest;
use lore_proto::ObliterateResponse;
use lore_revision::notification::NotificationSender;
use lore_storage::StoreObliterateStats;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tracing::info;
use tracing::warn;

use crate::grpc::can_obliterate;
use crate::grpc::extract_correlation_id;
use crate::grpc::get_repository;
use crate::grpc::get_user_id;
use crate::grpc::hook_error_to_status;
use crate::grpc::warn_mapped_error_status;
use crate::hooks::HookContext;
use crate::hooks::HookDispatcher;
use crate::hooks::HookPoint;
use crate::util::setup_execution;

#[allow(clippy::todo)]
#[tracing::instrument(name = "Obliterate::handle", skip_all)]
pub async fn handler(
    request: Request<ObliterateRequest>,
    immutable_store: Arc<dyn lore_storage::ImmutableStore>,
    _mutable_store: Arc<dyn lore_storage::MutableStore>,
    notification: Arc<dyn NotificationSender>,
    hook_dispatcher: &HookDispatcher,
) -> Result<Response<ObliterateResponse>, Status> {
    let repository = get_repository(request.metadata())?;
    let extensions = request.extensions().clone();
    let user_id = get_user_id(&extensions);
    let correlation_id = extract_correlation_id(&request).unwrap_or_default();
    let req = request.into_inner();
    let address = Address::from(req.address.unwrap_or_default());

    let execution = setup_execution(module_path!(), correlation_id.clone(), user_id.clone());

    LORE_CONTEXT
        .scope(execution, async move {
            if !can_obliterate(&extensions, repository) {
                warn!("Attempt to obliterate {address} in repository, but user does not have the correct permissions");
                return Err(Status::permission_denied("Permission denied"));
            }

            let hook_ctx = HookContext::builder()
                .correlation_id(correlation_id)
                .hook_point(HookPoint::Obliterate)
                .repository(repository)
                .user(user_id)
                .build();

            hook_dispatcher
                .dispatch_pre(HookPoint::Obliterate, &hook_ctx)
                .map_err(|error| {
                    let source_error = error.clone();
                    let response = hook_error_to_status(error);
                    warn_mapped_error_status(&source_error, &response);
                    response
                })?;

            info!("Handling obliterate request for address {address}");

            let stats = Arc::new(StoreObliterateStats::default());
            immutable_store
                .obliterate(repository, address, stats.clone())
                .await
                .map_err(|e| {
                    warn!("Failed to obliterate {address}: {e}");
                    if e.is_address_not_found() {
                        // Distinguish absent from internal failure so the client can map this
                        // back to `AddressNotFound` and treat it as idempotent success.
                        // Without this, every absent obliterate would surface as a generic
                        // Internal error.
                        Status::not_found(format!("Address not found: {address}"))
                    } else {
                        Status::internal(format!("Failed to obliterate {address}: {e}"))
                    }
                })?;

            info!("Successfully obliterated {address}, stats: {stats:?}");
            // TODO(jcohen): track metrics for stats

            notification
                .obliterate(repository, address)
                .await
                .map_err(|e| {
                    warn!("Failed to obliterate address: {address}: {e:?}");
                    Status::internal("Obliterate failed")
                })?;

            hook_dispatcher.spawn_post(HookPoint::Obliterate, hook_ctx);

            Ok(Response::new(ObliterateResponse {}))
        })
        .await
}
