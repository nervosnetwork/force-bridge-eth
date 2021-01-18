pub mod ckb_indexer;
pub mod eth_indexer;

pub trait IndexerFilter {
    fn filter(&self) -> bool;
}
