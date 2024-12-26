use std::ops::{Deref, DerefMut};

use alloy::{
    consensus::{Signed, TxEip4844WithSidecar},
    primitives::Address,
};

/// Wrapper struct over [`alloy_consensus::TxEip4844WithSidecar`] with recovered signer of the
/// transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobTransactionSignedEcRecovered {
    /// Signer of the transaction
    pub signer: Address,
    /// Signed transaction
    pub signed_transaction: Signed<TxEip4844WithSidecar>,
}

impl Deref for BlobTransactionSignedEcRecovered {
    type Target = Signed<TxEip4844WithSidecar>;

    fn deref(&self) -> &Self::Target {
        &self.signed_transaction
    }
}

impl DerefMut for BlobTransactionSignedEcRecovered {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.signed_transaction
    }
}
