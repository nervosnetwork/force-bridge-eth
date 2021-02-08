use merkle_cbt::{merkle_tree::Merge, CBMT as ExCBMT};
use tiny_keccak::{Hasher, Keccak};

pub struct Keccak256;

impl Merge for Keccak256 {
    type Item = [u8; 32];
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut output = [0u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize(&mut output);
        output
    }
}

pub type CBMT = ExCBMT<[u8; 32], Keccak256>;
