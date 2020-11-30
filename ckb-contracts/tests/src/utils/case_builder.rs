#![allow(clippy::all)]

use ckb_testtool::context::Context;
pub use ckb_tool::ckb_types::bytes::Bytes;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use core::convert::TryInto;
use force_eth_types::{
    eth_recipient_cell::ETHAddress,
    generated::{
        basic, eth_bridge_lock_cell::ETHBridgeLockArgs, eth_recipient_cell::ETHRecipientCellData,
    },
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::vec::Vec;

use crate::eth_light_client_typescript::types as light_client_types;
use crate::eth_light_client_typescript::utils as light_client_utils;

pub const ETH_BRIDGE_LOCKSCRIPT_OUTPOINT_KEY: &str = "eth_bridge_lockcript_outpoint_key";
pub const ETH_LIGHT_CLIENT_LOCKSCRIPT_OUTPOINT_KEY: &str =
    "eth_light_client_lockcript_outpoint_key";
pub const ETH_LIGHT_CLIENT_TYPESCRIPT_OUTPOINT_KEY: &str =
    "eth_light_client_typecript_outpoint_key";
pub const ETH_RECIPIENT_TYPESCRIPT_OUTPOINT_KEY: &str = "eth_recipient_typescript_outpoint_key";
pub const SUDT_TYPESCRIPT_OUTPOINT_KEY: &str = "sudt_typescript_key";
pub const ALWAYS_SUCCESS_OUTPOINT_KEY: &str = "always_success_outpoint_key";
pub const FIRST_INPUT_OUTPOINT_KEY: &str = "cell_id_outpoint_key";

pub type OutpointsContext = HashMap<&'static str, OutPoint>;

pub trait CellBuilder {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput) {
        let (cell_data, cell) = self.build_output_cell(context, outpoints);
        let input_out_point = context.create_cell(cell, cell_data);
        let input_cell = CellInput::new_builder()
            .previous_output(input_out_point.clone())
            .build();
        (input_out_point, input_cell)
    }

    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput);

    fn get_index(&self) -> usize;
}

pub struct TestCase {
    pub cell_deps: Vec<CellDepView>,
    pub script_cells: CustomCells,
    pub sudt_cells: SudtCells,
    pub capacity_cells: CapacityCells,
    pub witnesses: Vec<Witness>,
    pub expect_return_error_info: String,
}

pub struct CustomCells {
    pub inputs: Vec<CustomCell>,
    pub outputs: Vec<CustomCell>,
}

pub enum CustomCell {
    ETHRecipientCustomCell(ETHRecipientCell),
    ETHLightClientLockCustomCell(ETHLightClientLockCell),
    ETHLightClientTypeCustomCell(ETHLightClientTypeCell),
}

impl CellBuilder for CustomCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        match self {
            CustomCell::ETHRecipientCustomCell(eth_recipient_cell) => {
                eth_recipient_cell.build_output_cell(context, outpoints)
            }
            CustomCell::ETHLightClientLockCustomCell(eth_light_client_lock_cell) => {
                eth_light_client_lock_cell.build_output_cell(context, outpoints)
            }
            CustomCell::ETHLightClientTypeCustomCell(eth_light_client_type_cell) => {
                eth_light_client_type_cell.build_output_cell(context, outpoints)
            }
        }
    }

    fn get_index(&self) -> usize {
        match self {
            CustomCell::ETHRecipientCustomCell(eth_recipient_cell) => eth_recipient_cell.index,
            CustomCell::ETHLightClientLockCustomCell(eth_light_client_lock_cell) => {
                eth_light_client_lock_cell.index
            }
            CustomCell::ETHLightClientTypeCustomCell(eth_light_client_type_cell) => {
                eth_light_client_type_cell.index
            }
        }
    }
}

pub enum CellDepView {
    ETHLightClientTypeCellDep(ETHLightClientTypeDep),
}

impl CellDepView {
    pub fn build_cell_dep(&self, _context: &mut Context) -> CellDep {
        match self {
            CellDepView::ETHLightClientTypeCellDep(cell_dep) => cell_dep.build_cell_dep(_context),
        }
    }
}

