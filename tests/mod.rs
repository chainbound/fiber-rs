use std::{process::Command, str::FromStr};

use ethereum_consensus::{ssz::prelude::deserialize, types::mainnet::SignedBeaconBlock};
use fiber::Client;
use reth_primitives::{
    AccessList, Address, Signature, Transaction, TransactionKind, TransactionSigned, TxEip1559,
    TxHash, TxType, B256, U256,
};
use tokio_stream::StreamExt;

mod decode;

/// Tests work only with a testing api key valid for Fiber.
const FIBER_TEST_KEY: &str = env!("FIBER_TEST_KEY");

/// Testing server endpoint
const FIBER_TEST_ENDPOINT: &str = "beta.fiberapi.io:8080";

async fn get_client() -> Client {
    Client::connect(FIBER_TEST_ENDPOINT, FIBER_TEST_KEY)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_new_type_3_transactions() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;

    let mut sub = client.subscribe_new_transactions(None).await;

    while let Some(tx) = sub.next().await {
        if tx.tx_type() == TxType::EIP4844 {
            println!("blob tx: {}", tx.hash());
            println!("blob hashes: {:?}", tx.blob_versioned_hashes());
        }
    }
}

#[tokio::test]
async fn test_new_blob_transactions() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;
    let mut start = std::time::Instant::now();

    let mut sub = client.subscribe_new_blob_transactions().await;

    let mut i = 0;
    while let Some(tx) = sub.next().await {
        println!(
            "blob tx: {:?}, blobs: {}, time since last: {:?}",
            tx.signed_transaction.hash,
            tx.signed_transaction.sidecar.blobs.len(),
            start.elapsed()
        );
        start = std::time::Instant::now();
        i += 1;

        if i > 10 {
            break;
        }
    }
}

#[tokio::test]
async fn test_new_raw_blob_transactions() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;
    let mut start = std::time::Instant::now();

    let mut sub = client.subscribe_new_raw_blob_transactions().await;

    let mut i = 0;
    while let Some(tx) = sub.next().await {
        println!(
            "raw blob tx length: {:?}, time since last: {:?}",
            tx.1.len(),
            start.elapsed()
        );
        start = std::time::Instant::now();
        i += 1;

        if i > 10 {
            break;
        }
    }
}

#[tokio::test]
async fn test_new_payloads() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;

    let mut sub = client.subscribe_new_execution_payloads().await;

    while let Some(block) = sub.next().await {
        println!(
            "block num: {}, txs: {}, block hash: {}",
            block.header.number.unwrap(),
            block.transactions.len(),
            block.header.hash.unwrap()
        );
    }
}

#[tokio::test]
async fn test_new_beacon_blocks() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;

    let mut sub = client.subscribe_new_beacon_blocks().await;

    while let Some(block) = sub.next().await {
        println!(
            "slot: {}, block num: {} block hash: {}",
            block.capella().unwrap().message.slot,
            block
                .capella()
                .unwrap()
                .message
                .body
                .execution_payload
                .block_number,
            block.capella().unwrap().message.body.eth1_data.block_hash
        );
    }
}

#[tokio::test]
async fn test_send_raw_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;

    let raw_tx_bytes =
        hex::decode("19285649286491826489162498124968129846198246912648912864").unwrap();

    let (tx_hash, timestamp) = client.send_raw_transaction(raw_tx_bytes).await.unwrap();
    println!("tx_hash: {}", tx_hash);
    println!("timestamp: {}", timestamp);
}

