#![allow(clippy::all)]

use ckb_testtool::context::Context;
pub use ckb_tool::ckb_types::bytes::Bytes;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use core::convert::TryInto;
use force_eth_types::{
    eth_recipient_cell::ETHAddress,
    generated::{
        basic, eth_bridge_lock_cell::ETHBridgeLockArgs, eth_header_cell,
        eth_recipient_cell::ETHRecipientCellData,
    },
};
use hex::FromHex;
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

pub const ETH_BRIDGE_INPUT_OUTPOINT: &str =
    "5f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc94482200000000";

pub type OutpointsContext = HashMap<&'static str, OutPoint>;

pub trait CellBuilder {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput);

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
    ETHBridgeLockCustomCell(ETHBridgeLockCell),
}

impl CellBuilder for CustomCell {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput) {
        match self {
            CustomCell::ETHBridgeLockCustomCell(eth_bridge_lock_cell) => {
                eth_bridge_lock_cell.build_input_cell(context, outpoints)
            }
            _ => {
                let (cell_data, cell) = self.build_output_cell(context, outpoints);
                let input_out_point = context.create_cell(cell, cell_data);
                let input_cell = CellInput::new_builder()
                    .previous_output(input_out_point.clone())
                    .build();
                (input_out_point, input_cell)
            }
        }
    }

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
            CustomCell::ETHBridgeLockCustomCell(eth_bridge_lock_cell) => {
                eth_bridge_lock_cell.build_output_cell(context, outpoints)
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
            CustomCell::ETHBridgeLockCustomCell(eth_bridge_lock_cell) => eth_bridge_lock_cell.index,
        }
    }
}

pub enum CellDepView {
    ETHLightClientTypeCellDep(ETHLightClientTypeDep),
    ETHBridgeLockCellDep(ETHBridgeLockDep),
}

impl CellDepView {
    pub fn build_cell_dep(&self, context: &mut Context) -> CellDep {
        match self {
            CellDepView::ETHLightClientTypeCellDep(cell_dep) => cell_dep.build_cell_dep(context),
            CellDepView::ETHBridgeLockCellDep(cell_dep) => cell_dep.build_cell_dep(context),
        }
    }
}

pub struct ETHBridgeLockDep {
    pub confirmed_hashes: Vec<String>,
    pub unconfirmed_blocks: Vec<String>,
}

