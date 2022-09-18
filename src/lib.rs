use eth::Transaction;
use ethers::types::{OtherFields, Transaction as EthersTx, U256};
use futures_core::stream::Stream;
use tonic::{transport::Channel, Request};

pub mod api;
pub mod eth;
pub mod types;

use api::{api_client::ApiClient, BackrunMsg, TxFilter};

struct Client<'a> {
    target: &'a str,
    key: &'a str,
    client: ApiClient<Channel>,
}

impl<'a> Client<'a> {
    pub async fn connect(
        target: &'a str,
        api_key: &'a str,
    ) -> Result<Client<'a>, Box<dyn std::error::Error>> {
        let targetstr = "http://".to_owned() + target;
        let client = ApiClient::connect(targetstr.to_owned()).await?;
        Ok(Client {
            target,
            client,
            key: api_key,
        })
    }

    /// sends a SIGNED transaction (e.g. v,r,s fields filled in). Returns hash and timestamp.
    pub async fn send_transaction(
        &mut self,
        tx: EthersTx,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let mut req = Request::new(tx_to_proto(tx));

        // Append the api key metadata
        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let res = self.client.send_transaction(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// backruns a transaction (propagates them in a bundle for ensuring the correct sequence).
    pub async fn backrun_transaction(
        &mut self,
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

        let res = self.client.backrun(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// sends a raw transaction signed transaction encoded as a byte slice.
    pub async fn send_raw_transaction(
        &mut self,
        raw_tx: &[u8],
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
        let mut req = Request::new(api::RawTxMsg {
            raw_tx: raw_tx.to_vec(),
        });

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());
        let res = self.client.send_raw_transaction(req).await?;

        Ok((res.get_ref().hash.to_owned(), res.get_ref().timestamp))
    }

    /// subscribes to new transactions. This function returns an async stream that needs
    /// to be pinned with futures_util::pin_mut, which can then be used to iterate over.
    pub async fn subscribe_new_txs(&mut self) -> impl Stream<Item = EthersTx> {
        // TODO: tx filtering
        let mut req = Request::new(TxFilter {
            from: String::new(),
            to: String::new(),
            value: vec![0],
        });

        req.metadata_mut()
            .append("x-api-key", self.key.parse().unwrap());

        let mut stream = self
            .client
            .subscribe_new_txs(req)
            .await
            .unwrap()
            .into_inner();

        async_stream::stream! {
            while let Some(proto) = stream.message().await.unwrap() {
                yield proto_to_tx(proto);
            }
        }
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
    use futures_util::{pin_mut, StreamExt};

    #[tokio::test]
    async fn connect() {
        // let target = "fiber-node.fly.dev:8080";
        let target = "localhost:8080";
        // let result = add(2, 2);
        // assert_eq!(result, 4);
        let client = Client::connect(target, "api_key").await.unwrap();
        assert_eq!(client.key, "api_key");
        assert_eq!(client.target, target);
    }

    #[test]
    fn test_metadata() {
        let mut req = Request::new("str");

        req.metadata_mut()
            .append("x-api-key", "api_key".parse().unwrap());

        assert_eq!(req.metadata().get("x-api-key").unwrap(), &"api_key");
    }

    #[tokio::test]
    async fn test_subscribe() {
        let target = "localhost:8080";
        let mut client = Client::connect(target, "api_key").await.unwrap();

        println!("connected to client");
        let sub = client.subscribe_new_txs().await;
        pin_mut!(sub);

        println!("listening to txs");

        while let Some(value) = sub.next().await {
            println!("{:#?}", value);
        }
    }
}
