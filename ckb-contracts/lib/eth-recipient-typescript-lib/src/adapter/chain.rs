use super::{Adapter, ComplexData};
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{Byte, Byte32, Bytes, Script};
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_type, load_tx_hash, QueryIter};

use force_eth_types::{config::SUDT_CODE_HASH, eth_recipient_cell::ETHRecipientDataView};
use molecule::prelude::Entity;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_output_data(&self) -> Option<ETHRecipientDataView> {
        let data_list =
            QueryIter::new(load_cell_data, Source::GroupOutput).collect::<Vec<Vec<u8>>>();
        match data_list.len() {
            0 => None,
            1 => Some(
                ETHRecipientDataView::new(data_list[0].as_slice())
                    .expect("ETHRecipientDataView coding error"),
            ),
            _ => panic!("outputs have more than 1 eth recipient cell"),
        }
    }

    fn get_sudt_amount_from_source(&self, source: Source, eth_bridge_lock_hash: &[u8]) -> u128 {
        fn is_sudt_typescript(script: Option<Script>, lock_hash: &[u8]) -> bool {
            if script.is_none() {
                false
            }
            let script = script.unwrap();
            if script.code_hash().raw_data().as_ref() == SUDT_CODE_HASH.as_ref()
                && script.args().raw_data().as_ref() == lock_hash
                && script.hash_type() == 0u8.into()
            {
                true
            }
            false
        }

        let mut index = 0;
        let mut sudt_sum = 0;
        loop {
            let cell_type = load_cell_type(index, source);
            match cell_type {
                Err(SysError::IndexOutOfBound) => break,
                Err(_err) => panic!("iter input return an error"),
                Ok(cell_type) => {
                    if !(is_sudt_typescript(cell_type, eth_bridge_lock_hash)) {
                        index += 1;
                        continue;
                    }

                    let data = load_cell_data(index, source)?;
                    let mut buf = [0u8; 16];
                    if data.len() == 16 {
                        buf.copy_from_slice(&data);
                        sudt_sum += u128::from_le_bytes(buf)
                    }
                    index += 1;
                }
            }
        }
        sudt_sum
    }
}
