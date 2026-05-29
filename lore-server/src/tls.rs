// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::path::PathBuf;

use lore_error_set::prelude::*;
use serde::Deserialize;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Identity;

/// Referenced by config toml in several places - be careful of how you rename
/// these fields
#[derive(Clone, Debug, Deserialize)]
pub struct CertificateSettings {
    pub cert_chain: Option<PathBuf>,
    pub cert_file: PathBuf,
    pub pkey_file: PathBuf,
}

#[error_set]
pub enum LoadClientTlsError {}

pub fn load_client_tls(
    settings: CertificateSettings,
) -> Result<ClientTlsConfig, LoadClientTlsError> {
    let client_cert = std::fs::read(settings.cert_file).internal("Error loading Client Cert")?;
    let client_key = std::fs::read(settings.pkey_file).internal("Error loading Client Key")?;
    let client_identity = Identity::from_pem(client_cert, client_key);

    let mut tls = ClientTlsConfig::new().identity(client_identity);
    if let Some(ca_cert_path) = settings.cert_chain {
        let ca_cert = std::fs::read(ca_cert_path).internal("Error loading CA Cert")?;
        tls = tls.ca_certificate(Certificate::from_pem(ca_cert));
    }

    Ok(tls)
}
