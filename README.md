# `fiber-rs`
Rust client package for interacting with a Fiber Network API over gRPC.

## Installation
```bash
cargo add fiber-rs --git https://github.com/chainbound/fiber-rs
```

## Usage

### Connecting
```rs
use fiber_rs::{Client};

#[tokio::main]
async fn main() {
    let client = Client::connect("ENDPOINT_URL", "API_KEY").await.unwrap();
}
```

### Subscriptions
#### Transactions
Subscribing to transactions will return an `async_stream.Stream`, yielding `ethers-rs.Transaction`
for every new transaction that's received.

**Example:**
```rs
use fiber_rs::{Client};
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
There are 3 different endpoints for sending transactions:
* `send_transaction`
* `send_raw_transaction`
* `backrun_transaction`