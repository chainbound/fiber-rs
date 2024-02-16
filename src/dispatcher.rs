use std::time::Duration;

use reth_primitives::TransactionSigned;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{transport::Channel, Request};

use crate::{
    generated::api::{
        api_client::ApiClient, BlockSubmissionMsg, BlockSubmissionResponse, TransactionMsg,
        TransactionResponse, TxSequenceMsgV2, TxSequenceResponse,
    },
    utils::append_metadata,
};

/// The different types of messages that can be sent to the dispatcher
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[allow(missing_docs)]
pub enum SendType {
    Transaction {
        tx: TransactionSigned,
        response: oneshot::Sender<TransactionResponse>,
    },
    RawTransaction {
        raw_tx: Vec<u8>,
        response: oneshot::Sender<TransactionResponse>,
    },
    TransactionSequence {
        msg: Vec<TransactionSigned>,
        response: oneshot::Sender<TxSequenceResponse>,
    },
    RawTransactionSequence {
        raw_txs: Vec<Vec<u8>>,
        response: oneshot::Sender<TxSequenceResponse>,
    },
    Block {
        msg: BlockSubmissionMsg,
        response: oneshot::Sender<BlockSubmissionResponse>,
    },
}

/// The dispatcher is responsible of handling request / response messages (like sending transactions)
#[derive(Debug)]
pub struct Dispatcher {
    /// The receiver for the different types of messages that can be sent to the dispatcher
    pub cmd_rx: mpsc::UnboundedReceiver<SendType>,
    /// The API client
    pub client: ApiClient<Channel>,
    /// The API key
    pub api_key: String,
}

impl Dispatcher {
    /// Consumes the dispatcher and runs the main loop in a background task.
    pub async fn run(mut self) {
        let api_key = self.api_key.clone();
        let mut client = self.client;

        loop {
            // Set up the different streams
            let (new_tx_sender, rx) = mpsc::unbounded_channel();
            let rx_stream = UnboundedReceiverStream::new(rx);

            let mut req = Request::new(rx_stream);
            // Append the request metadata
            append_metadata(&mut req, &api_key);

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
            // Append the request metadata
            append_metadata(&mut req, &api_key);

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
            // Append the request metadata
            append_metadata(&mut req, &api_key);

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
                    SendType::RawTransaction { raw_tx, response } => {
                        if new_tx_sender
                            .send(TransactionMsg {
                                rlp_transaction: raw_tx,
                            })
                            .is_err()
                        {
                            tracing::error!("Failed sending raw transaction");
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
                        let mut rlp_transactions: Vec<Vec<u8>> = Vec::with_capacity(msg.len());
                        for tx in msg {
                            let mut rlp_transaction: Vec<u8> = Vec::new();
                            tx.encode_enveloped(&mut rlp_transaction);
                            rlp_transactions.push(rlp_transaction);
                        }
                        let sequence = TxSequenceMsgV2 {
                            sequence: rlp_transactions,
                        };

                        if new_tx_seq_sender.send(sequence).is_err() {
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
                    SendType::RawTransactionSequence { raw_txs, response } => {
                        let sequence = TxSequenceMsgV2 { sequence: raw_txs };

                        if new_tx_seq_sender.send(sequence).is_err() {
                            tracing::error!("Failed sending raw transaction sequence");
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
