use ckb_types::{
    packed,
    prelude::{Entity, Pack},
    H256,
};
use serde::{Deserialize, Serialize};

use crate::util::generated::ckb_tx_proof;
use ckb_types::prelude::Builder;
use molecule::prelude::Byte;

// tx_merkle_index == index in transactions merkle tree of the block
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CkbTxProof {
    pub tx_merkle_index: u16,
    pub block_number: u64,
    pub block_hash: H256,
    pub tx_hash: H256,
    pub witnesses_root: H256,
    pub lemmas: Vec<H256>,
}

// CKBChain   CkbTxProof
// TokenLocker  unlockToken( CkbTxProof, RawTransaction + funding_input_index + extra )
impl From<CkbTxProof> for ckb_tx_proof::CkbTxProof {
    fn from(json: CkbTxProof) -> Self {
        let CkbTxProof {
            tx_merkle_index,
            block_number,
            block_hash,
            tx_hash,
            witnesses_root,
            lemmas,
        } = json;

        let mol_lemmas_vec: Vec<ckb_tx_proof::Byte32> = lemmas
            .iter()
            .map(|hash| hash.pack().into())
            .collect::<Vec<_>>();

        let mol_lemmas = ckb_tx_proof::Byte32Vec::new_builder()
            .set(mol_lemmas_vec)
            .build();

        ckb_tx_proof::CkbTxProof::new_builder()
            .tx_merkle_index(tx_merkle_index.into())
            .block_number(block_number.into())
            .block_hash(block_hash.pack().into())
            .tx_hash(tx_hash.pack().into())
            .witnesses_root(witnesses_root.pack().into())
            .lemmas(mol_lemmas)
            .build()
    }
}

impl From<u64> for ckb_tx_proof::Uint64 {
    fn from(v: u64) -> Self {
        let mut inner = [Byte::new(0); 8];
        let v = v
            .to_le_bytes()
            .to_vec()
            .into_iter()
            .map(Byte::new)
            .collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
    }
}

impl From<u16> for ckb_tx_proof::Uint16 {
    fn from(v: u16) -> Self {
        let mut inner = [Byte::new(0); 2];
        let v = v
            .to_le_bytes()
            .to_vec()
            .into_iter()
            .map(Byte::new)
            .collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Self::new_builder().set(inner).build()
    }
}

impl From<packed::Byte32> for ckb_tx_proof::Byte32 {
    fn from(v: packed::Byte32) -> Self {
        Self::new_unchecked(v.as_bytes())
    }
}

impl From<packed::Byte32Vec> for ckb_tx_proof::Byte32Vec {
    fn from(v: packed::Byte32Vec) -> Self {
        Self::new_unchecked(v.as_bytes())
    }
}
