use crate::util::settings::{OutpointConf, Settings};
use anyhow::Result;
use ckb_sdk::{Address, GenesisInfo, HttpRpcClient};
use ckb_types::core::{BlockView, DepType, TransactionView};
// use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, CellDep, CellOutput, OutPoint, Script},
};
use ethereum_types::H160;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use force_sdk::tx_helper::TxHelper;
use force_sdk::util::get_live_cell;
use std::str::FromStr;

pub fn make_ckb_transaction(_from_lockscript: Script) -> Result<TransactionView> {
    todo!()
}

pub struct Generator {
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    _genesis_info: GenesisInfo,
    _settings: Settings,
}

impl Generator {
    pub fn new(rpc_url: String, indexer_url: String, settings: Settings) -> Result<Self, String> {
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
            _settings: settings,
        })
    }

    pub fn generate_eth_spv_tx(
        &mut self,
        from_lockscript: Script,
        eth_proof: &ETHSPVProofJson,
    ) -> Result<TransactionView, String> {
        let tx_fee: u64 = 10000;
        let mut helper = TxHelper::default();

        // 1 bridge cells
        {
            let lockscript = Script::new_builder()
                .code_hash(
                    Byte32::from_slice(&self._settings.lockscript.code_hash.as_bytes()).unwrap(),
                )
                .hash_type(DepType::Code.into())
                // FIXME: add script args
                .args(ckb_types::packed::Bytes::default())
                .build();
            let to_output = CellOutput::new_builder().lock(lockscript).build();
            helper.add_output_with_auto_capacity(to_output, ckb_types::bytes::Bytes::default());
        }

        // 2 xt cells
        {
            let recipient_lockscript = Script::from(
                Address::from_str(&eth_proof.ckb_recipient)
                    .unwrap()
                    .payload(),
            );

            let sudt_typescript_code_hash = hex::decode(&self._settings.sudt.code_hash)
                .expect("wrong sudt_script code hash config");
            let sudt_typescript = Script::new_builder()
                .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
                .hash_type(DepType::Code.into())
                // FIXME: add script args
                .args(ckb_types::packed::Bytes::default())
                .build();

            let sudt_user_output = CellOutput::new_builder()
                .type_(Some(sudt_typescript).pack())
                .lock(recipient_lockscript)
                .build();

            let to_user_amount_data = eth_proof.lock_amount.to_le_bytes().to_vec().into();
            helper.add_output_with_auto_capacity(sudt_user_output, to_user_amount_data);
        }

        // add witness
        {
            // let witness_data = Default::default();
            // let witness = WitnessArgs::new_builder()
            //     .input_type(Some(witness_data.as_bytes()).pack())
            //     .build();
            //
            // helper.transaction = helper
            //     .transaction
            //     .as_advanced_builder()
            //     .set_witnesses(vec![witness.as_bytes().pack()])
            //     .build();
        }

        // build tx
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self._genesis_info,
            tx_fee,
        )?;
        Ok(tx)
        // Ok(TransactionView::)
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
}

#[derive(Default, Debug, Clone)]
pub struct ETHSPVProofJson {
    pub log_index: u64,
    pub log_entry_data: String,
    pub receipt_index: u64,
    pub receipt_data: String,
    pub header_data: String,
    pub proof: Vec<Vec<u8>>,
    pub token: H160,
    pub lock_amount: u128,
    pub ckb_recipient: String,
}
