# `fiber-rs`
Rust client package for interacting with a Fiber Network API over gRPC.

## Installation
```bash
cargo add fiber-rs --git https://github.com/chainbound/fiber-rs
```

## Usage

### Connecting
```rs
use fiber::{Client};

#[tokio::main]
async fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();
}
```

### Subscriptions
#### Transactions
Subscribing to transactions will return an `async_stream.Stream`, yielding `ethers-rs.Transaction`
for every new transaction that's received.

**Example:**
```rs
use fiber::{Client};
use futures_util::{pin_mut, StreamExt};

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    let sub = client.subscribe_new_txs().await;
    // Pin the value on the stack
    pin_mut!(sub);

    // Use the stream as an async iterator
    while let Some(tx) = sub.next().await {
        handle_transaction(tx);
    }
}

```
#### Blocks
WIP

### Sending Transactions
#### `send_raw_transaction`
```rs
use ethers::{
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, Address, TransactionRequest}, utils::hex::ToHex,
};
use fiber::Client;

#[tokio::main]
asyn fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    let tx: TypedTransaction = TransactionRequest::new()
        .nonce(3)
        .gas_price(1)
        .gas(25000)
        .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b".parse::<Address>().unwrap())
        .value(10)
        .data(vec![0x55, 0x44])
        .chain_id(1)
        .into();

    let wallet: LocalWallet = "PRIVATE_KEY".parse().unwrap();

    let sig = wallet.sign_transaction(&tx.clone()).await.unwrap();

    let signed = tx.rlp_signed(&sig);

    let res = client.send_raw_transaction(&signed).await.unwrap();

    println!("{:?}", res);
}
```
#### `raw_backrun_transaction`
```rs
use ethers::{
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, Address, TransactionRequest}, utils::hex::ToHex,
};
use fiber::Client;

#[tokio::main]
asyn fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    let tx: TypedTransaction = TransactionRequest::new()
        .nonce(3)
        .gas_price(1)
        .gas(25000)
        .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b".parse::<Address>().unwrap())
        .value(10)
        .data(vec![0x55, 0x44])
        .chain_id(1)
        .into();

    let wallet: LocalWallet = "PRIVATE_KEY".parse().unwrap();

    let sig = wallet.sign_transaction(&tx.clone()).await.unwrap();

    let signed = tx.rlp_signed(&sig);

    let target_hash = "0x..."

    let res = client.raw_backrun_transaction(target_hash, &signed).await.unwrap();

    println!("{:?}", res);
}
```