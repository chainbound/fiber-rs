use std::time::Duration;

use ethers::{
    types::{
        transaction::eip2930::{AccessList, AccessListItem},
        OtherFields, Transaction as EthersTx, U256,
    },
    utils::rlp::{Decodable, Rlp},
};
use pin_project::pin_project;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{codec::CompressionEncoding, transport::Channel, Request, Streaming};

pub mod api;
pub mod eth;
pub mod filter;
pub mod types;

use api::{
    api_client::ApiClient, BlockSubmissionMsg, BlockSubmissionResponse, RawTxMsg, RawTxSequenceMsg,
    TransactionResponse, TxFilter, TxSequenceMsg, TxSequenceResponse,
};
use eth::{CompactBeaconBlock, ExecutionPayload, ExecutionPayloadHeader, Transaction};

#[pin_project]
pub struct TxStream {
    #[pin]
    stream: Streaming<Transaction>,
}

impl TxStream {
    pub async fn next(&mut self) -> Option<EthersTx> {
        let proto = self.stream.message().await.unwrap_or(None)?;
        Some(proto_to_tx(proto))
    }
}

#[allow(clippy::large_enum_variant)]
pub enum SendType {
    Transaction {
        tx: Transaction,
        response: oneshot::Sender<TransactionResponse>,
    },
    RawTransaction {
        msg: RawTxMsg,
        response: oneshot::Sender<TransactionResponse>,
    },
    TransactionSequence {
        msg: TxSequenceMsg,
        response: oneshot::Sender<TxSequenceResponse>,
    },
    RawTransactionSequence {
        msg: RawTxSequenceMsg,
        response: oneshot::Sender<TxSequenceResponse>,
    },
    Block {
        msg: BlockSubmissionMsg,
        response: oneshot::Sender<BlockSubmissionResponse>,
    },
}

struct Dispatcher {
    cmd_rx: mpsc::UnboundedReceiver<SendType>,
    client: ApiClient<Channel>,
    api_key: String,
}

