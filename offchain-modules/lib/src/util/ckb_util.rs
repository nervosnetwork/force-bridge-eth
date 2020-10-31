use crate::util::eth_proof_helper::Witness;
use crate::util::settings::{OutpointConf, Settings};
use anyhow::{anyhow, bail, Result};
use ckb_sdk::{Address, AddressPayload, GenesisInfo, HttpRpcClient, SECP256K1};
use ckb_types::core::{BlockView, DepType, TransactionView};
use ckb_types::packed::{HeaderVec, ScriptReader, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack, Reader};
use ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, CellDep, CellOutput, OutPoint, Script},
    H256,
};
use ethereum_types::H160;
use faster_hex::hex_decode;
use force_eth_types::generated::basic::BytesVec;
use force_eth_types::generated::{basic, witness};
use force_sdk::cell_collector::{collect_sudt_amount, get_live_cell_by_lockscript, get_live_cell_by_typescript};
use force_sdk::indexer::{Cell, IndexerRpcClient};
use force_sdk::tx_helper::{sign, TxHelper};
use force_sdk::util::{get_live_cell_with_cache,send_tx_sync};
use secp256k1::SecretKey;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use web3::types::{Block, BlockHeader};



pub struct Generator {
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    genesis_info: GenesisInfo,
    pub settings: Settings,
}

