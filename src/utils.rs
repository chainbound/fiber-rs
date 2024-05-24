use alloy_rpc_types::{
    AccessList, AccessListItem, Block, BlockTransactions, Header, Signature, Transaction,
};
use alloy_rpc_types_engine::ExecutionPayload;
use reth_primitives::{TransactionSigned, B256, B64, U256, U64, U8};
use tonic::Request;

const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CLIENT_NAME: &str = env!("CARGO_PKG_NAME");

/// Appends the api key metadata to the request, keyed by x-api-key.
pub(crate) fn append_api_key<T>(req: &mut Request<T>, key: &str) {
    req.metadata_mut().append("x-api-key", key.parse().unwrap());
}

/// Appends the client version metadata to the request, keyed by x-client-version.
pub(crate) fn append_client_version<T>(req: &mut Request<T>) {
    let client_full_name = format!("{}-rs/v{}", CLIENT_NAME, CLIENT_VERSION);
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

/// Parses an execution payload into a block.
pub(crate) fn parse_execution_payload_to_block(payload: ExecutionPayload) -> Block {
    let v1 = payload.as_v1();

    // Terminal difficulty (we don't support pre-Merge blocks)
    let diff = U256::from(58750003716598352816469u128);

    // NOTE: missing fields are set to `None` or ZERO, and documented in the library as such.
    let header = Header {
        hash: Some(v1.block_hash),
        total_difficulty: None,
        parent_hash: v1.parent_hash,
        uncles_hash: B256::ZERO,
        miner: v1.fee_recipient,
        state_root: v1.state_root,
        receipts_root: v1.receipts_root,
        logs_bloom: v1.logs_bloom,
        difficulty: diff,
        number: Some(U256::from(v1.block_number)),
        gas_limit: U256::from(v1.gas_limit),
        gas_used: U256::from(v1.gas_used),
        timestamp: U256::from(v1.timestamp),
        extra_data: v1.extra_data.clone(),
        mix_hash: Some(v1.prev_randao),
        nonce: Some(B64::ZERO),
        base_fee_per_gas: Some(v1.base_fee_per_gas),
        blob_gas_used: payload.as_v3().map(|v3| U64::from(v3.blob_gas_used)),
        excess_blob_gas: payload.as_v3().map(|v3| U64::from(v3.excess_blob_gas)),
        transactions_root: B256::ZERO, // This field is missing in the ExecutonPayload.
        withdrawals_root: None,        // This field is missing in the ExecutonPayload.
        parent_beacon_block_root: None, // This field is missing in the ExecutonPayload.
    };

    let mut transactions = Vec::with_capacity(v1.transactions.len());
    for (index, raw_transaction) in v1.transactions.iter().enumerate() {
        let Ok(reth_tx) = TransactionSigned::decode_enveloped(&mut raw_transaction.as_ref()) else {
            tracing::error!("failed to RLP-decode tx in block: {}", v1.block_hash);
            continue;
        };
        let Some(sender) = reth_tx.recover_signer() else {
            tracing::error!("failed to recover tx signer for tx: {}", reth_tx.hash);
            continue;
        };

        let alloy_sig = Signature {
            r: reth_tx.signature().r,
            s: reth_tx.signature().s,
            v: U256::from(reth_tx.signature().v(reth_tx.chain_id())),
            y_parity: Some(reth_tx.signature().odd_y_parity.into()),
        };

        let alloy_acl = reth_tx.access_list().map(|reth_acl| {
            AccessList(
                reth_acl
                    .iter()
                    .map(|reth_item| AccessListItem {
                        address: reth_item.address,
                        storage_keys: reth_item.storage_keys.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
        });

        let alloy_tx = Transaction {
            hash: reth_tx.hash,
            nonce: reth_tx.nonce(),
            block_hash: Some(v1.block_hash),
            block_number: Some(U256::from(v1.block_number)),
            transaction_index: Some(U256::from(index)),
            from: sender,
            to: reth_tx.to(),
            value: reth_tx.value(),
            gas_price: Some(U256::from(reth_tx.max_fee_per_gas())),
            gas: U256::from(reth_tx.gas_limit()),
            max_fee_per_gas: Some(U256::from(reth_tx.max_fee_per_gas())),
            max_priority_fee_per_gas: reth_tx.max_priority_fee_per_gas().map(U256::from),
            max_fee_per_blob_gas: reth_tx.max_fee_per_blob_gas().map(U256::from),
            input: reth_tx.input().clone(),
            signature: Some(alloy_sig),
            chain_id: reth_tx.chain_id(),
            blob_versioned_hashes: reth_tx.blob_versioned_hashes(),
            access_list: alloy_acl,
            transaction_type: Some(U8::from(u8::from(reth_tx.tx_type()))),
            other: Default::default(),
        };

        transactions.push(alloy_tx);
    }

    let transactions = BlockTransactions::Full(transactions);
    let withdrawals = payload.as_v3().map(|v3| v3.withdrawals().clone());

    Block {
        header,
        transactions,
        withdrawals,
        size: None,
        uncles: Vec::new(),
        other: Default::default(),
    }
}