impl Dispatcher {
    async fn run(mut self) {
        let api_key = self.api_key.clone();
        let mut client = self.client;

        loop {
            // Set up the different streams
            let (new_tx_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the api key metadata
            append_api_key(&mut req, &api_key);

            let mut new_tx_responses = match client.send_transaction(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let (new_raw_tx_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the api key metadata
            append_api_key(&mut req, &api_key);

            let mut new_raw_tx_responses = match client.send_raw_transaction(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in raw transaction stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let (new_tx_seq_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the api key metadata
            append_api_key(&mut req, &api_key);

            let mut new_tx_seq_responses = match client.send_transaction_sequence(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in transaction sequence stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let (new_raw_tx_seq_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the api key metadata
            append_api_key(&mut req, &api_key);

            let mut new_raw_tx_seq_responses = match client.send_raw_transaction_sequence(req).await
            {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in raw transaction sequence stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let (new_block_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            append_api_key(&mut req, &api_key);

            let mut new_block_responses = match client.submit_block_stream(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in block submission stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            tracing::info!("All bi-directional streams established. Listening for commands...");

            while let Some(cmd) = self.cmd_rx.recv().await {
                match cmd {
                    SendType::Transaction { tx, response } => {
                        if new_tx_sender.send(tx).is_err() {
                            tracing::error!("Failed sending transaction");
                            break;
                        }

                        if let Some(res) = new_tx_responses.next().await {
                            match res {
                                Ok(res) => {
                                    let _ = response.send(res);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error in response stream, retrying...");
                                    break;
                                }
                            }
                        }
                    }
                    SendType::RawTransaction { msg, response } => {
                        if new_raw_tx_sender.send(msg).is_err() {
                            tracing::error!("Failed sending raw transaction");
                            break;
                        }

                        if let Some(res) = new_raw_tx_responses.next().await {
                            match res {
                                Ok(res) => {
                                    let _ = response.send(res);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error in response stream, retrying...");
                                    break;
                                }
                            }
                        }
                    }
                    SendType::TransactionSequence { msg, response } => {
                        if new_tx_seq_sender.send(msg).is_err() {
                            tracing::error!("Failed sending transaction sequence");
                            break;
                        }

                        if let Some(res) = new_tx_seq_responses.next().await {
                            match res {
                                Ok(res) => {
                                    let _ = response.send(res);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error in response stream, retrying...");
                                    break;
                                }
                            }
                        }
                    }
                    SendType::RawTransactionSequence { msg, response } => {
                        if new_raw_tx_seq_sender.send(msg).is_err() {
                            tracing::error!("Failed sending raw transaction sequence");
                            break;
                        }

                        if let Some(res) = new_raw_tx_seq_responses.next().await {
                            match res {
                                Ok(res) => {
                                    let _ = response.send(res);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error in response stream, retrying...");
                                    break;
                                }
                            }
                        }
                    }
                    SendType::Block { msg, response } => {
                        if new_block_sender.send(msg).is_err() {
                            tracing::error!("Failed sending block");
                            break;
                        }

                        if let Some(res) = new_block_responses.next().await {
                            match res {
                                Ok(res) => {
                                    let _ = response.send(res);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error in response stream, retrying...");
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

/// A client for interacting with the Fiber Network.
/// This wraps the inner [`ApiClient`] and provides a more ergonomic interface, as well as
/// automatic retries for streams.
pub struct Client {
    key: String,
    client: ApiClient<Channel>,
    cmd_tx: mpsc::UnboundedSender<SendType>,
}

#[derive(Debug, Default)]
pub struct ClientOptions {
    send_compressed: bool,
    accept_compressed: bool,
}

impl ClientOptions {
    /// Enables GZIP compression for outgoing data.
    pub fn send_compressed(mut self, send_compressed: bool) -> Self {
        self.send_compressed = send_compressed;
        self
    }

    /// Enables GZIP compression for incoming data.
    pub fn accept_compressed(mut self, accept_compressed: bool) -> Self {
        self.accept_compressed = accept_compressed;
        self
    }
}

/// Appends the api key metadata to the request, keyed by x-api-key.
fn append_api_key<T>(req: &mut Request<T>, key: &str) {
    req.metadata_mut().append("x-api-key", key.parse().unwrap());
}

impl Client {
    /// Connects to the given gRPC target with the API key, returning a [`Client`] instance.
    pub async fn connect(
        target: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        Self::connect_with_options(target, api_key, ClientOptions::default()).await
    }

    pub async fn connect_with_options(
        target: impl Into<String>,
        api_key: impl Into<String>,
        opts: ClientOptions,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let target = target.into();
        let api_key = api_key.into();

        let targetstr = if !target.starts_with("http://") {
            "http://".to_owned() + &target
        } else {
            target
        };

        // Set up the inner gRPC connection
        let mut client = ApiClient::connect(targetstr.to_owned()).await?;

        if opts.accept_compressed {
            client = client.accept_compressed(CompressionEncoding::Gzip);
        }

        if opts.send_compressed {
            client = client.send_compressed(CompressionEncoding::Gzip);
        }

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        // The dispatcher will handle request / response messages (like sending transactions)
        let dispatcher = Dispatcher {
            cmd_rx,
            api_key: api_key.clone(),
            client: client.clone(),
        };

        let client = Client {
            client,
            key: api_key,
            cmd_tx,
        };

        tokio::task::spawn(dispatcher.run());

        Ok(client)
    }

    /// Broadcasts a signed transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_transaction(
        &self,
        tx: EthersTx,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::Transaction {
            tx: tx_to_proto(tx),
            response: res,
        });

        let res = rx.await?;

        Ok((res.hash.to_owned(), res.timestamp))
    }

    /// Broadcasts a signed, RLP encoded transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_raw_transaction(
        &self,
        raw_tx: Vec<u8>,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::RawTransaction {
            msg: RawTxMsg { raw_tx },
            response: res,
        });

        let res = rx.await?;

        Ok((res.hash.to_owned(), res.timestamp))
    }

    /// Broadcasts a signed transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_transaction_sequence(
        &self,
        tx_sequence: Vec<EthersTx>,
    ) -> Result<(Vec<String>, i64), Box<dyn std::error::Error>> {
        let mut proto_txs = Vec::with_capacity(tx_sequence.len());

        for tx in tx_sequence {
            proto_txs.push(tx_to_proto(tx));
        }

        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::TransactionSequence {
            msg: TxSequenceMsg {
                sequence: proto_txs,
            },
            response: res,
        });

        let res = rx.await?;

        let timestamp = res.sequence_response[0].timestamp;
        let hashes = res
            .sequence_response
            .into_iter()
            .map(|resp| resp.hash)
            .collect();

        Ok((hashes, timestamp))
    }

    /// Broadcasts a signed, RLP encoded transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_raw_transaction_sequence(
        &self,
        raw_tx_sequence: Vec<Vec<u8>>,
    ) -> Result<(Vec<String>, i64), Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::RawTransactionSequence {
            msg: RawTxSequenceMsg {
                raw_txs: raw_tx_sequence,
            },
            response: res,
        });

        let res = rx.await?;

        let timestamp = res.sequence_response[0].timestamp;
        let hashes = res
            .sequence_response
            .into_iter()
            .map(|resp| resp.hash)
            .collect();

        Ok((hashes, timestamp))
    }

    /// Publish an SSZ encoded block to the Fiber Network. Returns [`BlockSubmissionResponse`] which
    /// contains information about the newly published block.
    pub async fn publish_block(
        &self,
        ssz_block: Vec<u8>,
    ) -> Result<BlockSubmissionResponse, Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::Block {
            msg: BlockSubmissionMsg { ssz_block },
            response: res,
        });

        Ok(rx.await?)
    }

    /// Subscribes to new transactions, returning an [`UnboundedReceiverStream`] of [`EthersTx`].
    /// Uses the given encoded filter to filter transactions. Note: the actual subscription takes place in
    /// the background. It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_txs(
        &self,
        filter: Option<Vec<u8>>,
    ) -> UnboundedReceiverStream<EthersTx> {
        let f = match filter {
            Some(encoded_filter) => TxFilter {
                encoded: encoded_filter,
            },
            None => TxFilter { encoded: vec![] },
        };

        let key = self.key.clone();

        let mut req = Request::new(f.clone());
        append_api_key(&mut req, &key);

        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(f.clone());
                append_api_key(&mut req, &key);

                let mut stream = match client.subscribe_new_txs(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Transaction stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(transaction) => {
                            let _ = tx.send(proto_to_tx(transaction));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new execution headers, returning an [`UnboundedReceiverStream`] of [`ExecutionPayloadHeader`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_execution_headers(
        &self,
    ) -> UnboundedReceiverStream<ExecutionPayloadHeader> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_api_key(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_api_key(&mut req, &key);

                let mut stream = match client.subscribe_execution_headers(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in execution header stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Execution header stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(header) => {
                            let _ = tx.send(header);
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in execution header stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new execution payloads, returning an [`UnboundedReceiverStream`] of [`ExecutionPayload`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_execution_payloads(
        &self,
    ) -> UnboundedReceiverStream<ExecutionPayload> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_api_key(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_api_key(&mut req, &key);

                let mut stream = match client.subscribe_execution_payloads(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in execution payload stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Execution payload stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(payload) => {
                            let _ = tx.send(payload);
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in execution payload stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        UnboundedReceiverStream::new(rx)
    }

    pub async fn subscribe_new_beacon_blocks(&self) -> UnboundedReceiverStream<CompactBeaconBlock> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_api_key(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_api_key(&mut req, &key);

                let mut stream = match client.subscribe_beacon_blocks(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in beacon block stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Beacon block stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(payload) => {
                            let _ = tx.send(payload);
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in beacon block stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        UnboundedReceiverStream::new(rx)
    }
}

fn tx_to_proto(tx: EthersTx) -> Transaction {
    let to = tx.to.map(|to| to.as_bytes().to_vec());

    let tx_type = match tx.transaction_type {
        Some(tp) => tp.as_u64(),
        None => 0,
    };

    let mut val_bytes = [0; 32];
    tx.value.to_big_endian(&mut val_bytes);

    let mut r_bytes = [0; 32];
    tx.r.to_big_endian(&mut r_bytes);

    let mut s_bytes = [0; 32];
    tx.s.to_big_endian(&mut s_bytes);

    let acl = match tx.access_list {
        Some(acl) => {
            let mut new_acl: Vec<eth::AccessTuple> = Vec::new();
            for tup in acl.0 {
                let mut new_keys: Vec<Vec<u8>> = Vec::new();

                for key in tup.storage_keys {
                    new_keys.push(key.as_bytes().to_vec())
                }

                new_acl.push(eth::AccessTuple {
                    address: tup.address.as_bytes().to_vec(),
                    storage_keys: new_keys,
                });
            }

            new_acl
        }
        _ => Vec::new(),
    };

    Transaction {
        to,
        gas: tx.gas.as_u64(),
        gas_price: tx.gas_price.unwrap_or(ethers::types::U256::zero()).as_u64(),
        hash: tx.hash.as_bytes().to_vec(),
        input: tx.input.to_vec(),
        nonce: tx.nonce.as_u64(),
        value: val_bytes.to_vec(),
        from: Some(tx.from.as_bytes().to_vec()),
        r#type: tx_type as u32,
        max_fee: tx
            .max_fee_per_gas
            .unwrap_or(ethers::types::U256::zero())
            .as_u64(),
        priority_fee: tx
            .max_priority_fee_per_gas
            .unwrap_or(ethers::types::U256::zero())
            .as_u64(),
        v: tx.v.as_u64(),
        r: r_bytes.to_vec(),
        s: s_bytes.to_vec(),
        chain_id: tx.chain_id.unwrap_or(ethers::types::U256::zero()).as_u32(),
        access_list: acl,
    }
}

fn proto_to_tx(proto: Transaction) -> EthersTx {
    let to = proto
        .to
        .map(|to| ethers::types::H160::from_slice(to.as_slice()));

    let tx_type: Option<ethers::types::U64> = match proto.r#type {
        1 => Some(1.into()),
        2 => Some(2.into()),
        _ => None,
    };
    let val = if proto.value.is_empty() {
        ethers::types::U256::zero()
    } else {
        ethers::types::U256::decode(&Rlp::new(&proto.value)).unwrap()
    };

    let r = ethers::types::U256::from_big_endian(proto.r.as_slice());
    let s = ethers::types::U256::from_big_endian(proto.s.as_slice());

    let gas_price: Option<U256> = if proto.gas_price == 0 {
        None
    } else {
        Some(proto.gas_price.into())
    };
    let max_fee: Option<U256> = if proto.max_fee == 0 {
        None
    } else {
        Some(proto.max_fee.into())
    };
    let priority_fee: Option<U256> = if proto.priority_fee == 0 {
        None
    } else {
        Some(proto.priority_fee.into())
    };

    let mut acl: Option<AccessList> = None;
    if !proto.access_list.is_empty() {
        let mut new_acl: Vec<AccessListItem> = Vec::new();

        for tup in proto.access_list {
            let mut keys: Vec<ethers::types::H256> = Vec::new();

            for key in tup.storage_keys {
                keys.push(ethers::types::H256::from_slice(key.as_slice()))
            }

            new_acl.push(AccessListItem {
                address: ethers::types::H160::from_slice(tup.address.as_slice()),
                storage_keys: keys,
            });
        }

        acl = Some(ethers::types::transaction::eip2930::AccessList(new_acl));
    }

    let v = if tx_type.is_some() {
        if proto.v > 1 {
            proto.v - 37
        } else {
            proto.v
        }
    } else {
        proto.v
    };

    let mut chain_id = Some(proto.chain_id.into());

    // If transaction is legacy (no type) and its v value is 27 or 28, we set chain ID to None.
    // This signifies a pre EIP-155 transaction.
    if tx_type.is_none() && proto.v < 37 {
        chain_id = None;
    }

    EthersTx {
        hash: ethers::types::H256::from_slice(proto.hash.as_slice()),
        nonce: proto.nonce.into(),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        from: ethers::types::H160::from_slice(proto.from.unwrap_or_default().as_slice()),
        to,
        value: val,
        gas_price,
        gas: proto.gas.into(),
        input: proto.input.into(),
        v: v.into(),
        r,
        s,
        transaction_type: tx_type,
        access_list: acl,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: max_fee,
        chain_id,
        other: OtherFields::default(),
    }
}

#[cfg(test)]
mod tests {
    use ethers::utils::rlp::{Decodable, Rlp};

    use super::*;

    #[test]
    fn should_convert_tx_to_proto() {
        let signed_rlp = hex::decode("02f864010314018261a894b94f5374fce5edbc8e2a8697c15331677e6ebf0b0a825544c001a0e4663a0f2ea882cf5b38ee63b375dd116d75153d7bbcff972aee6fe0ecc920fca050917b98425bd43a3bfdc4ecd2de4424d6b2b6b5fd55d0b34fc805db58d0224b").unwrap();

        let tx_rlp = Rlp::new(signed_rlp.as_slice());
        let tx = EthersTx::decode(&tx_rlp).unwrap();

        let proto = tx_to_proto(tx);

        println!("proto: {:?}", proto);
    }
}
