use crate::util::settings::{OutpointConf, Settings};
use anyhow::Result;
use ckb_sdk::{GenesisInfo, HttpRpcClient};
use ckb_types::core::{BlockView, DepType, TransactionView};
use ckb_types::packed::HeaderVec;
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, CellDep, CellOutput, OutPoint, Script},
    H256,
};
use faster_hex::hex_decode;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use force_sdk::tx_helper::TxHelper;
use force_sdk::util::get_live_cell;
use serde::export::Clone;

pub fn make_ckb_transaction(_from_lockscript: Script) -> Result<TransactionView> {
    todo!()
}

pub struct Generator {
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    _genesis_info: GenesisInfo,
    // _settings: Settings,
}

impl Generator {
    pub fn new(rpc_url: String, indexer_url: String) -> Result<Self, String> {
        let mut rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let genesis_block: BlockView = rpc_client
            .get_block_by_number(0)?
            .expect("Can not get genesis block?")
            .into();
        let genesis_info = GenesisInfo::from_block(&genesis_block)?;
        Ok(Self {
            rpc_client,
            indexer_client,
            _genesis_info: genesis_info,
            // _settings: settings,
        })
    }

    fn _add_cell_deps(
        &mut self,
        helper: &mut TxHelper,
        outpoints: Vec<OutpointConf>,
    ) -> Result<(), String> {
        let mut builder = helper.transaction.as_advanced_builder();
        for conf in outpoints {
            let outpoint = OutPoint::new_builder()
                .tx_hash(
                    Byte32::from_slice(
                        &hex::decode(conf.tx_hash)
                            .map_err(|e| format!("invalid OutpointConf config. err: {}", e))?,
                    )
                    .map_err(|e| format!("invalid OutpointConf config. err: {}", e))?,
                )
                .index(conf.index.pack())
                .build();
            builder = builder.cell_dep(
                CellDep::new_builder()
                    .out_point(outpoint)
                    .dep_type(DepType::Code.into())
                    .build(),
            );
        }
        helper.transaction = builder.build();
        Ok(())
    }

    fn _get_ckb_cell(
        &mut self,
        helper: &mut TxHelper,
        cell_typescript: Script,
        add_to_input: bool,
    ) -> Result<(CellOutput, Bytes), String> {
        let genesis_info = self._genesis_info.clone();
        let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_typescript)?
            .ok_or("cell not found")?;
        let ckb_cell = CellOutput::from(cell.output);
        let ckb_cell_data = packed::Bytes::from(cell.output_data).raw_data();
        if add_to_input {
            let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
                get_live_cell(&mut self.rpc_client, out_point, with_data).map(|(output, _)| output)
            };

            helper.add_input(
                cell.out_point.into(),
                None,
                &mut get_live_cell_fn,
                &genesis_info,
                true,
            )?;
        }
        Ok((ckb_cell, ckb_cell_data))
    }
    pub fn get_ckb_headers(&mut self, block_numbers: Vec<u64>) -> Vec<u8> {
        let mut mol_header_vec: Vec<packed::Header> = Default::default();
        for number in block_numbers {
            match self.rpc_client.get_block_by_number(number).unwrap() {
                Some(block) => mol_header_vec.push(block.header.inner.into()),
                None => continue,
            }
        }

        let mol_headers = HeaderVec::new_builder().set(mol_header_vec).build();
        Vec::from(mol_headers.as_slice())
    }
}

pub fn covert_to_h256(mut tx_hash: &str) -> Result<H256, String> {
    if tx_hash.starts_with("0x") || tx_hash.starts_with("0X") {
        tx_hash = &tx_hash[2..];
    }
    if tx_hash.len() % 2 != 0 {
        return Err(format!("Invalid hex string lenth: {}", tx_hash.len()));
    }
    let mut bytes = vec![0u8; tx_hash.len() / 2];
    hex_decode(tx_hash.as_bytes(), &mut bytes)
        .map_err(|err| format!("parse hex string failed: {:?}", err))?;
    H256::from_slice(&bytes).map_err(|err| err.to_string())
}
