use crate::debug;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use eth_spv_lib::eth_types::*;

#[derive(Default, Debug, Clone)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: Vec<H512>, // [H512; 2]
    pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
    pub fn new(f: Vec<H512>, s: Vec<H128>) -> Self {
        Self {
            dag_nodes: f,
            proof: s,
        }
    }
    fn truncate_to_h128(arr: H256) -> H128 {
        let mut data = [0u8; 16];
        data.copy_from_slice(&(arr.0).0[16..]);
        H128(data.into())
    }

    fn hash_h128(l: H128, r: H128) -> H128 {
        let mut data = [0u8; 64];
        data[16..32].copy_from_slice(&(l.0).0);
        data[48..64].copy_from_slice(&(r.0).0);
        Self::truncate_to_h128(hash256(&data).into())
    }

    pub fn apply_merkle_proof(&self, index: u64) -> H128 {
        let mut data = [0u8; 128];
        data[..64].copy_from_slice(&(self.dag_nodes[0].0).0);
        data[64..].copy_from_slice(&(self.dag_nodes[1].0).0);

        let mut leaf = Self::truncate_to_h128(hash256(&data).into());

        for i in 0..self.proof.len() {
            if (index >> i as u64) % 2 == 0 {
                leaf = Self::hash_h128(leaf, self.proof[i]);
            } else {
                leaf = Self::hash_h128(self.proof[i], leaf);
            }
        }
        leaf
    }
}

pub fn verify_header(
    header: &BlockHeader,
    prev: Option<BlockHeader>,
    merkle_root: H128,
    dag_nodes: &[DoubleNodeWithMerkleProof],
) -> bool {
    let (_mix_hash, result) = hashimoto_merkle(
        &header.partial_hash.unwrap(),
        &header.nonce,
        header.number,
        merkle_root,
        dag_nodes,
    );
    debug!("verify_header header: {:?}", header);
    // See YellowPaper formula (50) in section 4.3.4
    // 1. Simplified difficulty check to conform adjusting difficulty bomb
    // 2. Added condition: header.parent_hash() == prev.hash()
    let result = U256((result.0).0.into()) < U256(ethash::cross_boundary(header.difficulty.0))
        && (header.difficulty < header.difficulty * 101 / 100
            && header.difficulty > header.difficulty * 99 / 100)
        && header.gas_used <= header.gas_limit
        && header.gas_limit >= U256(5000.into())
        && header.extra_data.len() <= 32;
    match prev {
        Some(prev) => {
            debug!("verify_header prev: {:?}", prev);
            result
                && header.gas_limit < prev.gas_limit * 1025 / 1024
                && header.gas_limit > prev.gas_limit * 1023 / 1024
                && header.timestamp > prev.timestamp
                && header.number == prev.number + 1
                && header.parent_hash == prev.hash.unwrap()
        }
        None => result,
    }
}

/// Verify merkle paths to the DAG nodes.
fn hashimoto_merkle(
    header_hash: &H256,
    nonce: &H64,
    header_number: u64,
    merkle_root: H128,
    nodes: &[DoubleNodeWithMerkleProof],
) -> (H256, H256) {
    let mut index = 0;
    let pair = ethash::hashimoto_with_hasher(
        header_hash.0,
        nonce.0,
        ethash::get_full_size(header_number as usize / 30000),
        |offset| {
            let idx = index;
            debug!("hashimoto_with_hasher index: {}", index);
            index += 1;
            // Each two nodes are packed into single 128 bytes with Merkle proof
            let node = &nodes[idx / 2];
            if idx % 2 == 0 {
                // Divide by 2 to adjust offset for 64-byte words instead of 128-byte
                assert_eq!(merkle_root, node.apply_merkle_proof((offset / 2) as u64));
            };

            // Reverse each 32 bytes for ETHASH compatibility
            let mut data = (node.dag_nodes[idx % 2].0).0;
            data[..32].reverse();
            data[32..].reverse();
            data.into()
        },
        my_keccak256,
        my_keccak512,
    );

    (H256(pair.0), H256(pair.1))
}
