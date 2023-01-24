#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockNumber {
    #[prost(oneof="block_number::BlockNumber", tags="1, 2, 3")]
    pub block_number: ::core::option::Option<block_number::BlockNumber>,
}
/// Nested message and enum types in `BlockNumber`.
pub mod block_number {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum BlockNumber {
        #[prost(message, tag="1")]
        Latest(()),
        #[prost(message, tag="2")]
        Pending(()),
        #[prost(uint64, tag="3")]
        Number(u64),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(oneof="block_id::Id", tags="1, 2")]
    pub id: ::core::option::Option<block_id::Id>,
}
/// Nested message and enum types in `BlockId`.
pub mod block_id {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Id {
        #[prost(message, tag="1")]
        Hash(super::super::types::H256),
        #[prost(message, tag="2")]
        Number(super::BlockNumber),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalTransactionData {
    #[prost(message, optional, tag="1")]
    pub block_hash: ::core::option::Option<super::types::H256>,
    #[prost(uint64, tag="2")]
    pub block_number: u64,
    #[prost(uint64, tag="3")]
    pub index: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessListItem {
    #[prost(message, optional, tag="1")]
    pub address: ::core::option::Option<super::types::H160>,
    #[prost(message, repeated, tag="2")]
    pub slots: ::prost::alloc::vec::Vec<super::types::H256>,
}
/// / Typed transaction
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(bytes="vec", optional, tag="1")]
    pub to: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag="2")]
    pub gas: u64,
    #[prost(uint64, tag="3")]
    pub gas_price: u64,
    #[prost(bytes="vec", tag="4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="5")]
    pub input: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="6")]
    pub nonce: u64,
    #[prost(bytes="vec", tag="7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", optional, tag="8")]
    pub from: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint32, tag="9")]
    pub r#type: u32,
    /// = maxFeePerGas = GasFeeCap
    #[prost(uint64, tag="10")]
    pub max_fee: u64,
    /// = maxPriorityFeePerGas = GasTipCap
    #[prost(uint64, tag="11")]
    pub priority_fee: u64,
    #[prost(uint64, tag="12")]
    pub v: u64,
    #[prost(bytes="vec", tag="13")]
    pub r: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="14")]
    pub s: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag="15")]
    pub chain_id: u32,
    #[prost(message, repeated, tag="16")]
    pub access_list: ::prost::alloc::vec::Vec<AccessTuple>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessTuple {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", repeated, tag="2")]
    pub storage_keys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(uint64, tag="1")]
    pub number: u64,
    #[prost(bytes="vec", tag="2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="4")]
    pub prev_randao: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="5")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub receipt_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub fee_recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", optional, tag="8")]
    pub extra_data: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag="9")]
    pub gas_limit: u64,
    #[prost(uint64, tag="10")]
    pub gas_used: u64,
    #[prost(uint64, tag="11")]
    pub timestamp: u64,
    #[prost(bytes="vec", tag="12")]
    pub logs_bloom: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="13")]
    pub base_fee_per_gas: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag="14")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
}
