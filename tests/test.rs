use alloy_consensus::{TxKind, TxLegacy};
use alloy_primitives::{address, bytes, Bytes, U256};
use alloy_signer::{LocalWallet, Signer, SignerSync};
// use ethers::{
//     signers::{LocalWallet, Signer},
//     types::{
//         transaction::eip2718::TypedTransaction, Address, Bytes, Eip1559TransactionRequest,
//         TransactionRequest,
//     },
// };
// use fiber::{Client, ClientOptions};
use fiber::{Client, ClientOptions};
// use reth_primitives::TransactionSigned;
// use reth_rlp::Decodable;
use std::time::Duration;
use tokio_stream::StreamExt;

// #[test]
// fn test_rlp() {
//     let raw_hex = Bytes::from_hex("f8ac820295850b1b0edbf58301d4c094dcd41c11d444dc34ac573a35761d65487912a10180b844095ea7b30000000000000000000000005200a0e9b161bc59feecb165fe2592bef3e1847affffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff25a062928dd0e456ea8bd8819502ebbabcf5664a7c2128a963821ff7983ffa02c4cda05d0ef812d94e26a0efff3d750901f0322b01857c1f30e33e1f9eeab643171477").unwrap();

//     println!("{:?}", raw_hex);

//     let vec = raw_hex.to_vec();
//     let mut slice = vec.as_slice();

//     // let tx = TransactionSigned::decode_enveloped(raw_hex.0).unwrap();
//     // println!("{:?}", tx);
// }

// #[tokio::test]
// async fn test_publish_block() {
//     let _ = tracing_subscriber::fmt::try_init();

//     let target = String::from("3.121.197.99:8080");
//     let client = Client::connect(target, String::from("6722c9f6-1edc-497f-ab7e-d4f70626e159"))
//         .await
//         .unwrap();

//     tokio::time::sleep(Duration::from_secs(1)).await;
//     let bytes = std::fs::read(
//         env!("CARGO_MANIFEST_DIR").to_string() + "/testdata/mainnetCapellaBlock7928030.bin.ssz",
//     )
//     .unwrap();

//     let res = client.publish_block(bytes).await.unwrap();

//     println!("{res:?}");
// }

#[tokio::test]
async fn test_send_transaction() {
    let _ = tracing_subscriber::fmt::try_init();
    let target = String::from("3.115.36.183:8080");
    let client = Client::connect(target, String::from("6722c9f6-1edc-497f-ab7e-d4f70626e159"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut tx = TxLegacy {
        to: TxKind::Call(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045")),
        value: U256::from(1_000_000_000),
        gas_limit: 2_000_000,
        nonce: 0,
        gas_price: 21_000_000_000,
        input: Bytes::new(),
        chain_id: Some(1),
    };

    println!("tx request: {:?}", tx);

    let wallet: LocalWallet = "5b2ba9904dfd28dc04fb37f5932b73bce4d2af43ef595c3316905335848b7d07"
        .parse()
        .unwrap();

    let sig = wallet.sign_transaction(&mut tx.clone()).await.unwrap();

    let signed = tx.rlp_signed(&sig);
    println!("Raw tx: {:?}", signed);

    // let vec = signed.to_vec();

    // let tx = TransactionSigned::decode(&mut vec.as_slice()).unwrap();
    // println!("Decoded: {:?}", tx);

    let res = client
        .send_raw_transaction_sequence(vec![signed.to_vec(), signed.to_vec()])
        .await
        .unwrap();

    println!("{:?}", res);
}

#[tokio::test]
async fn test_subscribe_transactions() {
    let _ = tracing_subscriber::fmt::try_init();
    let target = String::from("beta.fiberapi.io:8080");
    // let target = String::from("localhost:8080");
    let uncompressed_client =
        Client::connect(target.clone(), "6722c9f6-1edc-497f-ab7e-d4f70626e159")
            .await
            .unwrap();

    let compressed_client = Client::connect_with_options(
        target,
        "6722c9f6-1edc-497f-ab7e-d4f70626e159",
        ClientOptions::default().accept_compressed(true),
    )
    .await
    .unwrap();

    println!("connected to client");

    let mut un_sub = uncompressed_client.subscribe_new_txs(None).await;
    let mut com_sub = compressed_client.subscribe_new_txs(None).await;

    println!("listening to transactions");

    loop {
        tokio::select! {
            Some(tx) = un_sub.next() => {
                println!("{:?}", tx.hash());
            },
            // Some(tx) = com_sub.next() => {
            //     println!("{}", tx.hash());
            // }
        }
    }
}

#[tokio::test]
async fn test_subscribe_blocks() {
    let _ = tracing_subscriber::fmt::try_init();
    let target = String::from("3.122.250.155:8080");
    let uncompressed_client = Client::connect(
        target.clone(),
        String::from("6722c9f6-1edc-497f-ab7e-d4f70626e159"),
    )
    .await
    .unwrap();

    // let compressed_client = Client::connect_with_options(
    //     target,
    //     "6722c9f6-1edc-497f-ab7e-d4f70626e159",
    //     ClientOptions::default().accept_compressed(true),
    // )
    // .await
    // .unwrap();

    println!("connected to client");

    let mut un_sub = uncompressed_client.subscribe_new_execution_payloads().await;
    // let mut com_sub = compressed_client.subscribe_new_execution_payloads().await;

    println!("listening to blocks");

    loop {
        tokio::select! {
            Some(tx) = un_sub.next() => {
                // let utc = chrono::Utc::now();
                // let size = tx.encoded_len();
                // println!("U: {}: {:?} (size: {})", utc.to_rfc3339(), tx.header.unwrap().block_number, size);
            },
            // Some(tx) = com_sub.next() => {
            //     let size = tx.encoded_len();
            //     let utc = chrono::Utc::now();
            //     println!("C: {}: {:?} (size: {})", utc.to_rfc3339(), tx.header.unwrap().block_number, size);
            // }
        }
    }
}