impl Generator {
    pub fn new(rpc_url: String, indexer_url: String, settings: Settings) -> Result<Self> {
        let mut rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let genesis_block: BlockView = rpc_client
            .get_block_by_number(0)
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("Can not get genesis block?"))?
            .into();
        let genesis_info = GenesisInfo::from_block(&genesis_block).map_err(|err| anyhow!(err))?;
        Ok(Self {
            rpc_client,
            indexer_client,
            genesis_info,
            settings,
        })
    }

    #[allow(clippy::mutable_key_type)]
    pub fn init_light_client_tx(
        &mut self,
        _witness: &Witness,
        from_lockscript: Script,
        typescript: Script,
    ) -> Result<TransactionView> {
        let tx_fee: u64 = 10000;
        let mut helper = TxHelper::default();

        let outpoints = vec![self.settings.light_client_typescript.outpoint.clone()];
        self.add_cell_deps(&mut helper, outpoints)
            .map_err(|err| anyhow!(err))?;

        // build tx
        let tx = helper
            .supply_capacity(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript,
                &self.genesis_info,
                tx_fee,
            )
            .map_err(|err| anyhow!(err))?;
        let first_outpoint = tx
            .inputs()
            .get(0)
            .expect("should have input")
            .previous_output()
            .as_bytes();
        let typescript_args = first_outpoint.as_ref();
        let new_typescript = typescript.as_builder().args(typescript_args.pack()).build();

        let output = CellOutput::new_builder()
            .type_(Some(new_typescript).pack())
            .build();
        let output_data = ckb_types::bytes::Bytes::default();
        helper.add_output_with_auto_capacity(output, output_data);

        Ok(tx)
    }

    #[allow(clippy::mutable_key_type)]
    pub fn generate_eth_light_client_tx(
        &mut self,
        header: &Block<ethereum_types::H256>,
        cell: &Cell,
        _witness: &Witness,
        headers: &[BlockHeader],
        from_lockscript: Script,
    ) -> Result<TransactionView> {
        let tx_fee: u64 = 10000;
        let mut helper = TxHelper::default();

        let outpoints = vec![self.settings.light_client_typescript.outpoint.clone()];
        self.add_cell_deps(&mut helper, outpoints)
            .map_err(|err| anyhow!(err))?;

        let mut live_cell_cache: HashMap<(OutPoint, bool), (CellOutput, Bytes)> =
            Default::default();
        let rpc_client = &mut self.rpc_client;
        let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
            get_live_cell_with_cache(&mut live_cell_cache, rpc_client, out_point, with_data)
                .map(|(output, _)| output)
        };
        helper
            .add_input(
                OutPoint::from(cell.clone().out_point),
                None,
                &mut get_live_cell_fn,
                &self.genesis_info,
                true,
            )
            .map_err(|err| anyhow!(err))?;
        {
            let cell_output = CellOutput::from(cell.clone().output);
            let output = CellOutput::new_builder()
                .lock(cell_output.lock())
                .type_(cell_output.type_())
                .build();
            let tip = &headers[headers.len() - 1];
            let mut _output_data = ckb_types::bytes::Bytes::default();
            if tip.parent_hash == header.hash.unwrap()
                || header.number.unwrap().as_u64() >= tip.number.unwrap().as_u64()
            {
                // the new header is on main chain.
                // FIXME: build output data. Wait for the ckb contract to define the data structure
            } else {
                // the new header is on uncle chain.
                // FIXME: build output data.
                _output_data = ckb_types::bytes::Bytes::default();
            }
            helper.add_output_with_auto_capacity(output, _output_data);
        }

        {
            // add witness
            // FIXME: add witness data. Wait for the ckb contract to define the data structure
            // let witness_data = witness::Witness::new_builder()
            //     .header(case.witness.header.into())
            //     .merkle_proof(BytesVec::new_builder().set(proofs).build())
            //     .cell_dep_index_list(case.witness.cell_dep_index_list.into())
            //     .build();
            // let witness = WitnessArgs::new_builder()
            //     .input_type(Some(witness_data.as_bytes()).pack())
            //     .build();
        }

        // build tx
        let tx = helper
            .supply_capacity(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript,
                &self.genesis_info,
                tx_fee,
            )
            .map_err(|err| anyhow!(err))?;

        Ok(tx)
    }

    #[allow(clippy::mutable_key_type)]
    pub fn generate_eth_spv_tx(
        &mut self,
        from_lockscript: Script,
        eth_proof: &ETHSPVProofJson,
        cell_dep: String,
    ) -> Result<TransactionView> {
        let tx_fee: u64 = 10000;
        let mut helper = TxHelper::default();

        // add cell deps.
        {
            let outpoints = vec![
                self.settings.bridge_lockscript.outpoint.clone(),
                self.settings.bridge_typescript.outpoint.clone(),
                self.settings.sudt.outpoint.clone(),
            ];
            self.add_cell_deps(&mut helper, outpoints)
                .map_err(|err| anyhow!(err))?;

            let cell_script = parse_cell(cell_dep.as_str())?;
            let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_script)
                .map_err(|err| anyhow!(err))?
                .ok_or_else(|| anyhow!("no cell found for cell dep"))?;
            let mut builder = helper.transaction.as_advanced_builder();
            builder = builder.cell_dep(
                CellDep::new_builder()
                    .out_point(cell.out_point.into())
                    .dep_type(DepType::Code.into())
                    .build(),
            );
            helper.transaction = builder.build();
        }

        let lockscript_code_hash = hex::decode(&self.settings.bridge_lockscript.code_hash)?;
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash)?)
            .hash_type(DepType::Code.into())
            // FIXME: add script args
            .args(ckb_types::packed::Bytes::default())
            .build();

        // input bridge cells
        {
            let rpc_client = &mut self.rpc_client;
            let indexer_client = &mut self.indexer_client;
            let mut live_cell_cache: HashMap<(OutPoint, bool), (CellOutput, Bytes)> =
                Default::default();
            let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
                get_live_cell_with_cache(&mut live_cell_cache, rpc_client, out_point, with_data)
                    .map(|(output, _)| output)
            };
            let cell = get_live_cell_by_lockscript(indexer_client, lockscript.clone())
                .map_err(|err| anyhow!(err))?
                .ok_or_else(|| anyhow!("there are no remaining public cells available"))?;
            helper
                .add_input(
                    OutPoint::from(cell.out_point),
                    None,
                    &mut get_live_cell_fn,
                    &self.genesis_info,
                    true,
                )
                .map_err(|err| anyhow!(err))?;
        }

        // 1 bridge cells
        {
            let to_output = CellOutput::new_builder().lock(lockscript).build();
            helper.add_output_with_auto_capacity(to_output, ckb_types::bytes::Bytes::default());
        }

        // 2 xt cells
        {
            let recipient_lockscript = Script::from(
                Address::from_str(&eth_proof.ckb_recipient)
                    .map_err(|err| anyhow!(err))?
                    .payload(),
            );
            let sudt_typescript_code_hash = hex::decode(&self.settings.sudt.code_hash)?;
            let sudt_typescript = Script::new_builder()
                .code_hash(Byte32::from_slice(&sudt_typescript_code_hash)?)
                .hash_type(DepType::Code.into())
                .args(recipient_lockscript.calc_script_hash().as_bytes().pack())
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
            let witness_data = EthWitness {
                cell_dep_index_list: vec![0],
                spv_proof: eth_proof.clone(),
            };

            let witness = WitnessArgs::new_builder()
                .input_type(Some(witness_data.as_bytes()).pack())
                .build();

            helper.transaction = helper
                .transaction
                .as_advanced_builder()
                .set_witnesses(vec![witness.as_bytes().pack()])
                .build();
        }
        // build tx
        let tx = helper
            .supply_capacity(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript,
                &self.genesis_info,
                tx_fee,
            )
            .map_err(|err| anyhow!(err))?;
        Ok(tx)
    }

    fn add_cell_deps(
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

    pub fn get_ckb_cell(
        &mut self,
        // helper: &mut TxHelper,
        cell_typescript: Script,
        // add_to_input: bool,
    ) -> Result<(CellOutput, Bytes), String> {
        // let genesis_info = self.genesis_info.clone();
        let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_typescript)?
            .ok_or("cell not found")?;
        let ckb_cell = CellOutput::from(cell.output);
        let ckb_cell_data = packed::Bytes::from(cell.output_data).raw_data();
        // if add_to_input {
        //     let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
        //         get_live_cell(&mut self.rpc_client, out_point, with_data).map(|(output, _)| output)
        //     };
        //
        //     helper.add_input(
        //         cell.out_point.into(),
        //         None,
        //         &mut get_live_cell_fn,
        //         &genesis_info,
        //         true,
        //     )?;
        // }
        Ok((ckb_cell, ckb_cell_data))
    }
    pub fn get_ckb_headers(&mut self, block_numbers: Vec<u64>) -> Result<Vec<u8>> {
        let mut mol_header_vec: Vec<packed::Header> = Default::default();
        for number in block_numbers {
            let header = self
                .rpc_client
                .get_header_by_number(number)
                .map_err(|e| anyhow::anyhow!("failed to get header: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("failed to get header which is none"))?;

            mol_header_vec.push(header.inner.into());
        }
        let mol_headers = HeaderVec::new_builder().set(mol_header_vec).build();
        Ok(Vec::from(mol_headers.as_slice()))
    }
    pub fn burn(
        &mut self,
        tx_fee: u64,
        from_lockscript: Script,
        burn_sudt_amount: u128,
        token_addr: H160,
        eth_receiver_addr: H160,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();

        // add cellDeps
        {
            let outpoints = vec![
                self.settings.bridge_lockscript.outpoint.clone(),
                self.settings.bridge_typescript.outpoint.clone(),
                self.settings.sudt.outpoint.clone(),
            ];
            self.add_cell_deps(&mut helper, outpoints)?;
        }

        let sudt_typescript = get_sudt_lock_script(
            &self.settings.bridge_lockscript.code_hash,
            &self.settings.sudt.code_hash,
            token_addr,
        );

        let ckb_amount = 200;
        // gen output of eth_recipient cell
        {
            let eth_recipient_data: Bytes = eth_receiver_addr.as_bytes().to_vec().into();
            // check_capacity(ckb_amount, eth_recipient_data.len())?;
            let eth_recipient_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(ckb_amount).pack()) // check cap
                .lock(from_lockscript.clone())
                .build();
            helper.add_output(eth_recipient_output, eth_recipient_data);
        }

        helper.supply_sudt(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript.clone(),
            &self.genesis_info,
            burn_sudt_amount,
            sudt_typescript,
        )?;

        // build tx
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn transfer_sudt(
        &mut self,
        from_lockscript: Script,
        token_addr: H160,
        to_lockscript: Script,
        sudt_amount: u128,
        ckb_amount: u64,
        tx_fee: u64,
    ) -> Result<TransactionView, String> {
        let mut helper = TxHelper::default();

        // add cellDeps
        let outpoints = vec![self.settings.sudt.outpoint.clone()];
        self.add_cell_deps(&mut helper, outpoints)?;

        {
            let sudt_typescript = get_sudt_lock_script(
                &self.settings.bridge_lockscript.code_hash,
                &self.settings.sudt.code_hash,
                token_addr,
            );

            let sudt_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(ckb_amount).pack())
                .type_(Some(sudt_typescript.clone()).pack())
                .lock(to_lockscript)
                .build();

            helper.add_output(sudt_output, sudt_amount.to_le_bytes().to_vec().into());

            helper.supply_sudt(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript.clone(),
                &self.genesis_info,
                sudt_amount,
                sudt_typescript,
            )?;
        }

        // add signature to pay tx fee
        let tx = helper.supply_capacity(
            &mut self.rpc_client,
            &mut self.indexer_client,
            from_lockscript,
            &self.genesis_info,
            tx_fee,
        )?;
        Ok(tx)
    }

    pub fn get_sudt_balance(&mut self, address: String, token_addr: H160) -> Result<u128, String> {
        let addr_lockscript: Script = Address::from_str(&address)?.payload().into();

        let sudt_typescript = get_sudt_lock_script(
            &self.settings.bridge_lockscript.code_hash,
            &self.settings.sudt.code_hash,
            token_addr,
        );
        collect_sudt_amount(&mut self.indexer_client, addr_lockscript, sudt_typescript)
    }

    pub fn sign_and_send_transaction(
        &mut self,
        unsigned_tx: TransactionView,
        from_privkey: SecretKey,
    ) -> Result<String> {
        let tx = sign(unsigned_tx, &mut self.rpc_client, &from_privkey)
            .map_err(|e| anyhow!("failed to sign tx : {}", e))?;
        log::info!(
            "tx: \n{}",
            serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))?
        );
        send_tx_sync(&mut self.rpc_client, &tx, 60).map_err(|e| anyhow!(e))?;
        // let cell_typescript = tx
        //     .output(0)
        //     .ok_or_else(|| anyhow!("first output cell is none"))?
        //     .type_()
        //     .to_opt();
        // let cell_script = match cell_typescript {
        //     Some(script) => hex::encode(script.as_slice()),
        //     None => "".to_owned(),
        // };
        // let print_res = serde_json::json!({
        //     "tx_hash": hex::encode(tx.hash().as_slice()),
        //     "cell_typescript": cell_script,
        // });
        // debug!("{}", serde_json::to_string_pretty(&print_res)?);
        Ok(hex::encode(tx.hash().as_slice()))
    }
}

