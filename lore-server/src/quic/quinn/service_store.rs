// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::sync::Arc;

use crate::protocol::attribute_map::AttributeMap;
use crate::quic::StreamDataHandler;

pub type StreamDataHandlerBuilder =
    Box<dyn Fn(Arc<AttributeMap>) -> Box<dyn StreamDataHandler> + Send + Sync + 'static>;

#[derive(Default)]
pub struct ServiceStore {
    services: HashMap<&'static str, StreamDataHandlerBuilder>,
}

impl ServiceStore {
    pub fn add_service(&mut self, alpn: &'static str, builder: StreamDataHandlerBuilder) {
        self.services.insert(alpn, builder);
    }

    pub fn get_supported_services(&self) -> Vec<String> {
        self.services.keys().map(|key| (*key).to_string()).collect()
    }

    pub fn get_stream_builder(
        &self,
        alpn: &str,
    ) -> Option<(&&'static str, &StreamDataHandlerBuilder)> {
        self.services.get_key_value(alpn)
    }
}
