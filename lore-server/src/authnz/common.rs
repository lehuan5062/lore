// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use tonic::Request;
use tonic::Status;

use crate::grpc::ServerResultExt;

// TODO: if no authorization string is passed, do not add a metadata for 'authorization'.
// See test can_create_request_without_authorization
fn grpc_set_authorization_metadata<F>(
    request: &mut Request<F>,
    authorization: Option<String>,
) -> Result<(), Status> {
    let auth_header: tonic::metadata::MetadataValue<_> = authorization
        .unwrap_or_default()
        .parse()
        .warn_map_err(|err| Status::internal(format!("Failed to create metadata: {err}")))?;
    request.metadata_mut().append("authorization", auth_header);
    Ok(())
}

pub fn create_request_with_authorization<T>(
    payload: T,
    authorization: Option<String>,
) -> Result<Request<T>, Status> {
    let mut request = tonic::Request::new(payload);
    grpc_set_authorization_metadata(&mut request, authorization)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    use super::create_request_with_authorization;

    #[test]
    fn can_create_request_with_authorization() -> Result<(), Error> {
        let payload = (4, 20);
        let request = create_request_with_authorization(payload, Some("my-auth".into()))?;
        assert_eq!(request.get_ref(), &payload);

        let auth_metadata = request.metadata().get("authorization").unwrap();
        assert_eq!(auth_metadata.to_str()?, "my-auth");

        Ok(())
    }

    #[test]
    fn can_create_request_without_authorization() -> Result<(), Error> {
        let payload = (4, 20);
        let request = create_request_with_authorization(payload, None)?;
        assert_eq!(request.get_ref(), &payload);

        let auth_metadata = request.metadata().get("authorization").unwrap();
        // looks dodgy to me but this was the original code.
        // to reduce the surface area we will keep this - providing None to `authorization`
        // results in an empty authorization metadata. If you come across this and think
        // it is strange then you are right and it could probably be changed; something
        // we don't have time/risk to investigate right
        assert_eq!(auth_metadata.to_str()?, "");

        Ok(())
    }
}