pub fn covert_to_h256(mut tx_hash: &str) -> Result<H256> {
    if tx_hash.starts_with("0x") || tx_hash.starts_with("0X") {
        tx_hash = &tx_hash[2..];
    }
    if tx_hash.len() % 2 != 0 {
        bail!(format!("Invalid hex string length: {}", tx_hash.len()))
    }
    let mut bytes = vec![0u8; tx_hash.len() / 2];
    hex_decode(tx_hash.as_bytes(), &mut bytes)
        .map_err(|err| anyhow!("parse hex string failed: {:?}", err))?;
    H256::from_slice(&bytes).map_err(|e| anyhow!("failed to covert tx hash: {}", e))
}

pub fn get_sudt_lock_script(
    bridge_lock_code_hash: &str,
    sudt_code_hash: &str,
    token_addr: H160,
) -> Script {
    let bridge_lockscript_code_hash =
        hex::decode(bridge_lock_code_hash).expect("wrong sudt_script code hash config");

    let bridge_lockscript: Script = Script::new_builder()
        .code_hash(Byte32::from_slice(&bridge_lockscript_code_hash).unwrap())
        .hash_type(DepType::Code.into())
        .args(token_addr.as_bytes().pack())
        .build();

    let sudt_typescript_code_hash =
        hex::decode(sudt_code_hash).expect("wrong sudt_script code hash config");
    Script::new_builder()
        .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).unwrap())
        .hash_type(DepType::Code.into())
        .args(bridge_lockscript.calc_script_hash().as_bytes().pack())
        .build()
}

