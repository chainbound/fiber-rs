use std::time::Duration;

use ethereum_consensus::{
    ssz::prelude::deserialize,
    types::mainnet::{ExecutionPayload, ExecutionPayloadHeader, SignedBeaconBlock},
};
use reth_primitives::{Address, TransactionSigned, TransactionSignedEcRecovered};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{codec::CompressionEncoding, transport::Channel, Request};

pub mod api;
pub mod filter;
pub mod types;

use api::{
    api_client::ApiClient, BlockSubmissionMsg, BlockSubmissionResponse, TransactionResponse,
    TxFilter, TxSequenceMsgV2, TxSequenceResponse,
};

use crate::api::TransactionMsg;

#[allow(clippy::large_enum_variant)]
pub enum SendType {
    Transaction {
        tx: TransactionSigned,
        response: oneshot::Sender<TransactionResponse>,
    },
    TransactionSequence {
        msg: TxSequenceMsgV2,
        response: oneshot::Sender<TxSequenceResponse>,
    },
    Block {
        msg: BlockSubmissionMsg,
        response: oneshot::Sender<BlockSubmissionResponse>,
    },
}

/// The dispatcher is responsible of handling request / response messages (like sending transactions)
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

            let mut new_tx_responses = match client.send_transaction_v2(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in transaction stream, retrying...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let (new_tx_seq_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the api key metadata
            append_api_key(&mut req, &api_key);

            let mut new_tx_seq_responses = match client.send_transaction_sequence_v2(req).await {
                Ok(stream) => stream.into_inner(),
                Err(e) => {
                    tracing::error!(error = ?e, "Error in transaction sequence stream, retrying...");
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
                        let mut rlp_transaction: Vec<u8> = Vec::new();
                        tx.encode_enveloped(&mut rlp_transaction);

                        if new_tx_sender
                            .send(TransactionMsg { rlp_transaction })
                            .is_err()
                        {
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
        tx: TransactionSigned,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let _ = self
            .cmd_tx
            .send(SendType::Transaction { tx, response: res });

        let res = rx.await?;

        Ok((res.hash.to_owned(), res.timestamp))
    }

    /// Broadcasts a signed transaction sequence to the Fiber Network. Returns the array of hashes and
    /// the timestamp of when the first node received the sequence.
    pub async fn send_transaction_sequence(
        &self,
        tx_sequence: Vec<TransactionSigned>,
    ) -> Result<(Vec<String>, i64), Box<dyn std::error::Error>> {
        let (res, rx) = oneshot::channel();

        let sequence: Vec<Vec<u8>> = tx_sequence
            .iter()
            .map(|tx| {
                let mut buf = Vec::new();
                tx.encode_enveloped(&mut buf);
                buf
            })
            .collect();

        let _ = self.cmd_tx.send(SendType::TransactionSequence {
            msg: TxSequenceMsgV2 { sequence },
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

    /// Subscribes to new transactions, returning an [`UnboundedReceiverStream`] of [`RethTx`].
    /// Uses the given encoded filter to filter transactions. Note: the actual subscription takes place in
    /// the background. It will automatically retry every 2s if the stream fails.
    pub async fn subscribe_new_txs(
        &self,
        filter: Option<Vec<u8>>,
    ) -> UnboundedReceiverStream<TransactionSignedEcRecovered> {
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

                let mut stream = match client.subscribe_execution_payloads_v2(req).await {
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
                            match deserialize::<ExecutionPayloadHeader>(&header.ssz_payload) {
                                Ok(header_deserialized) => {
                                    let _ = tx.send(header_deserialized);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing execution header");
                                    continue;
                                }
                            }
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
                            match deserialize::<ExecutionPayload>(&payload.ssz_payload) {
                                Ok(payload_deserialized) => {
                                    let _ = tx.send(payload_deserialized);
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Error deserializing execution payload");
                                    continue;
                                }
                            }
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

    pub async fn subscribe_new_beacon_blocks(&self) -> UnboundedReceiverStream<SignedBeaconBlock> {
        let key = self.key.clone();

        let mut req = Request::new(());
        append_api_key(&mut req, &key);

        let mut client = self.client.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let mut req = Request::new(());
                append_api_key(&mut req, &key);

                let mut stream = match client.subscribe_empty_beacon_blocks(req).await {
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
}
