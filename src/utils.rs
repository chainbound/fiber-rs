use alloy::{
    consensus::{Block, BlockBody, Header, TxEnvelope},
    eips::eip2718::Decodable2718,
    primitives::{B256, B64, U256},
    rpc::types::engine::ExecutionPayload,
    rpc::types::Withdrawals,
};
use tonic::Request;
use tracing::error;

const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CLIENT_NAME: &str = env!("CARGO_PKG_NAME");

/// Appends the api key metadata to the request, keyed by x-api-key.
pub(crate) fn append_api_key<T>(req: &mut Request<T>, key: &str) {
    req.metadata_mut().append("x-api-key", key.parse().unwrap());
}

/// Appends the client version metadata to the request, keyed by x-client-version.
pub(crate) fn append_client_version<T>(req: &mut Request<T>) {
    let client_full_name = format!("{CLIENT_NAME}-rs/v{CLIENT_VERSION}");
    req.metadata_mut()
        .append("x-client-version", client_full_name.parse().unwrap());
}

/// Appends the following metadata to the request:
/// - api-key: keyed x-api-key
/// - client version: keyed x-client-version
pub(crate) fn append_metadata<T>(req: &mut Request<T>, api_key: &str) {
    append_api_key(req, api_key);
    append_client_version(req);
}

/// Parses an execution payload into a block. Returns `None` if the payload is pre-cancun.
pub(crate) fn parse_execution_payload_to_block(
    payload: ExecutionPayload,
) -> Option<Block<TxEnvelope>> {
    let Some(v3) = payload.as_v3() else {
        error!(?payload, "payload is not a v3 execution payload");
        return None;
    };

    // Terminal difficulty (we don't support pre-Merge blocks)
    let difficulty = U256::from(58750003716598352816469u128);

    // NOTE: missing fields are set to `None` or ZERO, and documented in the library as such.
    let header = Header {
        difficulty,
        nonce: B64::ZERO,
        ommers_hash: B256::ZERO,
        beneficiary: v3.payload_inner.payload_inner.fee_recipient,
        parent_hash: v3.payload_inner.payload_inner.parent_hash,
        state_root: v3.payload_inner.payload_inner.state_root,
        receipts_root: v3.payload_inner.payload_inner.receipts_root,
        logs_bloom: v3.payload_inner.payload_inner.logs_bloom,
        number: v3.payload_inner.payload_inner.block_number,
        gas_limit: v3.payload_inner.payload_inner.gas_limit,
        gas_used: v3.payload_inner.payload_inner.gas_used,
        timestamp: v3.payload_inner.payload_inner.timestamp,
        mix_hash: v3.payload_inner.payload_inner.prev_randao,
        extra_data: v3.payload_inner.payload_inner.extra_data.clone(),
        base_fee_per_gas: Some(v3.payload_inner.payload_inner.base_fee_per_gas.to::<u64>()),
        blob_gas_used: Some(v3.blob_gas_used),
        excess_blob_gas: Some(v3.excess_blob_gas),
        transactions_root: B256::ZERO, // This field is missing in the ExecutonPayload.
        withdrawals_root: None,        // This field is missing in the ExecutonPayload.
        parent_beacon_block_root: None, // This field is missing in the ExecutonPayload.
        requests_hash: None,           // This field is missing in the ExecutonPayload.
    };

    let raw_transactions = &v3.payload_inner.payload_inner.transactions;

    let mut transactions = Vec::with_capacity(raw_transactions.len());
    for raw_transaction in raw_transactions.iter() {
        let alloy_tx = match TxEnvelope::decode_2718(&mut raw_transaction.as_ref()) {
            Ok(enveloped) => enveloped,
            Err(e) => {
                error!("failed to decode tx in block: {}", e);
                continue;
            }
        };

        transactions.push(alloy_tx);
    }

    let withdrawals = match payload {
        ExecutionPayload::V3(v3) => Some(Withdrawals::from(v3.payload_inner.withdrawals)),
        ExecutionPayload::V2(v2) => Some(Withdrawals::from(v2.withdrawals)),
        ExecutionPayload::V1(_) => None,
    };

    Some(Block {
        header,
        body: BlockBody {
            transactions,
            ommers: Vec::new(),
            withdrawals,
        },
    })
}