pub fn parse_privkey(privkey: &SecretKey) -> Script {
    let public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, privkey);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    Script::from(&address_payload)
}

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
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

impl TryFrom<ETHSPVProofJson> for witness::ETHSPVProof {
    type Error = anyhow::Error;
    fn try_from(proof: ETHSPVProofJson) -> Result<Self> {
        let mut proof_vec: Vec<basic::Bytes> = vec![];
        for i in 0..proof.proof.len() {
            proof_vec.push(proof.proof[i].to_vec().into())
        }
        Ok(witness::ETHSPVProof::new_builder()
            .log_index(proof.log_index.into())
            .log_entry_data(hex::decode(clear_0x(&proof.log_entry_data))?.into())
            .receipt_index(proof.receipt_index.into())
            .receipt_data(hex::decode(clear_0x(&proof.receipt_data))?.into())
            .header_data(hex::decode(clear_0x(&proof.header_data))?.into())
            .proof(BytesVec::new_builder().set(proof_vec).build())
            .build())
    }
}

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

#[derive(Clone)]
pub struct EthWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: ETHSPVProofJson,
}

impl EthWitness {
    pub fn as_bytes(&self) -> Bytes {
        let spv_proof: witness::ETHSPVProof = self
            .spv_proof
            .clone()
            .try_into()
            .expect("try into mint_xt_witness::ETHSPVProof success");
        let spv_proof = spv_proof.as_slice().to_vec();
        let witness_data = witness::MintTokenWitness::new_builder()
            .spv_proof(spv_proof.into())
            .cell_dep_index_list(self.cell_dep_index_list.clone().into())
            .build();
        let witness = WitnessArgs::new_builder()
            .input_type(Some(witness_data.as_bytes()).pack())
            .build();
        witness.as_bytes()
    }
}
