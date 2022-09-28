#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxFilter {
    #[prost(bytes="vec", tag="1")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    /// MethodID is the 4 bytes method identifier
    #[prost(bytes="vec", tag="3")]
    pub method_id: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockFilter {
    #[prost(string, tag="1")]
    pub producer: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawTxMsg {
    #[prost(bytes="vec", tag="1")]
    pub raw_tx: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionResponse {
    #[prost(string, tag="1")]
    pub hash: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub timestamp: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BackrunMsg {
    #[prost(string, tag="1")]
    pub hash: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub tx: ::core::option::Option<super::eth::Transaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawBackrunMsg {
    #[prost(string, tag="1")]
    pub hash: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub raw_tx: ::prost::alloc::vec::Vec<u8>,
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
            D: std::convert::TryInto<tonic::transport::Endpoint>,
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
        pub async fn subscribe_new_txs(
            &mut self,
            request: impl tonic::IntoRequest<super::TxFilter>,
        ) -> Result<
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
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn subscribe_new_blocks(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockFilter>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::super::eth::Block>>,
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
                "/api.API/SubscribeNewBlocks",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn send_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::super::eth::Transaction>,
        ) -> Result<tonic::Response<super::TransactionResponse>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn send_raw_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::RawTxMsg>,
        ) -> Result<tonic::Response<super::TransactionResponse>, tonic::Status> {
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
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Backrun is the RPC method for backrunning a transaction.
        pub async fn backrun(
            &mut self,
            request: impl tonic::IntoRequest<super::BackrunMsg>,
        ) -> Result<tonic::Response<super::TransactionResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/api.API/Backrun");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn raw_backrun(
            &mut self,
            request: impl tonic::IntoRequest<super::RawBackrunMsg>,
        ) -> Result<tonic::Response<super::TransactionResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/api.API/RawBackrun");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
