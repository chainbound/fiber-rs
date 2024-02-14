use ethereum_consensus::{ssz::prelude::deserialize, types::mainnet::SignedBeaconBlock};

#[test]
fn test_decode_block_ssz() {
    let file = std::fs::read("./tests/blocks/block8424647.ssz").unwrap();
    println!("file len: {}", file.len());

    let block = deserialize::<SignedBeaconBlock>(&file).unwrap();

    let m = block.message();
    let b = m.body();
    let p = b.execution_payload().unwrap();

    assert_eq!(p.block_number(), 19226703);
}