pub struct ETHLightClientTypeDep {}

impl ETHLightClientTypeDep {
    pub fn build_cell_dep(&self, context: &mut Context) -> CellDep {
        let dep_data = light_client_utils::create_dep_data();
        let data_out_point = context.deploy_cell(dep_data);
        CellDep::new_builder().out_point(data_out_point).build()
    }
}

pub struct ETHRecipientCell {
    pub capacity: u64,
    pub data: ETHRecipientDataView,
    pub index: usize,
}

impl ETHRecipientCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.build_lockscript(context, outpoints))
            .build();
        let output_data = self.data.as_molecule_bytes();
        (output_data, output_cell)
    }

    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(
                &outpoints[ETH_RECIPIENT_TYPESCRIPT_OUTPOINT_KEY],
                Default::default(),
            )
            .expect("build eth recipient typescript")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[ALWAYS_SUCCESS_OUTPOINT_KEY], Default::default())
            .expect("build eth recipient lockscript")
    }
}

pub struct ETHRecipientDataView {
    pub eth_recipient_address: String,
    pub eth_token_address: String,
    pub eth_lock_contract_address: String,
    pub eth_bridge_lock_hash: [u8; 32],
    pub token_amount: u128,
    pub fee: u128,
}

impl ETHRecipientDataView {
    pub fn as_molecule_bytes(&self) -> Bytes {
        let data = ETHRecipientCellData::new_builder()
            .eth_recipient_address(str_to_eth_address(self.eth_recipient_address.as_str()))
            .eth_token_address(str_to_eth_address(self.eth_token_address.as_str()))
            .eth_lock_contract_address(str_to_eth_address(self.eth_lock_contract_address.as_str()))
            .eth_bridge_lock_hash(self.eth_bridge_lock_hash.to_vec().try_into().unwrap())
            .token_amount(self.token_amount.into())
            .fee(self.fee.into())
            .build();
        data.as_bytes()
    }
}

pub struct ETHLightClientLockCell {
    pub capacity: u64,
    pub index: usize,
    pub args: Vec<u8>,
}

impl ETHLightClientLockCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.build_lockscript(context, outpoints))
            .build();
        (Default::default(), output_cell)
    }

    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[ALWAYS_SUCCESS_OUTPOINT_KEY], Default::default())
            .expect("build eth light client typescript")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(
                &outpoints[ETH_LIGHT_CLIENT_LOCKSCRIPT_OUTPOINT_KEY],
                Bytes::from(self.args.clone()),
            )
            .expect("build eth light client lockscript")
    }
}

pub struct ETHLightClientTypeCell {
    pub capacity: u64,
    pub index: usize,
    pub main: Vec<String>,
    pub uncle: Vec<String>,
    pub merkle: Option<String>,
}

impl ETHLightClientTypeCell {
    fn build_output_data(&self) -> Bytes {
        let mut main_vec = vec![];
        let mut pre_difficulty: u64 = 0;
        for file in self.main.clone() {
            let block_with_proof = light_client_types::read_block(file);
            let (data, difficulty) =
                light_client_utils::create_data(&block_with_proof, pre_difficulty);
            pre_difficulty = difficulty;
            main_vec.push(data);
        }

        let mut uncle_vec = vec![];
        for file in self.uncle.clone() {
            let block_with_proof = light_client_types::read_block(file);
            let (data, _) = light_client_utils::create_data(&block_with_proof, 0);
            uncle_vec.push(data);
        }

        let mut block_with_proof =
            light_client_types::read_block(self.main.last().unwrap().clone());

        if self.merkle.is_some() {
            block_with_proof = light_client_types::read_block(self.merkle.clone().unwrap());
        }

        if uncle_vec.is_empty() {
            return light_client_utils::create_cell_data(main_vec, None, &block_with_proof)
                .as_bytes();
        }
        return light_client_utils::create_cell_data(main_vec, Some(uncle_vec), &block_with_proof)
            .as_bytes();
    }

    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.build_lockscript(context, outpoints))
            .build();
        let output_data = self.build_output_data();
        (output_data, output_cell)
    }

    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(
                &outpoints[ETH_LIGHT_CLIENT_TYPESCRIPT_OUTPOINT_KEY],
                outpoints[FIRST_INPUT_OUTPOINT_KEY].as_bytes(),
            )
            .expect("build eth light client typescript")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[ALWAYS_SUCCESS_OUTPOINT_KEY], Default::default())
            .expect("build eth light client lockscript")
    }
}

