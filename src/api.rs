#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxSequenceMsg {
    #[prost(message, repeated, tag = "1")]
    pub sequence: ::prost::alloc::vec::Vec<super::eth::Transaction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawTxSequenceMsg {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub raw_txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxSequenceResponse {
    #[prost(message, repeated, tag = "1")]
    pub sequence_response: ::prost::alloc::vec::Vec<TransactionResponse>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxFilter {
    #[prost(bytes = "vec", tag = "1")]
    pub encoded: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockFilter {
    #[prost(string, tag = "1")]
    pub producer: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawTxMsg {
    #[prost(bytes = "vec", tag = "1")]
    pub raw_tx: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockSubmissionMsg {
    #[prost(bytes = "vec", tag = "1")]
    pub ssz_block: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockSubmissionResponse {
    /// The slot of the block.
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    /// The re-calculated state root after reconstructing the block.
    #[prost(bytes = "vec", tag = "2")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    /// Timestamp in microseconds.
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionResponse {
    #[prost(string, tag = "1")]
    pub hash: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    pub timestamp: i64,
}
/// Generated client implementations.
pub mod api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ApiClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ApiClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ApiClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ApiClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Opens a new transaction stream with the given filter.
        pub async fn subscribe_new_txs(
            &mut self,
            request: impl tonic::IntoRequest<super::TxFilter>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::super::eth::Transaction>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.API/SubscribeNewTxs");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("api.API", "SubscribeNewTxs"));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Sends a signed transaction to the network.
        pub async fn send_transaction(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = super::super::eth::Transaction,
            >,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::TransactionResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/api.API/SendTransaction");
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new("api.API", "SendTransaction"));
            self.inner.streaming(req, path, codec).await
        }
        /// Sends a signed, RLP encoded transaction to the network
        pub async fn send_raw_transaction(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::RawTxMsg>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::TransactionResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SendRawTransaction",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SendRawTransaction"));
            self.inner.streaming(req, path, codec).await
        }
        /// Sends a sequence of signed transactions to the network.
        pub async fn send_transaction_sequence(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::TxSequenceMsg>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::TxSequenceResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SendTransactionSequence",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SendTransactionSequence"));
            self.inner.streaming(req, path, codec).await
        }
        /// Sends a sequence of signed, RLP encoded transactions to the network.
        pub async fn send_raw_transaction_sequence(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::RawTxSequenceMsg>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::TxSequenceResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SendRawTransactionSequence",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SendRawTransactionSequence"));
            self.inner.streaming(req, path, codec).await
        }
        /// Opens a stream of new execution payloads.
        pub async fn subscribe_execution_payloads(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<super::super::eth::ExecutionPayload>,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SubscribeExecutionPayloads",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SubscribeExecutionPayloads"));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Opens a stream of new execution payload headers.
        pub async fn subscribe_execution_headers(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<super::super::eth::ExecutionPayloadHeader>,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SubscribeExecutionHeaders",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SubscribeExecutionHeaders"));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Opens a stream of new beacon blocks. The beacon blocks are "compacted", meaning that the
        /// execution payload is not included.
        pub async fn subscribe_beacon_blocks(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<super::super::eth::CompactBeaconBlock>,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SubscribeBeaconBlocks",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("api.API", "SubscribeBeaconBlocks"));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Opens a bi-directional stream for new block submissions. The client stream is used to send
        /// SSZ-encoded beacon blocks, and the server stream is used to send back the state_root, slot and
        /// a local timestamp as a confirmation that the block was seen and handled.
        pub async fn submit_block_stream(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = super::BlockSubmissionMsg,
            >,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::BlockSubmissionResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/api.API/SubmitBlockStream",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new("api.API", "SubmitBlockStream"));
            self.inner.streaming(req, path, codec).await
        }
    }
}
