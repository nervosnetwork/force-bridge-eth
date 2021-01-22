use crate::util::ckb_util::parse_cell;
use ckb_types::prelude::Entity;

pub mod ckb_indexer;
pub mod eth_indexer;

pub trait IndexerFilter {
    fn filter(&self, data: String) -> bool;
}

pub struct DexFilter {
    pub code_hash: String,
}

impl IndexerFilter for DexFilter {
    fn filter(&self, data: String) -> bool {
        true
        // let recipient_lockscript_res = parse_cell(data.as_str());
        // if let Ok(recipient_lockscript) = recipient_lockscript_res {
        //     if hex::encode(recipient_lockscript.code_hash().as_slice()) == self.code_hash {
        //         return true;
        //     }
        // }
        // false
    }
}
