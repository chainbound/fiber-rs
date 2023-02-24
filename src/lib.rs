use ethers::types::{
    transaction::eip2930::{AccessList, AccessListItem},
    OtherFields, Transaction as EthersTx, U256,
};
use pin_project::pin_project;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tonic::{transport::Channel, Request, Streaming};

pub mod api;
pub mod eth;
pub mod filter;
pub mod types;

use api::{
    api_client::ApiClient, RawTxMsg, RawTxSequenceMsg, TransactionResponse, TxFilter,
    TxSequenceMsg, TxSequenceResponse,
};
use eth::{Block, Transaction};

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
}

struct ClientInner {
    cmd_rx: mpsc::UnboundedReceiver<SendType>,
    new_tx_sender: mpsc::UnboundedSender<Transaction>,
    new_raw_tx_sender: mpsc::UnboundedSender<RawTxMsg>,
    new_tx_seq_sender: mpsc::UnboundedSender<TxSequenceMsg>,
    new_raw_tx_seq_sender: mpsc::UnboundedSender<RawTxSequenceMsg>,
    new_tx_responses: Streaming<TransactionResponse>,
    new_raw_tx_responses: Streaming<TransactionResponse>,
    new_tx_seq_responses: Streaming<TxSequenceResponse>,
    new_raw_tx_seq_responses: Streaming<TxSequenceResponse>,
}

impl ClientInner {
    async fn run_loop(mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                SendType::Transaction { tx, response } => {
                    self.new_tx_sender.send(tx).unwrap();

                    if let Some(res) = self.new_tx_responses.next().await {
                        let _ = response.send(res.unwrap());
                    }
                }
                SendType::RawTransaction { msg, response } => {
                    self.new_raw_tx_sender.send(msg).unwrap();

                    if let Some(res) = self.new_raw_tx_responses.next().await {
                        let _ = response.send(res.unwrap());
                    }
                }
                SendType::TransactionSequence { msg, response } => {
                    self.new_tx_seq_sender.send(msg).unwrap();

                    if let Some(res) = self.new_tx_seq_responses.next().await {
                        let _ = response.send(res.unwrap());
                    }
                }
                SendType::RawTransactionSequence { msg, response } => {
                    self.new_raw_tx_seq_sender.send(msg).unwrap();

                    if let Some(res) = self.new_raw_tx_seq_responses.next().await {
                        let _ = response.send(res.unwrap());
                    }
                }
            }
        }
    }
}

pub struct Client {
    key: String,
    client: ApiClient<Channel>,
    cmd_tx: mpsc::UnboundedSender<SendType>,
}

impl Client {
    pub async fn connect(
        target: String,
        api_key: String,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let targetstr = "http://".to_owned() + &target;
        let mut client = ApiClient::connect(targetstr.to_owned()).await?;

        // Set up the different streams
        let (new_tx_sender, rx) = mpsc::unbounded_channel();
        let rx_stream = UnboundedReceiverStream::new(rx);

        let mut req = Request::new(rx_stream);
        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", api_key.parse().unwrap());

        let new_tx_responses = client.send_transaction(req).await?.into_inner();

        let (new_raw_tx_sender, rx) = mpsc::unbounded_channel();
        let rx_stream = UnboundedReceiverStream::new(rx);

        let mut req = Request::new(rx_stream);
        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", api_key.parse().unwrap());

        let new_raw_tx_responses = client.send_raw_transaction(req).await?.into_inner();

        let (new_tx_seq_sender, rx) = mpsc::unbounded_channel();
        let rx_stream = UnboundedReceiverStream::new(rx);

        let mut req = Request::new(rx_stream);
        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", api_key.parse().unwrap());

        let new_tx_seq_responses = client.send_transaction_sequence(req).await?.into_inner();

        let (new_raw_tx_seq_sender, rx) = mpsc::unbounded_channel();
        let rx_stream = UnboundedReceiverStream::new(rx);

        let mut req = Request::new(rx_stream);
        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", api_key.parse().unwrap());

        let new_raw_tx_seq_responses = client
            .send_raw_transaction_sequence(req)
            .await?
            .into_inner();

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        let client = Client {
            client,
            key: api_key,
            cmd_tx,
        };

        // Create the inner client which has access to all the gRPC channels
        let inner = ClientInner {
            cmd_rx,
            new_tx_sender,
            new_tx_responses,
            new_raw_tx_sender,
            new_raw_tx_responses,
            new_tx_seq_sender,
            new_tx_seq_responses,
            new_raw_tx_seq_sender,
            new_raw_tx_seq_responses,
        };

        // Spawn the loop
        tokio::task::spawn(inner.run_loop());

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

    /// Subscribes to new transactions, returning a stream of ethers transactions.
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

        let mut req = Request::new(f);

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let mut inner = self
            .client
            .clone()
            .subscribe_new_txs(req)
            .await
            .unwrap()
            .into_inner();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(Ok(transaction)) = inner.next().await {
                let _ = tx.send(proto_to_tx(transaction));
            }
        });

        UnboundedReceiverStream::new(rx)
    }

    /// Subscribes to new blocks, returns a stream of proto blocks. TODO: convert
    /// these to ethers blocks.
    pub async fn subscribe_new_blocks(&self) -> UnboundedReceiverStream<Block> {
        let mut req = Request::new(());

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let mut inner = self
            .client
            .clone()
            .subscribe_new_blocks(req)
            .await
            .unwrap()
            .into_inner();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(Ok(block)) = inner.next().await {
                let _ = tx.send(block);
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

    let val = ethers::types::U256::from_big_endian(proto.value.as_slice());
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
        // Legacy
        if proto.v > 30 {
            proto.v - 10
        } else {
            proto.v
        }
    };

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
        chain_id: Some(proto.chain_id.into()),
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
