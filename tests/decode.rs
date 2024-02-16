use ethereum_consensus::{ssz::prelude::deserialize, types::mainnet::SignedBeaconBlock};

#[test]
fn test_decode_block_ssz() {
    let file = std::fs::read("./tests/blocks/block8424647.ssz").unwrap();

    let block = deserialize::<SignedBeaconBlock>(&file).unwrap();

    let m = block.message();
    let b = m.body();
    let p = b.execution_payload().unwrap();

    assert_eq!(p.block_number(), 19226703);
    assert_eq!(
        format!("{:?}", p.parent_hash()),
        "0xe532bc32b0f170702d11ee71a36c91919df3249be14d2976a46e0a637436ad66"
    );
    assert_eq!(p.timestamp(), 1707919787);
    assert_eq!(
        format!("{:?}", p.extra_data()),
        "0x6265617665726275696c642e6f7267"
    );
    assert_eq!(p.gas_limit(), 30000000);
    assert_eq!(p.gas_used(), 8652723);

    assert_eq!(p.transactions().len(), 89);
    assert_eq!(p.withdrawals().unwrap().len(), 16);
}
