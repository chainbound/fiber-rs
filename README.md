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

Subscribing to transactions will return a `Stream`, yielding [`reth_primitives::TransactionSignedEcRecovered`](https://github.com/paradigmxyz/reth/blob/0e166f0f326b86491c0b23a8cc483e8a224e9731/crates/primitives/src/transaction/mod.rs#L1474)
for every new transaction that's received.

**Example:**

```rs
use fiber::Client;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_transactions(None).await;

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
use tokio_stream::StreamExt;

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
    let mut sub = client.subscribe_new_transactions(f.encode().unwrap()).await;

    // Use the stream as an async iterator
    while let Some(tx) = sub.next().await {
        handle_transaction(tx);
    }
}
```

#### Raw transactions

Subscribing to raw transactions will return a `Stream`, yielding raw RLP encoded transactions.

**Example:**

```rs
use fiber::Client;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_raw_transactions(None).await;

    // Use the stream as an async iterator
    while let Some(raw_tx) = sub.next().await {
        handle_raw_transaction(tx);
    }
}
```

#### Execution Payloads (new block headers + transactions)

Returns a stream of newly seen execution payloads. This is useful for getting the state updates of a
newly confirmed block. An `ExecutionPayload` contains both the block header and the transactions
that were executed in that block.

The type returned by this stream is an [`alloy-rpc-types::Block`](https://github.com/alloy-rs/alloy/blob/a4453d42ffb755a46bace2ceca3baa454e0cd807/crates/rpc-types/src/eth/block.rs#L18). Since the blocks returned are parsed from consensus-layer payloads, they are missing the following fields, which are set to `None` or `zero` in all returned stream items:

- `parent_beacon_block_root`
- `transactions_root`
- `withdrawals_root`

**Example:**

```rs
use fiber::Client;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_execution_payloads().await;

    // Use the stream as an async iterator
    while let Some(block) = sub.next().await {
        handle_block(block);
    }
}
```

#### Beacon Blocks

Returns a stream of consensus-layer [`SignedBeaconBlock`](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock) objects.

**Example:**

```rs
use fiber::Client;
use tokio_stream::StreamExt;

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

#### Raw beacon blocks

Returns a stream of raw SSZ-encoded beacon blocks. Decoding is left to the caller to handle.

**Example:**

```rs
use fiber::Client;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Client needs to be mutable
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // No filter in this example
    let mut sub = client.subscribe_new_raw_beacon_blocks().await;

    // Use the stream as an async iterator
    while let Some(raw_block) = sub.next().await {
        handle_raw_beacon_block(raw_block);
    }
}
```

### Sending Transactions

#### `send_transaction`

Allows to broadcast transactions quickly to the Fiber network. It expects a [`reth_primitives::TransactionSigned`](https://github.com/paradigmxyz/reth/blob/0e166f0f326b86491c0b23a8cc483e8a224e9731/crates/primitives/src/transaction/mod.rs#L947) object, and returns the transaction hash and the timestamp of when the first Fiber node received it.

```rs
use reth_primitives::TransactionSigned;
use fiber::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // Create a transaction and a signature for it
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

    let res = client.send_transaction(signed_tx).await.unwrap();

    println!("{:?}", res);
}
```

#### `send_transaction_sequence`

Sends a sequence of transactions to the Fiber network at once.

```rs
use reth_primitives::{Transaction, TxEip1559, Address, TransactionKind, AccessList, Signature, U256, TransactionSigned};
use std::str::FromStr;
use fiber::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // Provide a transaction with signature as the first transaction of the sequence
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

    let signed_tx_1 = TransactionSigned::from_transaction_and_signature(tx, sig);

    // Then, provide a second signed transaction from wherever you want
    let tx_bytes = hex::decode("02f872018307910d808507204d2cb1827d0094388c818ca8b9251b393131c08a736a67ccb19297880320d04823e2701c80c001a0cf024f4815304df2867a1a74e9d2707b6abda0337d2d54a4438d453f4160f190a07ac0e6b3bc9395b5b9c8b9e6d77204a236577a5b18467b9175c01de4faa208d9").unwrap();
    let signed_tx_2 = TransactionSigned::decode_enveloped(&mut &tx_bytes[..]).unwrap();

    let res = client.send_transaction_sequence(vec![signed_tx_1, signed_tx_2]).await.unwrap();

    println!("{:?}", res);
}
```

#### `send_raw_transaction`

Sends a raw RLP encoded transaction to the Fiber network.

```rs
use fiber::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    let raw_tx = hex::decode("02f872018307910d808507204d2cb1827d0094388c818ca8b9251b393131c08a736a67ccb19297880320d04823e2701c80c001a0cf024f4815304df2867a1a74e9d2707b6abda0337d2d54a4438d453f4160f190a07ac0e6b3bc9395b5b9c8b9e6d77204a236577a5b18467b9175c01de4faa208d9").unwrap();

    let res = client.send_raw_transaction(raw_tx).await.unwrap();

    println!("{:?}", res);
}
```

#### `send_raw_transaction_sequence`

Sends a sequence of RLP encoded transactions, for things like backrunning and other strategies
where the explicitly stated order is important.

```rs
use fiber::Client;

#[tokio::main]
asyn fn main() {
    let mut client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();

    // Our transaction, RLP-encoded
    let raw_tx_1 = hex::decode("02f872018307910d808507204d2cb1827d0094388c818ca8b9251b393131c08a736a67ccb19297880320d04823e2701c80c001a0cf024f4815304df2867a1a74e9d2707b6abda0337d2d54a4438d453f4160f190a07ac0e6b3bc9395b5b9c8b9e6d77204a236577a5b18467b9175c01de4faa208d9").unwrap();

    // The target transaction should be an RLP encoded transaction as well
    let target_tx_2: Vec<u8> = vec![...]

    let res = client.send_raw_transaction_sequence(vec![raw_tx_1, target_tx_2]).await.unwrap();

    println!("{:?}", res);
}
```