impl ETHBridgeLockDep {
    pub fn build_cell_dep(&self, context: &mut Context) -> CellDep {
        let mut main = vec![];
        for hash in self.confirmed_hashes.clone() {
            main.push(hex::decode(hash).unwrap().into())
        }
        for hash in self.unconfirmed_blocks.clone() {
            let header = eth_header_cell::ETHHeaderInfo::new_builder()
                .header(basic::Bytes::from(Vec::from_hex(hash).unwrap()))
                .build();
            main.push(header.as_slice().to_vec().into())
        }

        let data = eth_header_cell::ETHHeaderCellData::new_builder()
            .headers(
                eth_header_cell::ETHChain::new_builder()
                    .main(basic::BytesVec::new_builder().set(main).build())
                    .build(),
            )
            .build();

        let data_out_point = context.deploy_cell(data.as_bytes());
        CellDep::new_builder().out_point(data_out_point).build()
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
    pub merkle: Vec<String>,
}

impl ETHLightClientTypeCell {
    fn build_output_data(&self) -> Bytes {
        let mut main_vec = vec![];

        let main_len = self.main.clone().len();
        if main_len > 10 {
            for i in 0..main_len - 10 {
                let file = self.main[i].clone();
                if i > 0 && file == self.main[i - 1].clone() {
                    let block_with_proof = light_client_types::read_block(file);
                    let data = light_client_utils::create_hash_data(&block_with_proof);
                    let mut extra = vec![data; main_len - 10 - i];
                    main_vec.append(&mut extra);
                    break;
                } else {
                    let block_with_proof = light_client_types::read_block(file.clone());
                    let data = light_client_utils::create_hash_data(&block_with_proof);
                    main_vec.push(data);
                }
            }
        }
        let mut start_index = 0;
        if main_len > 10 {
            start_index = main_len - 10;
        }
        let mut pre_difficulty: u64 = 0;
        for i in start_index..main_len {
            let file = self.main[i].clone();
            if i > 0 && i < main_len - 1 && file == self.main[i + 1].clone() {
                let block_with_proof = light_client_types::read_block(file.clone());
                let (data, _) = light_client_utils::create_data(&block_with_proof, pre_difficulty);
                main_vec.push(data);
            } else {
                let block_with_proof = light_client_types::read_block(file.clone());
                let (data, difficulty) =
                    light_client_utils::create_data(&block_with_proof, pre_difficulty);
                pre_difficulty = difficulty;
                main_vec.push(data);
            }
        }

        let mut uncle_vec = vec![];
        for file in self.uncle.clone() {
            let block_with_proof = light_client_types::read_block(file);
            let (data, _) = light_client_utils::create_data(&block_with_proof, 0);
            uncle_vec.push(data);
        }

        let mut blocks = vec![];
        for file in self.merkle.clone() {
            let block_with_proof = light_client_types::read_block(file);
            blocks.push(block_with_proof);
        }

        if uncle_vec.is_empty() {
            return light_client_utils::create_cell_data(main_vec, None, &blocks).as_bytes();
        }
        return light_client_utils::create_cell_data(main_vec, Some(uncle_vec), &blocks).as_bytes();
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

pub struct ETHBridgeLockCell {
    pub capacity: u64,
    pub index: usize,
    pub eth_contract_address: String,
    pub eth_token_address: String,
}

impl ETHBridgeLockCell {
    fn build_input_cell(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
    ) -> (OutPoint, CellInput) {
        let (cell_data, cell) = self.build_output_cell(context, outpoints);

        let outpoint = hex::decode(ETH_BRIDGE_INPUT_OUTPOINT).unwrap();
        let outpoint = OutPoint::from_slice(outpoint.as_slice()).unwrap();
        context.create_cell_with_out_point(outpoint.clone(), cell, cell_data);
        let input_cell = CellInput::new_builder()
            .previous_output(outpoint.clone())
            .build();
        (outpoint, input_cell)
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
        (Default::default(), output_cell)
    }

    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[ALWAYS_SUCCESS_OUTPOINT_KEY], Default::default())
            .expect("build eth bridge typescript")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        let eth_contract_address = str_to_eth_address(&self.eth_contract_address);
        let eth_token_address = str_to_eth_address(&self.eth_token_address);
        let args = ETHBridgeLockArgs::new_builder()
            .eth_contract_address(eth_contract_address)
            .eth_token_address(eth_token_address)
            .build()
            .as_bytes();
        context
            .build_script(&outpoints[ETH_BRIDGE_LOCKSCRIPT_OUTPOINT_KEY], args)
            .expect("build eth bridge lockscript")
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
    pub lockscript: Script,
    pub owner_script: ScriptView,
    pub index: usize,
    pub sudt_extra_data: String,
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
    ) -> (Bytes, CellOutput) {
        let output_cell = CellOutput::new_builder()
            .capacity(self.capacity.pack())
            .type_(Some(self.build_typescript(context, outpoints)).pack())
            .lock(self.lockscript.clone())
            .build();

        let mut output_data = self.amount.to_le_bytes().to_vec();
        output_data.extend(self.sudt_extra_data.as_bytes().to_vec());
        (output_data.into(), output_cell)
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
            Witness::ETHBridgeWitness(witness) => witness.as_bytes(),
        }
    }
}

#[derive(Clone)]
pub struct ETHLightClientTypeWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub headers: Vec<String>,
}

impl ETHLightClientTypeWitness {
    pub fn as_bytes(&self) -> Bytes {
        let mut header_rlps = vec![];
        for file in self.headers.clone() {
            let block_with_proof = light_client_types::read_block(file);
            header_rlps.push(block_with_proof.header_rlp);
        }
        light_client_utils::create_witness(header_rlps, self.cell_dep_index_list.clone()).into()
    }
}

#[derive(Clone)]
pub struct ETHBridgeLockWitness {}

impl ETHBridgeLockWitness {
    pub fn as_bytes(&self) -> Bytes {
        let raw_witness = "6d0b0000100000006d0b00006d0b0000590b0000590b00001000000011000000540b0000003f0b00003f0b00001c000000240000006802000070020000c1050000dd070000000000000000000040020000f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000000000000000000004d030000f9034a01828005b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000018020000f90215a04809bc46bf97f29706c0ecf624e4c1365a031afd0956dd2442e870611a39dcdaa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479417c4b5ce0605f63732bfd175fece7ac6b4620fd2a074ecc5c6946ccf5216e5ac05388aa24685ea3763f19d203c4a6516ac20f25faea03a0eee4b28ff7e6da2cf5a78c00c1115e3c452162bc2639a9da749605596b2e4a0d3b519ded443cf60121272d2eef6705f5c45c068f02603ee65d61b111e0ea958b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000830209042786578c90689556828005845fcf552b99d883010917846765746888676f312e31352e33856c696e7578a0d67a513647fc02e9e29f8b5e499caa22753fead639d4fde47c185b6704fbcfeb8805d4f554585bc533620300000800000056030000f90353822080b9034df9034a01828005b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f6461746100000000000000000000000000000000000100000000".to_string();
        hex::decode(raw_witness).unwrap().into()
    }
}

fn str_to_eth_address(s: &str) -> basic::ETHAddress {
    let address: ETHAddress = ETHAddress::try_from(hex::decode(s).unwrap()).expect("decode fail");
    address.get_address().into()
}
