// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::sync::Arc;

use ::quinn::ClosedStream;
use ::quinn::ReadError;
use ::quinn::ReadExactError;
use ::quinn::RecvStream;
use ::quinn::SendStream;
use ::quinn::WriteError;
use async_trait::async_trait;
use bytes::Bytes;
use lore_transport::quic::QuicErrorStatus;
use lore_transport::quic::QuicOpCode;
use lore_transport::quic::command_header::COMMAND_HEADER_SIZE;
use lore_transport::quic::command_header::CommandHeader;
use thiserror::Error;
use tracing::Span;

use crate::protocol::attribute_map::AttributeMap;
use crate::quic::quinn::service_store::StreamDataHandlerBuilder;

pub mod client_monitor;
pub mod quinn;
pub mod replication_store_service;
pub mod storage_service;
pub mod storage_service_v4;
pub mod stream_handler;

#[derive(Error, Debug)]
pub enum StreamHandlerError {
    #[error("Error reading from stream")]
    Failed,
    #[error("Failed to read from stream: {0}")]
    StreamReadError(ReadError),
    #[error("Failed to read from stream: {0}")]
    InsufficientHeaderBytes(ReadExactError),
    #[error("Bad header on stream: {0:?}")]
    BadHeader(CommandHeader),
    #[error("Reached end of stream, expected {1} bytes, read {0}")]
    EndOfStream(usize, usize),
    #[error("Stream operation failed: {0}")]
    UnknownStream(ClosedStream),
    #[error("Write failed: {0}")]
    WriteFailed(WriteError),
}

#[async_trait]
pub trait StreamDataHandler: Send + Sync + 'static {
    /// For handling data from stream-based streams (i.e. Quinn).
    async fn handle_stream(
        &self,
        recv: &mut RecvStream,
        send: SendStream,
    ) -> Result<(), StreamHandlerError>;

    /// For closing Quinn streams
    async fn close(
        &self,
        recv: &mut RecvStream,
        error_code: Option<u32>,
    ) -> Result<(), StreamHandlerError>;
}

/// This trait knows how to provide a `QuicService` for a given protocol name
/// and what protocols are supported by the server
pub trait StreamHandlerFactory: Send + Sync + 'static {
    /// All the ALPNs that have a known/allowed `QuicService`
    fn supported_protocols(&self) -> Vec<String>;

    /// Returns the builder that can make a `QuicService` backed `StreamDataHandler`
    /// for the given protocol name, or None if the protocol is not supported
    fn get_stream_handler_builder(
        &self,
        protocol: &str,
    ) -> Option<(&&'static str, &StreamDataHandlerBuilder)>;

    fn name(&self) -> &'static str {
        "Unknown StreamHandlerFactory"
    }
}

/// Sentinel rendered into a per-RPC span field when the value is not present.
pub const NO_CONNECTION_ID: &str = "<no_connection_id>";
pub const NO_REPOSITORY_ID: &str = "<no_repository_id>";
pub const NO_CORRELATION_ID: &str = "<no_correlation_id>";
pub const NO_USER_ID: &str = "<no_user_id>";

/// A QUIC service  implementation for use with the `StreamDataHandler` scaffolding.
/// From a high level, the service implementation takes input bytes from the stream and either
/// returns response bytes or an error status to be sent back to the client.
/// `ParsedRequestType` - the concrete message type the Protocol expects
/// `RequestParseErrorType` - error raised when parsing fails. This concrete type is only used for tracing
/// `RequestHandlerError` implementation specific errors that arise from handling errors.
#[async_trait]
pub trait QuicService: Send + Sync + 'static {
    type ParsedRequestType: std::fmt::Debug + Send + Sync + 'static;
    type RequestParseErrorType: std::error::Error + Send + Sync + 'static;
    type RequestHandlerError: std::error::Error + Send + Sync + 'static;

    /// For observability, what is the name of this service?
    fn get_service_name_label(&self) -> &'static str;

    /// Given the command header and payload bytes, parse into a concrete request.
    fn parse_request_bytes(
        &self,
        header: &CommandHeader,
        bytes: Bytes,
    ) -> Result<Self::ParsedRequestType, Self::RequestParseErrorType>;

    /// Given this concrete request for this protocol, carry out the request.
    /// Will return the response bytes for success
    async fn run_request_handler(
        &self,
        context: Arc<AttributeMap>,
        request: Self::ParsedRequestType,
    ) -> Result<Vec<Bytes>, Self::RequestHandlerError>;

    /// For observability, what is the opcode for this command ID?
    fn command_to_metrics_label(&self, opcode: QuicOpCode) -> &'static str;

    /// If the protocol request handling results in an error, how should this error be represented?
    fn transform_protocol_error(&self, error: &Self::RequestHandlerError) -> ProtocolErrorInfo;

    fn max_chunk_size(&self) -> usize;

    /// The size of the command header for this protocol version.
    fn header_size(&self) -> usize {
        COMMAND_HEADER_SIZE
    }

    /// Build the per-RPC `OTel` root span for an inbound opcode. Each service
    /// is responsible for sourcing its own repo / correlation / user / connection
    /// identifiers — replication reads them off the parsed `message`, while v0
    /// and v4 storage read them from per-connection or per-session state in
    /// `context`.
    fn build_request_span(
        &self,
        header: &CommandHeader,
        message: &Self::ParsedRequestType,
        context: &Arc<AttributeMap>,
    ) -> Span;
}

/// Describes how a protocol error should be represented to the client and observed.
pub struct ProtocolErrorInfo {
    /// The `QuicErrorStatus` to send back to the client.
    pub response_error_code: QuicErrorStatus,
    /// A label for observability on how this error should be recorded.
    pub message_handle_label: &'static str,
    /// Should this error be treated as an internal error?
    pub is_internal_error: bool,
    /// Whether this error is interesting from a server-side logging perspective.
    pub is_appropriate_for_logging: bool,
}

#[cfg(test)]
pub mod tests {
    use bytes::Bytes;
    use bytes::BytesMut;

    pub fn collapse_bytes_with_skip(chunks: &[Bytes], num_to_skip: usize) -> Bytes {
        let mut bytes = BytesMut::with_capacity(chunks.len() - 1);
        chunks
            .iter()
            .skip(num_to_skip)
            .for_each(|b| bytes.extend_from_slice(b));

        bytes.freeze()
    }

    pub fn collapse_bytes_without_header(chunks: &[Bytes]) -> Bytes {
        // first byte is the command header
        collapse_bytes_with_skip(chunks, 1)
    }

    pub fn collapse_bytes(chunks: &[Bytes]) -> Bytes {
        collapse_bytes_with_skip(chunks, 0)
    }
}
