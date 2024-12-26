use std::{
    fmt::Debug,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use alloy::{
    consensus::{
        transaction::{PooledTransaction, Recovered},
        Block, Signed, TxEip4844WithSidecar, TxEnvelope,
    },
    eips::eip2718::Decodable2718,
    hex::FromHexError,
    primitives::{Address, Bytes, B256},
    rpc::types::engine::{
        ExecutionPayload, ExecutionPayloadV1, ExecutionPayloadV2, ExecutionPayloadV3,
    },
};
use ethereum_consensus::{ssz::prelude::deserialize, types::mainnet::SignedBeaconBlock};
use ssz::Decode;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream, StreamExt};
use tonic::{codec::CompressionEncoding, transport::Channel, Request};

use crate::generated::api::{
    api_client::ApiClient, BlockSubmissionMsg, BlockSubmissionResponse, TxFilter,
};
use crate::utils::{append_metadata, parse_execution_payload_to_block};
use crate::{Dispatcher, SendType};

type FiberResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const BELLATRIX_DATA_VERSION: u32 = 3;
const CAPELLA_DATA_VERSION: u32 = 4;
const DENEB_DATA_VERSION: u32 = 5;

/// Options for the API client
#[derive(Debug, Default, Clone, Copy)]
pub struct ClientOptions {
    send_compressed: bool,
    accept_compressed: bool,
}

impl ClientOptions {
    /// Enables GZIP compression for outgoing data.
    pub const fn send_compressed(mut self, send_compressed: bool) -> Self {
        self.send_compressed = send_compressed;
        self
    }

    /// Enables GZIP compression for incoming data.
    pub const fn accept_compressed(mut self, accept_compressed: bool) -> Self {
        self.accept_compressed = accept_compressed;
        self
    }
}

/// A client for interacting with the Fiber Network.
///
/// This wraps the inner [`ApiClient`] and provides a more ergonomic interface, as well as
/// automatic retries for streams.
pub struct Client {
    key: String,
    client: ApiClient<Channel>,
    cmd_tx: mpsc::UnboundedSender<SendType>,
    background_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("key", &"********")
            .field("client", &self.client)
            .finish()
    }
}

impl Client {
    /// Connects to the given gRPC target with the API key, returning a [`Client`] instance.
    pub async fn connect(
        target: impl Into<String>,
        api_key: impl Into<String>,
    ) -> FiberResult<Client> {
        Self::connect_with_options(target, api_key, ClientOptions::default()).await
    }

    /// Connects to the given gRPC target with the API key and options, returning a [`Client`] instance.
    pub async fn connect_with_options(
        target: impl Into<String>,
        api_key: impl Into<String>,
        opts: ClientOptions,
    ) -> FiberResult<Client> {
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

        let dispatcher = Dispatcher {
            cmd_rx,
            api_key: api_key.clone(),
            client: client.clone(),
        };

        let client = Client {
            client,
            key: api_key,
            cmd_tx,
            background_tasks: Arc::new(Mutex::new(Vec::new())),
        };

        tokio::task::spawn(dispatcher.run());

        Ok(client)
    }

    /// Kills all background tasks spawned by the client.
    ///
    /// This is useful when you want to stop all background tasks and clean up resources.
    /// Note that this will abort all existing streams on this client instance.
    pub fn kill_all_background_tasks(&self) {
        for task in self.background_tasks.lock().unwrap().drain(..) {
            task.abort();
        }
    }

    /// Broadcasts a signed transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_transaction(&self, tx: TxEnvelope) -> FiberResult<(B256, i64)> {
        let (response, rx) = oneshot::channel();

        let cmd = SendType::Transaction { tx, response };
        let _ = self.cmd_tx.send(cmd);

        let res = rx.await?;

