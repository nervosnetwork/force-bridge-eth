use ckb_types::{
    packed,
    prelude::{Entity, Pack},
    H256,
};
use serde::{Deserialize, Serialize};

use crate::util::generated::ckb_tx_proof;
use ckb_types::bytes::Bytes;
use ckb_types::prelude::Builder;
use molecule::prelude::Byte;

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CKBHistoryTxProof {
    pub block_number: u64,
    pub tx_merkle_index: u16,
    pub witnesses_root: H256,
    pub lemmas: Vec<H256>,
    pub raw_transaction: Bytes,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CKBHistoryTxRootProof {
    pub init_block_number: u64,
    pub latest_block_number: u64,
    pub indices: Vec<u64>,
    pub proof_leaves: Vec<H256>,
    pub lemmas: Vec<H256>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CKBUnlockTokenParam {
    pub history_tx_root_proof: CKBHistoryTxRootProof,
    pub tx_proofs: Vec<CKBHistoryTxProof>,
}

impl From<CKBUnlockTokenParam> for ckb_tx_proof::CKBUnlockTokenParam {
    fn from(data: CKBUnlockTokenParam) -> Self {
        ckb_tx_proof::CKBUnlockTokenParam::new_builder()
            .history_tx_root_proof(data.history_tx_root_proof.into())
            .tx_proofs(data.tx_proofs.into())
            .build()
    }
}

impl From<CKBHistoryTxRootProof> for ckb_tx_proof::CKBHistoryTxRootProof {
    fn from(proof: CKBHistoryTxRootProof) -> Self {
        ckb_tx_proof::CKBHistoryTxRootProof::new_builder()
            .latest_block_number(proof.latest_block_number.into())
            .init_block_number(proof.init_block_number.into())
            .indices(proof.indices.into())
            .lemmas(proof.lemmas.into())
            .proof_leaves(proof.proof_leaves.into())
            .build()
    }
}

impl From<CKBHistoryTxProof> for ckb_tx_proof::CKBHistoryTxProof {
    fn from(json: CKBHistoryTxProof) -> Self {
        let CKBHistoryTxProof {
            tx_merkle_index,
            block_number,
            witnesses_root,
            lemmas,
            raw_transaction,
        } = json;

        let mol_lemmas_vec: Vec<ckb_tx_proof::Byte32> = lemmas
            .iter()
            .map(|hash| hash.pack().into())
            .collect::<Vec<_>>();

        let mol_lemmas = ckb_tx_proof::Byte32Vec::new_builder()
            .set(mol_lemmas_vec)
            .build();

        ckb_tx_proof::CKBHistoryTxProof::new_builder()
            .tx_merkle_index(tx_merkle_index.into())
            .block_number(block_number.into())
            .witnesses_root(witnesses_root.pack().into())
            .lemmas(mol_lemmas)
            .raw_transaction(raw_transaction.pack().as_bytes().into())
            .build()
    }
}

impl From<Bytes> for ckb_tx_proof::Bytes {
    fn from(b: Bytes) -> Self {
        Self::new_unchecked(b)
    }
}

impl From<Vec<CKBHistoryTxProof>> for ckb_tx_proof::CKBHistoryTxProofVec {
    fn from(data: Vec<CKBHistoryTxProof>) -> Self {
        Self::new_builder()
            .set(data.into_iter().map(|v| v.into()).collect())
            .build()
    }
}

impl From<Vec<H256>> for ckb_tx_proof::Byte32Vec {
    fn from(data: Vec<H256>) -> Self {
        Self::new_builder()
            .set(data.into_iter().map(|v| v.pack().into()).collect())
            .build()
    }
}

impl From<Vec<u64>> for ckb_tx_proof::Uint64Vec {
    fn from(data: Vec<u64>) -> Self {
        Self::new_builder()
            .set(data.into_iter().map(|v| v.into()).collect())
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
