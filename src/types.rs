use reth_primitives::{Address, BlobTransaction};

/// Wrapper struct over [`reth_primitives::BlobTransaction`] with recovered signer of the
/// transaction.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlobTransactionSignedEcRecovered {
    /// Signer of the transaction
    pub signer: Address,
    /// Signed transaction
    pub signed_transaction: BlobTransaction,
}