pub struct ScriptView {
    pub outpoint_key: &'static str,
    pub args: Bytes,
}

impl Default for ScriptView {
    fn default() -> Self {
        Self {
            outpoint_key: ALWAYS_SUCCESS_OUTPOINT_KEY,
            args: Default::default(),
        }
    }
}

impl ScriptView {
    pub fn build_script(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[self.outpoint_key], self.args.clone())
            .expect("build script succ")
    }

    pub fn build_sudt_owner(eth_contract_address: &str, eth_token_address: &str) -> Self {
        let eth_contract_address = str_to_eth_address(eth_contract_address);
        let eth_token_address = str_to_eth_address(eth_token_address);
        let args = ETHBridgeLockArgs::new_builder()
            .eth_contract_address(eth_contract_address)
            .eth_token_address(eth_token_address)
            .build()
            .as_bytes();
        Self {
            outpoint_key: ETH_BRIDGE_LOCKSCRIPT_OUTPOINT_KEY,
            args,
        }
    }
}

#[derive(Default)]
pub struct SudtCells {
    pub inputs: Vec<SudtCell>,
    pub outputs: Vec<SudtCell>,
}

#[derive(Default)]
pub struct SudtCell {
    pub capacity: u64,
    pub amount: u128,
    pub lockscript: ScriptView,
    pub owner_script: ScriptView,
    pub index: usize,
}

impl SudtCell {
    pub fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        let owner_script = context
            .build_script(
                &outpoints[self.owner_script.outpoint_key],
                self.owner_script.args.clone(),
            )
            .expect("build owner script");
        let args: [u8; 32] = owner_script.calc_script_hash().unpack();
        let args: Bytes = args.to_vec().into();
        context
            .build_script(&outpoints[SUDT_TYPESCRIPT_OUTPOINT_KEY], args)
            .expect("build sudt typescript fail")
    }
}

impl CellBuilder for SudtCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.lockscript.build_script(context, outpoints))
            .build();
        let output_data = self.amount.to_le_bytes().to_vec().into();
        (output_data, output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(Default)]
pub struct CapacityCells {
    pub inputs: Vec<CapacityCell>,
    pub outputs: Vec<CapacityCell>,
}

#[derive(Default)]
pub struct CapacityCell {
    pub capacity: u64,
    pub lockscript: ScriptView,
    pub index: usize,
}

impl CellBuilder for CapacityCell {
    fn build_output_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .lock(self.lockscript.build_script(context, outpoints))
            .build();
        (Default::default(), output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum Witness {
    ETHLightClientWitness(ETHLightClientTypeWitness),
    ETHBridgeWitness(ETHBridgeLockWitness),
}

impl Witness {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            Witness::ETHLightClientWitness(witness) => witness.as_bytes(),
            Witness::ETHBridgeWitness(_witness) => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct ETHLightClientTypeWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub header: String,
}

impl ETHLightClientTypeWitness {
    pub fn as_bytes(&self) -> Bytes {
        let block_with_proof = light_client_types::read_block(self.header.clone());
        light_client_utils::create_witness(block_with_proof, self.cell_dep_index_list.clone())
            .into()
    }
}

#[derive(Clone)]
pub struct ETHBridgeLockWitness {}

fn str_to_eth_address(s: &str) -> basic::ETHAddress {
    let address: ETHAddress = ETHAddress::try_from(hex::decode(s).unwrap()).expect("decode fail");
    address.get_address().into()
}