        Ok((B256::from_str(&res.hash)?, res.timestamp))
    }

    /// Broadcasts a raw, RLP-encoded transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_raw_transaction(&self, raw_tx: Vec<u8>) -> FiberResult<(B256, i64)> {
        let (response, rx) = oneshot::channel();

        let cmd = SendType::RawTransaction { raw_tx, response };
        let _ = self.cmd_tx.send(cmd);

        let res = rx.await?;

        Ok((B256::from_str(&res.hash)?, res.timestamp))
    }

    /// Broadcasts a signed transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_transaction_sequence(
        &self,
        tx_sequence: Vec<TxEnvelope>,
    ) -> FiberResult<(Vec<B256>, i64)> {
        let (response, rx) = oneshot::channel();

        let cmd = SendType::TransactionSequence {
            msg: tx_sequence,
            response,
        };
        let _ = self.cmd_tx.send(cmd);

        let res = rx.await?;

        let timestamp = res.sequence_response[0].timestamp;
        let hashes = res
            .sequence_response
            .into_iter()
            .map(|resp| B256::from_str(&resp.hash))
            .collect::<Result<Vec<B256>, FromHexError>>()
            .map_err(|e| e.to_string())?;

        Ok((hashes, timestamp))
    }

    /// Broadcasts a raw, RLP-encoded transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_raw_transaction_sequence(
        &self,
        tx_sequence: Vec<Vec<u8>>,
    ) -> FiberResult<(Vec<B256>, i64)> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::RawTransactionSequence {
            raw_txs: tx_sequence,
            response: res,
        });

        let res = rx.await?;

        let timestamp = res.sequence_response[0].timestamp;
        let hashes = res
            .sequence_response
            .into_iter()
            .map(|resp| B256::from_str(&resp.hash))
            .collect::<Result<Vec<B256>, FromHexError>>()
            .map_err(|e| e.to_string())?;

        Ok((hashes, timestamp))
    }

    /// Publish an SSZ encoded block to the Fiber Network. Returns [`BlockSubmissionResponse`] which
    /// contains information about the newly published block.
    pub async fn publish_block(&self, ssz_block: Vec<u8>) -> FiberResult<BlockSubmissionResponse> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::Block {
            msg: BlockSubmissionMsg { ssz_block },
            response: res,
        });

        Ok(rx.await?)
    }

    /// Subscribes to new transactions, returning a [`Stream`] of [`RecoveredTx<TxEnvelope>`].
    /// Uses the given encoded filter to filter transactions.
    ///
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    ///
    /// Note: don't call this method multiple times, as it will create multiple background tasks
    /// that will compete for the same stream, resulting in spamming connection requests. If you
    /// really need to manually restart the stream, call [`Self::kill_all_background_tasks`] first.
    pub async fn subscribe_new_transactions(
        &self,
        filter: Option<Vec<u8>>,
    ) -> impl Stream<Item = Recovered<TxEnvelope>> {
        let filter = match filter {
            Some(encoded_filter) => TxFilter {
                encoded: encoded_filter,
            },
            None => TxFilter { encoded: vec![] },
        };

        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(filter.clone());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_new_txs_v2(req).await {
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
                            let signer = match Address::try_from(transaction.sender.as_slice()) {
                                Ok(sender) => sender,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing sender");
                                    continue;
                                }
                            };

                            let signed_transaction = match TxEnvelope::decode_2718(
                                &mut transaction.rlp_transaction.as_slice(),
                            ) {
                                Ok(tx) => tx,
                                Err(e) => {
                                    // Note: In case we receive a transaction in its network protocol
                                    // encoding, we strip the blob out and try to decode it again.
                                    if e.to_string() == "unexpected list" {
                                        tracing::debug!("Received blob transaction in network protocol encoding");

                                        match PooledTransaction::decode_2718(
                                            &mut transaction.rlp_transaction.as_ref(),
                                        ) {
                                            Ok(pooled) => pooled.into_envelope(),
                                            Err(e) => {
                                                tracing::error!(error = ?e, "Error deserializing blob transaction");
                                                continue;
                                            }
                                        }
                                    } else {
                                        tracing::error!(error = ?e, "Error deserializing transaction");
                                        continue;
                                    }
                                }
                            };

                            tracing::trace!(hash = ?signed_transaction.tx_hash(), "Received transaction");
                            let _ = tx.send(Recovered::new_unchecked(signed_transaction, signer));
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

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new raw transactions, returning a [`Stream`] of [`(Address, Bytes)`].
    /// Uses the given encoded filter to filter transactions. Note: the actual subscription takes place in
    /// the background. It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_raw_transactions(
        &self,
        filter: Option<Vec<u8>>,
    ) -> impl Stream<Item = (Address, Bytes)> {
        let filter = match filter {
            Some(encoded_filter) => TxFilter {
                encoded: encoded_filter,
            },
            None => TxFilter { encoded: vec![] },
        };

        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(filter.clone());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_new_txs_v2(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Raw transaction stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(transaction) => {
                            let sender = match Address::try_from(transaction.sender.as_slice()) {
                                Ok(sender) => sender,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing sender");
                                    continue;
                                }
                            };
                            let _ = tx.send((sender, transaction.rlp_transaction.into()));
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

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new blob transactions, returning a [`Stream`] of [`BlobTransactionSignedEcRecovered`].
    /// Note: the actual subscription takes place in the background. It will automatically retry
    /// every 2s if the stream fails.
    pub async fn subscribe_new_blob_transactions(
        &self,
    ) -> impl Stream<Item = Recovered<Signed<TxEip4844WithSidecar>>> {
        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_new_blob_txs(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Blob transaction stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(transaction) => {
                            let signer = match Address::try_from(transaction.sender.as_slice()) {
                                Ok(sender) => sender,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing sender");
                                    continue;
                                }
                            };

                            let pooled = match PooledTransaction::decode_2718(
                                &mut transaction.rlp_transaction.as_ref(),
                            ) {
                                Ok(pooled) => pooled,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing blob transaction");
                                    continue;
                                }
                            };
                            let blob_tx = match pooled {
                                PooledTransaction::Eip4844(blob_tx) => blob_tx,
                                _ => {
                                    tracing::error!(
                                        "Wrong transaction type for blob transaction stream"
                                    );
                                    continue;
                                }
                            };

                            tracing::trace!("Received blob transaction");
                            let _ = tx.send(Recovered::new_unchecked(blob_tx, signer));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in blob transaction stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new raw blob transactions, returning a [`Stream`] of [`(Address, Bytes)`].
    /// Note:
    /// * transactions are returned with the "raw format" `type || rlp([tx_payload_body, blobs,
    ///   commitments, proofs])` compatible with the `eth_sendRawTransaction` RPC method.
    /// * the actual subscription takes place in the background. It will
    ///   automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_raw_blob_transactions(
        &self,
    ) -> impl Stream<Item = (Address, Bytes)> {
        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_new_blob_txs(req).await {
                    Ok(stream) => stream.into_inner(),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                tracing::info!("Raw blob transaction stream established");

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(transaction) => {
                            let signer = match Address::try_from(transaction.sender.as_slice()) {
                                Ok(sender) => sender,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing sender");
                                    continue;
                                }
                            };

                            let _ = tx.send((signer, transaction.rlp_transaction.into()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in blob transaction stream, retrying...");
                            // If we get an error, we set the inner stream to None and break out of the loop.
                            // Next iteration will retry the stream.
                            break;
                        }
                    }
                }
            }
        });

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new execution payloads, returning a [`Stream`] of [`Block`]s.
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    ///
    /// Since the blocks returned are parsed from consensus-layer payloads, they are missing
    /// the following fields, which are set to `None` or `zero` in all stream items:
    /// - `parent_beacon_block_root`
    /// - `transactions_root`
    /// - `withdrawals_root`
    pub async fn subscribe_new_execution_payloads(&self) -> impl Stream<Item = Block<TxEnvelope>> {
        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_execution_payloads_v2(req).await {
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
                            let payload_deserialized = match payload.data_version {
                                BELLATRIX_DATA_VERSION => {
                                    ExecutionPayloadV1::from_ssz_bytes(&payload.ssz_payload)
                                        .map(ExecutionPayload::V1)
                                }
                                CAPELLA_DATA_VERSION => {
                                    ExecutionPayloadV2::from_ssz_bytes(&payload.ssz_payload)
                                        .map(ExecutionPayload::V2)
                                }
                                DENEB_DATA_VERSION => {
                                    ExecutionPayloadV3::from_ssz_bytes(&payload.ssz_payload)
                                        .map(ExecutionPayload::V3)
                                }
                                _ => {
                                    tracing::error!(
                                        data_version = payload.data_version,
                                        "Error deserializing execution payload: invalid data version"
                                    );
                                    continue;
                                }
                            };

                            let execution_payload = match payload_deserialized {
                                Ok(payload) => payload,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing execution payload");
                                    continue;
                                }
                            };

                            // Parse an ExecutionPayload into a Block
                            let block = parse_execution_payload_to_block(execution_payload);

                            let _ = tx.send(block);
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

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new beacon blocks, returning a [`Stream`] of [`SignedBeaconBlock`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_beacon_blocks(&self) -> impl Stream<Item = SignedBeaconBlock> {
        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_beacon_blocks_v2(req).await {
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
                        Ok(payload) => match deserialize::<SignedBeaconBlock>(&payload.ssz_block) {
                            Ok(payload_deserialized) => {
                                let _ = tx.send(payload_deserialized);
                            }
                            Err(e) => {
                                tracing::error!(error = ?e, "Error deserializing beacon block");
                                continue;
                            }
                        },
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

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new beacon blocks, returning a [`Stream`] of raw [`Bytes`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_raw_beacon_blocks(&self) -> impl Stream<Item = Bytes> {
        let key = self.key.clone();
        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_metadata(&mut req, &key);

                let mut stream = match client.subscribe_beacon_blocks_v2(req).await {
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
                        Ok(item) => {
                            let _ = tx.send(item.ssz_block.into());
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

        self.background_tasks.lock().unwrap().push(handle);

        UnboundedReceiverStream::new(rx)
    }
}
