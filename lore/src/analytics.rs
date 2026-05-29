// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::time::Duration;

use bytes::BufMut;
use bytes::BytesMut;
use hyper_14::Body;
use hyper_14::Client;
use hyper_14::Method;
use hyper_14::Request;
use hyper_14_rustls::HttpsConnectorBuilder;
use serde::Serialize;
use tokio::time::timeout;
use tracing::error;
use url::Url;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct AnalyticsRequestParams {
    pub app_id: String,
    pub app_version: String,
    pub app_environment: String,
    pub upload_type: String,
    pub user_agent: String,
}

#[derive(Default, Serialize)]
pub struct AnalyticsEventCollection<T> {
    #[serde(rename = "Events")]
    pub events: Vec<T>,
}

#[derive(Serialize)]
pub struct AnalyticsEvent<T> {
    #[serde(rename = "EventName")]
    pub eventname: String,
    #[serde(rename = "Value")]
    pub value: T,
}

pub async fn send_analytics_event(
    url: String,
    params: AnalyticsRequestParams,
    event: serde_json::Value,
) {
    let mut payload = BytesMut::new().writer();

    if let Err(err) = serde_json::to_writer(&mut payload, &event) {
        error!("Failed to parse event: {err}");
        return;
    }

    let Ok(builder) = HttpsConnectorBuilder::new().with_native_roots() else {
        error!("Failed to find native root CA certificates");
        return;
    };

    let https = builder.https_only().enable_http1().enable_http2().build();

    let Ok(mut full_url) = Url::parse(&url) else {
        error!("Failed to parse url");
        return;
    };

    full_url
        .query_pairs_mut()
        .append_pair("AppID", &params.app_id);
    full_url
        .query_pairs_mut()
        .append_pair("AppVersion", &params.app_version);
    full_url
        .query_pairs_mut()
        .append_pair("AppEnvironment", &params.app_environment);
    full_url
        .query_pairs_mut()
        .append_pair("UploadType", &params.upload_type);

    let request = Request::builder()
        .method(Method::POST)
        .uri(full_url.as_str())
        .header("Content-Type", "application/json")
        .header("User-Agent", &params.user_agent)
        .body(Body::from(payload.into_inner().freeze()))
        .unwrap_or_default();

    let client: Client<_, Body> = Client::builder().build(https);

    if let Ok(result) = timeout(REQUEST_TIMEOUT, client.request(request)).await {
        match result {
            Ok(response) => {
                if !response.status().is_success() {
                    error!("Analytics event response: {}", response.status());
                }
            }
            Err(err) => error!("Failed to send analytics event: {err}"),
        }
    } else {
        error!("Analytics event request timed out");
    };
}
