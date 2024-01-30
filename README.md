# `fiber-rs`
Rust client package for interacting with a Fiber Network API over gRPC.

## Installation
```bash
cargo add fiber --git https://github.com/chainbound/fiber-rs
```

## Usage
`fiber-rs` prints traces to the `fiber` target. To see them, run your program with `RUST_LOG=fiber=info`.

### Connecting

Using default options:
```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();
}
```

Specifying options:
```rs
use fiber::{Client, ClientOptions};

#[tokio::main]
async fn main() {
    let opts = ClientOptions::default().send_compressed(true);
    let mut client = Client::connect_with_options("ENDPOINT_URL", "API_KEY", opts).await.unwrap();
}
```

### Performance & Compression
With `ClientOptions` you can negotiate incoming and outgoing stream compression. This activates gzip compression on the underlying gRPC connections. Compression is not always faster, and here are some general guidelines to follow:
- If your client is close to the target Fiber node and you have enough bandwidth, it's best to disable `accept` compression.
- If you client is far from the target Fiber node and you have limited bandwidth, it's best to enable `accept` compression.

### Subscriptions
All subscriptions return a `Stream` (`UnboundedReceiverStream`) of the specified type.
If the underlying gRPC stream fails due to connection issues, it will automatically be retried.

#### Transactions
Subscribing to transactions will return a `Stream`, yielding [`ethers::types::Transaction`](https://docs.rs/ethers/latest/ethers/types/struct.Transaction.html)
for every new transaction that's received.

**Example:**
```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_txs(None).await;

    // Use the stream as an async iterator
    while let Some(tx) = sub.next().await {
        handle_transaction(tx);
    }
}

```

#### Filtering
You can apply filters to the transaction stream using `fiber::filter::Filter`. The builder pattern is used
for constructing a filter, with a couple of examples below.
```rs
use fiber::{Client, filter::FilterBuilder};

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // Construct filter
    // example 1: simple receiver filter
    let f = FilterBuilder::new()
                .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");

    // example 2: all transactions with either of these addresses as the receiver
    let f = FilterBuilder::new()
                .or() // creates a new 'OR' level
                  .to("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
                  .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");

    // example 3: all ERC20 transfers on the 2 tokens below
    let f = FilterBuilder::new()
                .and()
                  .method_id("0xa9059cbb")
                  .or()
                    .to("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
                    .to("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    // Encode the filter
    let mut sub = client.subscribe_new_txs(f.encode().unwrap()).await;

    // Use the stream as an async iterator
    while let Some(tx) = sub.next().await {
        handle_transaction(tx);
    }
}
```

#### Execution Payloads (new block headers + transactions)
Returns a stream of newly seen execution payloads. This is useful for getting the state updates of a
newly confirmed block. An `ExecutionPayload` contains both the block header and the transactions
that were executed in that block.

**Example:**
```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_execution_payloads().await;

    // Use the stream as an async iterator
    while let Some(block) = sub.next().await {
        handle_block(tx);
    }
}
```

#### Execution Headers (new block headers only)
Returns a stream of newly seen execution headers. This is useful for updating the state root and other
block metadata without having to fetch all the transactions, which offers a latency improvement.

**Example:**
```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_execution_headers().await;

    // Use the stream as an async iterator
    while let Some(header) = sub.next().await {
        handle_block_header(header);
    }
}
```

#### Beacon Blocks
Returns a stream of consensus-layer [`BeaconBlock`](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock) objects. NOTE: the `ExecutionPayload` is not included in this stream, please use the `subscribe_new_execution_payloads` stream if you need it.

**Example:**
```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_beacon_blocks().await;

    // Use the stream as an async iterator
    while let Some(block) = sub.next().await {
        handle_beacon_block(block);
    }
}
```


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
#### `send_raw_transaction_sequence`
Sends a sequence of RLP encoded transactions, for things like backrunning and other strategies
where the explicitly stated order is important.
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

    // The target transaction should be an RLP encoded transaction as well
    let target_tx: Vec<u8> = vec![...]

    let res = client.send_raw_transaction_sequence(vec![target_tx, signed]).await.unwrap();

    println!("{:?}", res);
}
```
