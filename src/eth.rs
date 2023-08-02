#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockNumber {
    #[prost(oneof = "block_number::BlockNumber", tags = "1, 2, 3")]
    pub block_number: ::core::option::Option<block_number::BlockNumber>,
}
/// Nested message and enum types in `BlockNumber`.
pub mod block_number {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum BlockNumber {
        #[prost(message, tag = "1")]
        Latest(()),
        #[prost(message, tag = "2")]
        Pending(()),
        #[prost(uint64, tag = "3")]
        Number(u64),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(oneof = "block_id::Id", tags = "1, 2")]
    pub id: ::core::option::Option<block_id::Id>,
}
/// Nested message and enum types in `BlockId`.
pub mod block_id {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Id {
        #[prost(message, tag = "1")]
        Hash(super::super::types::H256),
        #[prost(message, tag = "2")]
        Number(super::BlockNumber),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalTransactionData {
    #[prost(message, optional, tag = "1")]
    pub block_hash: ::core::option::Option<super::types::H256>,
    #[prost(uint64, tag = "2")]
    pub block_number: u64,
    #[prost(uint64, tag = "3")]
    pub index: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessListItem {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::types::H160>,
    #[prost(message, repeated, tag = "2")]
    pub slots: ::prost::alloc::vec::Vec<super::types::H256>,
}
/// / Typed transaction
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(bytes = "vec", optional, tag = "1")]
    pub to: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag = "2")]
    pub gas: u64,
    #[prost(uint64, tag = "3")]
    pub gas_price: u64,
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub input: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "6")]
    pub nonce: u64,
    #[prost(bytes = "vec", tag = "7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", optional, tag = "8")]
    pub from: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint32, tag = "9")]
    pub r#type: u32,
    /// = maxFeePerGas = GasFeeCap
    #[prost(uint64, tag = "10")]
    pub max_fee: u64,
    /// = maxPriorityFeePerGas = GasTipCap
    #[prost(uint64, tag = "11")]
    pub priority_fee: u64,
    #[prost(uint64, tag = "12")]
    pub v: u64,
    #[prost(bytes = "vec", tag = "13")]
    pub r: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "14")]
    pub s: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "15")]
    pub chain_id: u32,
    #[prost(message, repeated, tag = "16")]
    pub access_list: ::prost::alloc::vec::Vec<AccessTuple>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessTuple {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub storage_keys: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// An execution payload
/// Last update: Capella
/// We don't include the withdrawals because we're not interested in them
/// and they add some overhead.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionPayload {
    /// The execution payload header
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<ExecutionPayloadHeader>,
    #[prost(message, repeated, tag = "2")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
}
/// The execution payload header: <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#executionpayloadheader>
/// Last update: Capella
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutionPayloadHeader {
    #[prost(bytes = "vec", tag = "1")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub fee_recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub receipts_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub logs_bloom: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "6")]
    pub prev_randao: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "7")]
    pub block_number: u64,
    #[prost(uint64, tag = "8")]
    pub gas_limit: u64,
    #[prost(uint64, tag = "9")]
    pub gas_used: u64,
    #[prost(uint64, tag = "10")]
    pub timestamp: u64,
    #[prost(bytes = "vec", tag = "11")]
    pub extra_data: ::prost::alloc::vec::Vec<u8>,
    /// Big endian encoded U256
    #[prost(bytes = "vec", tag = "12")]
    pub base_fee_per_gas: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "13")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "14")]
    pub transactions_root: ::prost::alloc::vec::Vec<u8>,
    /// Only: Capella
    #[prost(bytes = "vec", optional, tag = "15")]
    pub withdrawals_root: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
