use crate::adapter::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};
use molecule::prelude::*;

pub fn verify_manage_mode<T: Adapter>(data_loader: &T) {
    let owner = data_loader.load_script_args();
    if !data_loader.lock_script_exists_in_inputs(owner.as_ref()) {
        panic!("not authorized to unlock the cell");
    }
}
