// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::path::Path;

use rustls::pki_types::CertificateDer;
use rustls::pki_types::PrivateKeyDer;
use rustls::pki_types::pem::PemObject;

use crate::errors::UnhandledError;

/// Load one or more certificates from a PEM file.
pub fn load_certs(path: impl AsRef<Path>) -> Result<Vec<CertificateDer<'static>>, UnhandledError> {
    CertificateDer::pem_file_iter(path)
        .map_err(|e| UnhandledError::internal_with_context(e, "failed to read certificate file"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| UnhandledError::internal_with_context(e, "invalid certificate"))
}

/// Load a private key from a PEM file
pub fn load_private_key(path: impl AsRef<Path>) -> Result<PrivateKeyDer<'static>, UnhandledError> {
    PrivateKeyDer::from_pem_file(path).map_err(|e| match e {
        rustls::pki_types::pem::Error::NoItemsFound => {
            UnhandledError::internal("no private keys found")
        }
        other => UnhandledError::internal_with_context(other, "malformed private key"),
    })
}
