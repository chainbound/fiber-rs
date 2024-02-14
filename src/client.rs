use std::time::Duration;

use alloy_rpc_engine_types::{
    ExecutionPayload, ExecutionPayloadV1, ExecutionPayloadV2, ExecutionPayloadV3,
};
use alloy_rpc_types::Block;
use ethereum_consensus::{ssz::prelude::deserialize, types::mainnet::SignedBeaconBlock};
use reth_primitives::{Address, Bytes, TransactionSigned, TransactionSignedEcRecovered};
use ssz::Decode;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream, StreamExt};
use tonic::{codec::CompressionEncoding, transport::Channel, Request};

use crate::generated::api::{
    api_client::ApiClient, BlockSubmissionMsg, BlockSubmissionResponse, TxFilter,
};
use crate::utils::{append_metadata, parse_execution_payload_to_block};
use crate::{Dispatcher, SendType};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
/// This wraps the inner [`ApiClient`] and provides a more ergonomic interface, as well as
/// automatic retries for streams.
#[derive(Debug)]
pub struct Client {
    key: String,
    client: ApiClient<Channel>,
    cmd_tx: mpsc::UnboundedSender<SendType>,
}

impl Client {
    /// Connects to the given gRPC target with the API key, returning a [`Client`] instance.
    pub async fn connect(target: impl Into<String>, api_key: impl Into<String>) -> Result<Client> {
        Self::connect_with_options(target, api_key, ClientOptions::default()).await
    }

    /// Connects to the given gRPC target with the API key and options, returning a [`Client`] instance.
    pub async fn connect_with_options(
        target: impl Into<String>,
        api_key: impl Into<String>,
        opts: ClientOptions,
    ) -> Result<Client> {
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
        };

        tokio::task::spawn(dispatcher.run());

        Ok(client)
    }

    /// Broadcasts a signed transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_transaction(&self, tx: TransactionSigned) -> Result<(String, i64)> {
        let (res, rx) = oneshot::channel();

        let _ = self
            .cmd_tx
            .send(SendType::Transaction { tx, response: res });

        let res = rx.await?;

        Ok((res.hash.to_owned(), res.timestamp))
    }

    /// Broadcasts a raw, RLP-encoded transaction to the Fiber Network. Returns hash and the timestamp
    /// of when the first node received the transaction.
    pub async fn send_raw_transaction(&self, raw_tx: Vec<u8>) -> Result<(String, i64)> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::RawTransaction {
            raw_tx,
            response: res,
        });

        let res = rx.await?;

        Ok((res.hash.to_owned(), res.timestamp))
    }

    /// Broadcasts a signed transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_transaction_sequence(
        &self,
        tx_sequence: Vec<TransactionSigned>,
    ) -> Result<(Vec<String>, i64)> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::TransactionSequence {
            msg: tx_sequence,
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

    /// Broadcasts a raw, RLP-encoded transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_raw_transaction_sequence(
        &self,
        tx_sequence: Vec<Vec<u8>>,
    ) -> Result<(Vec<String>, i64)> {
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
            .map(|resp| resp.hash)
            .collect();

        Ok((hashes, timestamp))
    }

    /// Publish an SSZ encoded block to the Fiber Network. Returns [`BlockSubmissionResponse`] which
    /// contains information about the newly published block.
    pub async fn publish_block(&self, ssz_block: Vec<u8>) -> Result<BlockSubmissionResponse> {
        let (res, rx) = oneshot::channel();

        let _ = self.cmd_tx.send(SendType::Block {
            msg: BlockSubmissionMsg { ssz_block },
            response: res,
        });

        Ok(rx.await?)
    }

    /// Subscribes to new transactions, returning a [`Stream`] of [`TransactionSignedEcRecovered`].
    /// Uses the given encoded filter to filter transactions. Note: the actual subscription takes place in
    /// the background. It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_transactions(
        &self,
        filter: Option<Vec<u8>>,
    ) -> impl Stream<Item = TransactionSignedEcRecovered> {
        let f = match filter {
            Some(encoded_filter) => TxFilter {
                encoded: encoded_filter,
            },
            None => TxFilter { encoded: vec![] },
        };

        let key = self.key.clone();

        let mut req = Request::new(f.clone());
        append_metadata(&mut req, &key);

        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(f.clone());
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
                            // let transaction_encoded = Vec::new();
                            let signer = match Address::try_from(transaction.sender.as_slice()) {
                                Ok(sender) => sender,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing sender");
                                    continue;
                                }
                            };
                            let signed_transaction = match TransactionSigned::decode_enveloped(
                                &mut transaction.rlp_transaction.as_slice(),
                            ) {
                                Ok(tx) => tx,
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing transaction");
                                    continue;
                                }
                            };
                            let _ = tx.send(TransactionSignedEcRecovered::from_signed_transaction(
                                signed_transaction,
                                signer,
                            ));
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

    /// Subscribes to new raw transactions, returning a [`Stream`] of [`(Address, Bytes)`].
    /// Uses the given encoded filter to filter transactions. Note: the actual subscription takes place in
    /// the background. It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_raw_transactions(
        &self,
        filter: Option<Vec<u8>>,
    ) -> impl Stream<Item = (Address, Bytes)> {
        let f = match filter {
            Some(encoded_filter) => TxFilter {
                encoded: encoded_filter,
            },
            None => TxFilter { encoded: vec![] },
        };

        let key = self.key.clone();

        let mut req = Request::new(f.clone());
        append_metadata(&mut req, &key);

        let mut client = self.client.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(f.clone());
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
    pub async fn subscribe_new_execution_payloads(&self) -> impl Stream<Item = Block> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_metadata(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
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

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new beacon blocks, returning a [`Stream`] of [`SignedBeaconBlock`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_beacon_blocks(&self) -> impl Stream<Item = SignedBeaconBlock> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_metadata(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
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

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new beacon blocks, returning a [`Stream`] of raw [`Bytes`].
    /// Note: the actual subscription takes place in the background.
    /// It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_raw_beacon_blocks(&self) -> impl Stream<Item = Bytes> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_metadata(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
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

        UnboundedReceiverStream::new(rx)
    }
}