#[tokio::test]
async fn test_send_tx() {
    let _ = tracing_subscriber::fmt::try_init();
    let client = get_client().await;

    let signer = Address::from_str("0xdd6b8b3dc6b7ad97db52f08a275ff4483e024cea").unwrap();
    let hash =
        B256::from_str("0ec0b6a2df4d87424e5f6ad2a654e27aaeb7dac20ae9e8385cc09087ad532ee0").unwrap();

    let tx = Transaction::Eip1559(TxEip1559 {
        chain_id: 1,
            nonce: 0x42,
            gas_limit: 44386,
            to: TransactionKind::Call( Address::from_str("0x6069a6c32cf691f5982febae4faf8a6f3ab2f0f6").unwrap()),
            value: 0_u64.into(),
            input:  hex::decode("a22cb4650000000000000000000000005eee75727d804a2b13038928d36f8b188945a57a0000000000000000000000000000000000000000000000000000000000000000").unwrap().into(),
            max_fee_per_gas: 0x4a817c800,
            max_priority_fee_per_gas: 0x3b9aca00,
            access_list: AccessList::default(),
    });

    let sig = Signature {
        r: U256::from_str("0x840cfc572845f5786e702984c2a582528cad4b49b2a10b9db1be7fca90058565")
            .unwrap(),
        s: U256::from_str("0x25e7109ceb98168d95b09b18bbf6b685130e0562f233877d492b94eee0c5b6d1")
            .unwrap(),
        odd_y_parity: false,
    };

    let signed_tx = TransactionSigned::from_transaction_and_signature(tx, sig);

    assert_eq!(signed_tx.hash(), hash, "Expected same hash");
    assert_eq!(
        signed_tx.recover_signer(),
        Some(signer),
        "Recovering signer should pass."
    );

    let (tx_hash, timestamp) = client.send_transaction(signed_tx).await.unwrap();
    println!("tx_hash: {}", tx_hash);
    println!("timestamp: {}", timestamp);

    println!("expected: 0x{}", hex::encode(hash.0));

    assert_eq!(
        tx_hash,
        format!("0x{}", hex::encode(hash.0)),
        "Expected same hash"
    );
}

#[tokio::test]
async fn test_decode_transaction_rlp() {
    let tx_bytes = hex::decode("02f872018307910d808507204d2cb1827d0094388c818ca8b9251b393131c08a736a67ccb19297880320d04823e2701c80c001a0cf024f4815304df2867a1a74e9d2707b6abda0337d2d54a4438d453f4160f190a07ac0e6b3bc9395b5b9c8b9e6d77204a236577a5b18467b9175c01de4faa208d9").unwrap();
    let decoded = TransactionSigned::decode_enveloped(&mut &tx_bytes[..]).unwrap();
    assert_eq!(
        decoded.hash(),
        TxHash::from_str("0x86718885c4b4218c6af87d3d0b0d83e3cc465df2a05c048aa4db9f1a6f9de91f")
            .unwrap()
    );
}

#[tokio::test]
async fn test_validate_fiber_ssz_block() {
    let client = get_client().await;
    let mut block_sub = client.subscribe_new_beacon_blocks().await;
    let beacon_block = block_sub.next().await.unwrap();
    let slot = beacon_block.message().slot();

    // wait some time for the block to propagate and be available
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // ethdo block info --blockid $slot --ssz
    let ethdo_block = Command::new("ethdo")
        .arg("block")
        .arg("info")
        .arg("--blockid")
        .arg(slot.to_string())
        .arg("--ssz")
        .output()
        .expect("failed to execute process");

    let ethdo_block = String::from_utf8_lossy(&ethdo_block.stdout);
    let ethdo_block = hex::decode(ethdo_block.trim()).unwrap();
    let ethdo_decoded = deserialize::<SignedBeaconBlock>(&ethdo_block).unwrap();

    let a = beacon_block.message();
    let a = a.body();
    let a = a.execution_payload().unwrap();

    let b = ethdo_decoded.message();
    let b = b.body();
    let b = b.execution_payload().unwrap();

    assert_eq!(a.block_number(), b.block_number());
    assert_eq!(a.block_hash(), b.block_hash());
    assert_eq!(a.parent_hash(), b.parent_hash());
    assert_eq!(a.state_root(), b.state_root());
    assert_eq!(a.receipts_root(), b.receipts_root());
    assert_eq!(a.logs_bloom(), b.logs_bloom());
    assert_eq!(a.gas_used(), b.gas_used());
    assert_eq!(a.gas_limit(), b.gas_limit());
    assert_eq!(a.timestamp(), b.timestamp());
    assert_eq!(a.transactions(), b.transactions());
    assert_eq!(a.extra_data(), b.extra_data());
    assert_eq!(a.state_root(), b.state_root());
    assert_eq!(a.receipts_root(), b.receipts_root());
}
