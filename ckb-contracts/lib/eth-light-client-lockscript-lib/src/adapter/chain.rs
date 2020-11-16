use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::Script;
use ckb_std::high_level::{load_cell_lock, load_cell_lock_hash, load_script, QueryIter};

use molecule::bytes::Bytes;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_input_script(&self) -> Script {
        load_script().expect("load script fail")
    }

    fn load_output_script(&self) -> Script {
        load_cell_lock(0, Source::Output).expect("load output cell lock fail")
    }

    fn check_input_owner(&self, owner_script: &Bytes) -> bool {
        QueryIter::new(load_cell_lock_hash, Source::Input)
            .filter(|lock| lock == owner_script.as_ref())
            .count()
            > 0
    }
}
