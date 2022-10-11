use eth::Transaction;
use ethers::types::{OtherFields, Transaction as EthersTx, U256};
use pin_project::pin_project;
use tonic::{transport::Channel, Request};

pub mod api;
pub mod eth;
pub mod types;

use api::{api_client::ApiClient, BackrunMsg, TxFilter};

#[pin_project]
pub struct TxStream {
    #[pin]
    stream: tonic::codec::Streaming<Transaction>,
}

impl TxStream {
    pub async fn next(&mut self) -> Option<EthersTx> {
        let proto = self.stream.message().await.unwrap_or(None)?;
        Some(proto_to_tx(proto))
    }
}

pub struct Client {
    key: String,
    client: ApiClient<Channel>,
}

impl Client {
    pub async fn connect(
        target: String,
        api_key: String,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let targetstr = "http://".to_owned() + &target;
        let client = ApiClient::connect(targetstr.to_owned()).await?;
        Ok(Client {
            client,
            key: api_key,
        })
    }

    /// sends a SIGNED transaction (e.g. v,r,s fields filled in). Returns hash and timestamp.
    pub async fn send_transaction(
        &self,
        tx: EthersTx,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let mut req = Request::new(tx_to_proto(tx));

        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let res = self.client.clone().send_transaction(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// backruns a transaction (propagates them in a bundle for ensuring the correct sequence).
    pub async fn backrun_transaction(
        &self,
        hash: String,
        tx: EthersTx,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let proto = tx_to_proto(tx);
        let mut req = Request::new(BackrunMsg {
            hash: hash,
            tx: Some(proto),
        });

        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let res = self.client.clone().backrun(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// sends a signed transaction encoded as a byte slice.
    pub async fn send_raw_transaction(
        &self,
        raw_tx: &[u8],
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let mut req = Request::new(api::RawTxMsg {
            raw_tx: raw_tx.to_vec(),
        });

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());
        let res = self.client.clone().send_raw_transaction(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// sends a raw transaction signed transaction encoded as a byte slice.
    pub async fn raw_backrun_transaction(
        &self,
        hash: String,
        raw_tx: &[u8],
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let mut req = Request::new(api::RawBackrunMsg {
            hash: hash,
            raw_tx: raw_tx.to_vec(),
        });

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());
        let res = self.client.clone().raw_backrun(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// subscribes to new transactions. This function returns an async stream that needs
    /// to be pinned with futures_util::pin_mut, which can then be used to iterate over.
    pub async fn subscribe_new_txs(&mut self, filter: Option<TxFilter>) -> TxStream {
        let f = filter.unwrap_or(TxFilter {
            from: vec![],
            to: vec![],
            method_id: vec![],
        });

        let mut req = Request::new(f);

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let stream = self
            .client
            .subscribe_new_txs(req)
            .await
            .unwrap()
            .into_inner();

        TxStream { stream }
    }

    // subscribes to new blocks. This function returns an async stream that needs
    // to be pinned with futures_util::pin_mut, which can then be used to iterate over.
    // pub async fn subscribe_new_blocks(&mut self) -> impl Stream<Item = eth::Block> {
    //     // TODO: tx filtering
    //     let mut req = Request::new(BlockFilter {
    //         producer: String::new(),
    //     });

    //     req.metadata_mut()
    //         .append("x-api-key", self.key.parse().unwrap());

    //     let mut stream = self.client.subscribe_new_blocks(req).await.unwrap().into_inner();

    //     async_stream::stream! {
    //         while let Some(block) = stream.message().await.unwrap() {
    //             yield block;
    //         }
    //     }
    // }
}

fn tx_to_proto(tx: EthersTx) -> Transaction {
    let mut to: Option<Vec<u8>> = None;
    match tx.to {
        Some(rcv) => {
            to = Some(rcv.as_bytes().to_vec());
        }
        _ => {}
    }

    let tx_type = match tx.transaction_type {
        Some(tp) => tp.as_u32(),
        None => 0,
    };

    let mut val_bytes = [0];
    tx.value.to_big_endian(&mut val_bytes);

    let mut r_bytes = [0];
    tx.r.to_big_endian(&mut r_bytes);

    let mut s_bytes = [0];
    tx.s.to_big_endian(&mut s_bytes);

    Transaction {
        to: to,
        gas: tx.gas.as_u64(),
        gas_price: tx.gas_price.unwrap_or(ethers::types::U256::zero()).as_u64(),
        hash: tx.hash.as_bytes().to_vec(),
        input: tx.input.to_vec(),
        nonce: tx.nonce.as_u64(),
        value: val_bytes.to_vec(),
        from: tx.from.as_bytes().to_vec(),
        r#type: tx_type,
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
    }
}

fn proto_to_tx(proto: Transaction) -> EthersTx {
    let to = match proto.to {
        Some(vec) => Some(ethers::types::H160::from_slice(vec.as_slice())),
        None => None,
    };

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

    EthersTx {
        hash: ethers::types::H256::from_slice(proto.hash.as_slice()),
        nonce: proto.nonce.into(),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        from: ethers::types::H160::from_slice(proto.from.as_slice()),
        to: to,
        value: val,
        gas_price: gas_price,
        gas: proto.gas.into(),
        input: proto.input.into(),
        v: proto.v.into(),
        r: r,
        s: s,
        transaction_type: tx_type,
        access_list: None,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: max_fee,
        chain_id: Some(proto.chain_id.into()),
        other: OtherFields::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::{
        signers::{LocalWallet, Signer},
        types::{transaction::eip2718::TypedTransaction, Address, TransactionRequest},
    };

    #[tokio::test]
    async fn connect() {
        // let target = "fiber-node.fly.dev:8080";
        let target = String::from("localhost:8080");
        let client = Client::connect(target, String::from("api_key"))
            .await
            .unwrap();
        assert_eq!(client.key, "api_key");
    }

    #[test]
    fn test_metadata() {
        let mut req = Request::new("str");

        req.metadata_mut()
            .append("x-api-key", "api_key".parse().unwrap());

        assert_eq!(req.metadata().get("x-api-key").unwrap(), &"api_key");
    }

    #[tokio::test]
    async fn test_send_transaction() {
        let target = String::from("localhost:8080");
        let mut client = Client::connect(target, String::from("api_key"))
            .await
            .unwrap();

        let tx: TypedTransaction = TransactionRequest::new()
            .nonce(3)
            .gas_price(1)
            .gas(25000)
            .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b"
                .parse::<Address>()
                .unwrap())
            .value(10)
            .data(vec![0x55, 0x44])
            .chain_id(1)
            .into();

        let wallet: LocalWallet =
            "15bb7dd02dd8805338310f045ae9975aedb7c90285618bd2ecdc91db52170a90"
                .parse()
                .unwrap();

        let sig = wallet.sign_transaction(&tx.clone()).await.unwrap();

        let signed = tx.rlp_signed(&sig);

        let res = client.send_raw_transaction(&signed).await.unwrap();

        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_subscribe() {
        let target = String::from("localhost:8080");
        let mut client = Client::connect(target, String::from("api_key"))
            .await
            .unwrap();

        println!("connected to client");
        let mut sub = client.subscribe_new_txs(None).await;

        println!("listening to txs");

        while let Some(value) = sub.next().await {
            println!("{:#?}", value.hash);
        }
    }
}
