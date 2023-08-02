#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H128 {
    #[prost(uint64, tag = "1")]
    pub hi: u64,
    #[prost(uint64, tag = "2")]
    pub lo: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H160 {
    #[prost(message, optional, tag = "1")]
    pub hi: ::core::option::Option<H128>,
    #[prost(uint32, tag = "2")]
    pub lo: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H256 {
    #[prost(message, optional, tag = "1")]
    pub hi: ::core::option::Option<H128>,
    #[prost(message, optional, tag = "2")]
    pub lo: ::core::option::Option<H128>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H512 {
    #[prost(message, optional, tag = "1")]
    pub hi: ::core::option::Option<H256>,
    #[prost(message, optional, tag = "2")]
    pub lo: ::core::option::Option<H256>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H1024 {
    #[prost(message, optional, tag = "1")]
    pub hi: ::core::option::Option<H512>,
    #[prost(message, optional, tag = "2")]
    pub lo: ::core::option::Option<H512>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct H2048 {
    #[prost(message, optional, tag = "1")]
    pub hi: ::core::option::Option<H1024>,
    #[prost(message, optional, tag = "2")]
    pub lo: ::core::option::Option<H1024>,
}
