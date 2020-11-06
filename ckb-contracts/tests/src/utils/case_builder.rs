#![allow(clippy::all)]

use ckb_testtool::context::Context;
pub use ckb_tool::ckb_types::bytes::Bytes;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use force_eth_types::generated::{basic, eth_recipient_cell::ETHRecipientCellData};
use std::collections::HashMap;
use std::mem::replace;
use std::vec::Vec;

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
    pub script_cells: ScriptCellView,
    pub sudt_cells: SudtCells,
    pub capacity_cells: CapacityCells,
    pub witnesses: Vec<Witness>,
    pub expect_return_code: i8,
}

pub enum ScriptCellView {
    ETHRecipientScript(ETHRecipientCells),
}

impl ScriptCellView {
    pub fn inputs_len(&self) -> usize {
        match self {
            ScriptCellView::ETHRecipientScript(_eth_recipient_cells) => 0,
        }
    }

    pub fn outputs_len(&self) -> usize {
        match self {
            ScriptCellView::ETHRecipientScript(eth_recipient_cells) => {
                eth_recipient_cells.outputs.len()
            }
        }
    }

    pub fn build_inputs(
        &self,
        _context: &mut Context,
        _outpoints: &OutpointsContext,
        _inputs: &mut Vec<CellInput>,
    ) {
        match self {
            ScriptCellView::ETHRecipientScript(_eth_recipient_cells) => {}
        }
    }

    pub fn build_outputs(
        &self,
        context: &mut Context,
        outpoints: &OutpointsContext,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) {
        match self {
            ScriptCellView::ETHRecipientScript(eth_recipient_cells) => {
                for cell in eth_recipient_cells.outputs.iter() {
                    let index = cell.get_index();
                    let (output_data, output_cell) = cell.build_output_cell(context, outpoints);
                    let _old_value = replace(&mut outputs[index], output_cell);
                    let _old_value = replace(&mut outputs_data[index], output_data);
                }
            }
        }
    }
}

pub enum CellDepView {}

impl CellDepView {
    pub fn build_cell_dep(&self, _context: &mut Context) -> CellDep {
        todo!()
    }
}

pub struct ETHRecipientCells {
    pub outputs: Vec<ETHRecipientCell>,
}

pub struct ETHRecipientCell {
    pub capacity: u64,
    pub data: ETHRecipientDataView,
    pub index: usize,
}

impl ETHRecipientCell {
    fn build_typescript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(
                &outpoints[ETH_RECIPIENT_TYPESCRIPT_OUTPOINT_KEY],
                Bytes::new(),
            )
            .expect("build eth recipient typescript")
    }

    fn build_lockscript(&self, context: &mut Context, outpoints: &OutpointsContext) -> Script {
        context
            .build_script(&outpoints[ALWAYS_SUCCESS_OUTPOINT_KEY], Default::default())
            .expect("build eth recipient lockscript")
    }
}

impl CellBuilder for ETHRecipientCell {
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
            .lock(self.build_lockscript(context, outpoints))
            .build();
        let output_data = self.data.as_molecule_bytes();
        (output_data, output_cell)
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

pub struct ETHRecipientDataView {
    pub eth_recipient_address: String,
    pub eth_token_address: String,
    pub token_amount: u128,
}

impl ETHRecipientDataView {
    pub fn as_molecule_bytes(&self) -> Bytes {
        let data = ETHRecipientCellData::new_builder()
            .eth_recipient_address(str_to_molecule_bytes(self.eth_recipient_address.as_str()))
            .eth_token_address(str_to_molecule_bytes(self.eth_token_address.as_str()))
            .token_amount(self.token_amount.into())
            .build();
        data.as_bytes()
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
            .expect("build sudt typescript succ")
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
pub enum Witness {}

impl Witness {
    pub fn as_bytes(&self) -> Bytes {
        todo!()
    }
}

fn str_to_molecule_bytes(s: &str) -> basic::Bytes {
    basic::Bytes::new_builder()
        .set(
            s.as_bytes()
                .iter()
                .map(|c| Byte::new(*c))
                .collect::<Vec<_>>()
                .into(),
        )
        .build()
}
