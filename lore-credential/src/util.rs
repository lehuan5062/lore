// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use url::ParseError;
use url::Url;

pub fn domain_from_url_or_url(url: &Url) -> String {
    url.domain().unwrap_or(url.as_str()).to_string()
}

pub fn domain_from_url_str_or_url(remote_url: &str) -> Result<String, ParseError> {
    url::Url::parse(remote_url).map(|url| domain_from_url_or_url(&url))
}

pub fn get_domain_or_empty(url_string: &str) -> String {
    domain_from_url_str_or_url(url_string).unwrap_or("".to_string())
}