/// The beacon block: <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
/// Last update: Phase0
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BeaconBlock {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, tag = "2")]
    pub proposer_index: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub parent_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub body: ::core::option::Option<BeaconBlockBody>,
}
/// Type that follows \[`BeaconBlock`\] but without the execution payload, relying
/// on \[`CompactBeaconBlockBody`\] instead.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBeaconBlock {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, tag = "2")]
    pub proposer_index: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub parent_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub body: ::core::option::Option<CompactBeaconBlockBody>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedBeaconBlock {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<BeaconBlock>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// The beacon block body: <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#beaconblockbody>
/// Last update: Capella
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BeaconBlockBody {
    #[prost(bytes = "vec", tag = "1")]
    pub randao_reveal: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub eth1_data: ::core::option::Option<Eth1Data>,
    #[prost(bytes = "vec", tag = "3")]
    pub graffiti: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub proposer_slashings: ::prost::alloc::vec::Vec<ProposerSlashing>,
    #[prost(message, repeated, tag = "5")]
    pub attester_slashings: ::prost::alloc::vec::Vec<AttesterSlashing>,
    #[prost(message, repeated, tag = "6")]
    pub attestations: ::prost::alloc::vec::Vec<Attestation>,
    #[prost(message, repeated, tag = "7")]
    pub deposits: ::prost::alloc::vec::Vec<Deposit>,
    #[prost(message, repeated, tag = "8")]
    pub voluntary_exits: ::prost::alloc::vec::Vec<SignedVoluntaryExit>,
    #[prost(message, optional, tag = "9")]
    pub sync_aggregate: ::core::option::Option<SyncAggregate>,
    #[prost(message, optional, tag = "10")]
    pub execution_payload: ::core::option::Option<ExecutionPayload>,
    /// Only: Capella
    #[prost(message, repeated, tag = "11")]
    pub bls_to_execution_changes: ::prost::alloc::vec::Vec<SignedBlsToExecutionChange>,
}
/// Custom type that follows \[`BeaconBlockBody`\] but without the execution payload.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBeaconBlockBody {
    #[prost(bytes = "vec", tag = "1")]
    pub randao_reveal: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub eth1_data: ::core::option::Option<Eth1Data>,
    #[prost(bytes = "vec", tag = "3")]
    pub graffiti: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub proposer_slashings: ::prost::alloc::vec::Vec<ProposerSlashing>,
    #[prost(message, repeated, tag = "5")]
    pub attester_slashings: ::prost::alloc::vec::Vec<AttesterSlashing>,
    #[prost(message, repeated, tag = "6")]
    pub attestations: ::prost::alloc::vec::Vec<Attestation>,
    #[prost(message, repeated, tag = "7")]
    pub deposits: ::prost::alloc::vec::Vec<Deposit>,
    #[prost(message, repeated, tag = "8")]
    pub voluntary_exits: ::prost::alloc::vec::Vec<SignedVoluntaryExit>,
    #[prost(message, optional, tag = "9")]
    pub sync_aggregate: ::core::option::Option<SyncAggregate>,
    /// Only: Capella
    #[prost(message, repeated, tag = "10")]
    pub bls_to_execution_changes: ::prost::alloc::vec::Vec<SignedBlsToExecutionChange>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedBeaconBlockHeader {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<BeaconBlockHeader>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblockheader>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BeaconBlockHeader {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, tag = "2")]
    pub proposer_index: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub parent_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub body_root: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Eth1Data {
    #[prost(bytes = "vec", tag = "1")]
    pub deposit_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub deposit_count: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoluntaryExit {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<VoluntaryExit>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoluntaryExit {
    #[prost(uint64, tag = "1")]
    pub epoch: u64,
    #[prost(uint64, tag = "2")]
    pub validator_index: u64,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposerSlashing {
    #[prost(message, optional, tag = "1")]
    pub header_1: ::core::option::Option<SignedBeaconBlockHeader>,
    #[prost(message, optional, tag = "2")]
    pub header_2: ::core::option::Option<SignedBeaconBlockHeader>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AttesterSlashing {
    #[prost(message, optional, tag = "1")]
    pub attestation_1: ::core::option::Option<IndexedAttestation>,
    #[prost(message, optional, tag = "2")]
    pub attestation_2: ::core::option::Option<IndexedAttestation>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#indexedattestation>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexedAttestation {
    #[prost(uint64, repeated, tag = "1")]
    pub attesting_indices: ::prost::alloc::vec::Vec<u64>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<AttestationData>,
    #[prost(bytes = "vec", tag = "3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AttestationData {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, tag = "2")]
    pub index: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub beacon_block_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "4")]
    pub source: ::core::option::Option<Checkpoint>,
    #[prost(message, optional, tag = "5")]
    pub target: ::core::option::Option<Checkpoint>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Checkpoint {
    #[prost(uint64, tag = "1")]
    pub epoch: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub root: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Attestation {
    #[prost(bytes = "vec", tag = "1")]
    pub aggregation_bits: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<AttestationData>,
    #[prost(bytes = "vec", tag = "3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Deposit {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub proof: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<DepositData>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DepositData {
    #[prost(bytes = "vec", tag = "1")]
    pub pubkey: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub withdrawal_credentials: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub amount: u64,
    #[prost(bytes = "vec", tag = "4")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#syncaggregate>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncAggregate {
    #[prost(bytes = "vec", tag = "1")]
    pub sync_committee_bits: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub sync_committee_signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#signedblstoexecutionchange>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedBlsToExecutionChange {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<BlsToExecutionChange>,
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#blstoexecutionchange>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlsToExecutionChange {
    #[prost(uint64, tag = "1")]
    pub validator_index: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub from_bls_pubkey: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub to_execution_address: ::prost::alloc::vec::Vec<u8>,
}
