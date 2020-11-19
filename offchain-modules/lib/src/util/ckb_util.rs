use crate::transfer::to_ckb::build_eth_bridge_lock_args;
use crate::util::eth_proof_helper::Witness;
use crate::util::settings::{OutpointConf, Settings};
use anyhow::{anyhow, bail, Result};
use ckb_sdk::{Address, AddressPayload, GenesisInfo, HttpRpcClient, SECP256K1};
use ckb_types::core::{BlockView, Capacity, DepType, TransactionView};
use ckb_types::packed::{HeaderVec, ScriptReader, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack, Reader};
use ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, CellDep, CellOutput, OutPoint, Script},
    H256,
};
use ethereum_types::H160;
use faster_hex::hex_decode;
use force_eth_types::eth_recipient_cell::{ETHAddress, ETHRecipientDataView};
use force_eth_types::generated::basic::BytesVec;
use force_eth_types::generated::eth_bridge_lock_cell::ETHBridgeLockArgs;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};
use force_eth_types::generated::{basic, witness};
use force_sdk::cell_collector::{collect_sudt_amount, get_live_cell_by_typescript};
use force_sdk::indexer::{Cell, IndexerRpcClient};
use force_sdk::tx_helper::{sign, TxHelper};
use force_sdk::util::{get_live_cell_with_cache, send_tx_sync};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
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
        _cell_dep: String,
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

            // let cell_script = parse_cell(cell_dep.as_str())?;
            // let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_script)
            //     .map_err(|err| anyhow!(err))?
            //     .ok_or_else(|| anyhow!("no cell found for cell dep"))?;
            // let mut builder = helper.transaction.as_advanced_builder();
            // builder = builder.cell_dep(
            //     CellDep::new_builder()
            //         .out_point(cell.out_point.into())
            //         .dep_type(DepType::Code.into())
            //         .build(),
            // );
            // helper.transaction = builder.build();
        }

        let lockscript_code_hash = hex::decode(&self.settings.bridge_lockscript.code_hash)?;
        dbg!(&eth_proof.token);
        use force_eth_types::generated::basic::ETHAddress;
        let args = ETHBridgeLockArgs::new_builder()
            .eth_token_address(
                ETHAddress::from_slice(&eth_proof.token.as_bytes()).map_err(|err| anyhow!(err))?,
            )
            .eth_contract_address(
                ETHAddress::from_slice(&eth_proof.eth_address.as_bytes())
                    .map_err(|err| anyhow!(err))?,
            )
            .build();
        let lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&lockscript_code_hash)?)
            .hash_type(DepType::Code.into())
            .args(args.as_bytes().pack())
            .build();

        // input bridge cells
        let rpc_client = &mut self.rpc_client;
        let mut live_cell_cache: HashMap<(OutPoint, bool), (CellOutput, Bytes)> =
            Default::default();
        let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
            get_live_cell_with_cache(&mut live_cell_cache, rpc_client, out_point, with_data)
                .map(|(output, _)| output)
        };
        let outpoint = OutPoint::from_slice(&eth_proof.replay_resist_outpoint)
            .expect("replay resist outpoint in lock event is invalid");
        helper
            .add_input(
                outpoint.clone(),
                None,
                &mut get_live_cell_fn,
                &self.genesis_info,
                true,
            )
            .map_err(|err| anyhow!(err))?;

        let (_, bridge_cell_data) =
            get_live_cell_with_cache(&mut live_cell_cache, &mut self.rpc_client, outpoint, true)
                .expect("outpoint not exists");
        let owner_lock_script = ETHBridgeTypeData::from_slice(bridge_cell_data.as_ref())
            .expect("invalid bridge data")
            .owner_lock_script();
        assert_eq!(owner_lock_script.raw_data(), from_lockscript.as_bytes());
        // 1 bridge cells
        // {
        //     let to_output = CellOutput::new_builder().lock(lockscript.clone()).build();
        //     helper.add_output_with_auto_capacity(to_output, ckb_types::bytes::Bytes::default());
        // }
        // 2 xt cells
        {
            let recipient_lockscript = Script::from_slice(&eth_proof.recipient_lockscript).unwrap();

            let sudt_typescript_code_hash = hex::decode(&self.settings.sudt.code_hash)?;
            let sudt_typescript = Script::new_builder()
                .code_hash(Byte32::from_slice(&sudt_typescript_code_hash)?)
                .hash_type(DepType::Code.into())
                .args(lockscript.calc_script_hash().as_bytes().pack())
                .build();

            // recipient
            dbg!(&recipient_lockscript, &from_lockscript);
            let sudt_user_output = CellOutput::new_builder()
                .type_(Some(sudt_typescript.clone()).pack())
                .lock(recipient_lockscript)
                .build();
            let mut to_user_amount_data = (eth_proof.lock_amount - eth_proof.bridge_fee)
                .to_le_bytes()
                .to_vec();
            to_user_amount_data.extend(eth_proof.sudt_extra_data.clone());
            helper.add_output_with_auto_capacity(sudt_user_output, to_user_amount_data.into());
            // fee
            let sudt_fee_output = CellOutput::new_builder()
                .type_(Some(sudt_typescript).pack())
                .lock(from_lockscript.clone())
                .build();
            helper.add_output_with_auto_capacity(
                sudt_fee_output,
                eth_proof.bridge_fee.to_le_bytes().to_vec().into(),
            );
        }
        // add witness
        {
            let witness = EthWitness {
                cell_dep_index_list: vec![0],
                spv_proof: eth_proof.clone(),
            }
            .as_bytes();
            log::debug!("witness: {}", hex::encode(witness.as_ref()));
            helper.transaction = helper
                .transaction
                .as_advanced_builder()
                .witness(witness.pack())
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

    #[allow(clippy::too_many_arguments)]
    pub fn create_bridge_cell(
        &mut self,
        tx_fee: u64,
        from_lockscript: Script,
        eth_token_address: H160,
        eth_contract_address: H160,
        recipient_lockscript: Script,
        bridge_fee: u128,
    ) -> Result<TransactionView> {
        let mut tx_helper = TxHelper::default();
        // add cell deps
        let outpoints = vec![
            self.settings.bridge_lockscript.outpoint.clone(),
            self.settings.bridge_typescript.outpoint.clone(),
        ];
        self.add_cell_deps(&mut tx_helper, outpoints)
            .map_err(|err| anyhow!(err))?;
        // build lockscript
        let bridge_lockscript_args =
            build_eth_bridge_lock_args(eth_token_address, eth_contract_address)?;
        let bridge_lockscript = Script::new_builder()
            .code_hash(Byte32::from_slice(&hex::decode(
                &self.settings.bridge_lockscript.code_hash,
            )?)?)
            .args(bridge_lockscript_args.as_bytes().pack())
            .build();
        // build typescript
        let bridge_typescript_args = ETHBridgeTypeArgs::new_builder()
            .bridge_lock_hash(
                basic::Byte32::from_slice(bridge_lockscript.calc_script_hash().as_slice()).unwrap(),
            )
            .recipient_lock_hash(
                basic::Byte32::from_slice(recipient_lockscript.calc_script_hash().as_slice())
                    .unwrap(),
            )
            .build();
        let bridge_data = ETHBridgeTypeData::new_builder()
            .owner_lock_script(from_lockscript.as_slice().to_vec().into())
            .fee(bridge_fee.into())
            .build();
        let bridge_typescript = Script::new_builder()
            .code_hash(Byte32::from_slice(
                &hex::decode(&self.settings.bridge_typescript.code_hash).unwrap(),
            )?)
            .args(bridge_typescript_args.as_bytes().pack())
            .build();
        // build output
        let capacity: u64 = 500_0000_0000;
        let output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .type_(Some(bridge_typescript).pack())
            .lock(bridge_lockscript)
            .build();
        tx_helper.add_output(output, bridge_data.as_bytes());
        // build tx
        let tx = tx_helper
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

    #[allow(clippy::too_many_arguments)]
    pub fn burn(
        &mut self,
        tx_fee: u64,
        from_lockscript: Script,
        unlock_fee: u128,
        burn_sudt_amount: u128,
        token_addr: H160,
        lock_contract_addr: H160,
        eth_receiver_addr: H160,
    ) -> Result<TransactionView> {
        let mut helper = TxHelper::default();

        // add cellDeps
        {
            let outpoints = vec![
                self.settings.bridge_lockscript.outpoint.clone(),
                self.settings.recipient_typescript.outpoint.clone(),
                self.settings.sudt.outpoint.clone(),
            ];
            self.add_cell_deps(&mut helper, outpoints)
                .map_err(|err| anyhow!(err))?;
        }

        let sudt_typescript = get_sudt_type_script(
            &self.settings.bridge_lockscript.code_hash,
            &self.settings.sudt.code_hash,
            token_addr,
            lock_contract_addr,
        )?;

        // gen output of eth_recipient cell
        {
            let mut eth_bridge_lock_hash = [0u8; 32];
            eth_bridge_lock_hash.copy_from_slice(
                &hex::decode(&self.settings.bridge_lockscript.code_hash)
                    .map_err(|err| anyhow!(err))?,
            );
            let eth_recipient_data = ETHRecipientDataView {
                eth_recipient_address: ETHAddress::try_from(eth_receiver_addr.as_bytes().to_vec())
                    .map_err(|err| anyhow!(err))?,
                eth_token_address: ETHAddress::try_from(token_addr.as_bytes().to_vec())
                    .map_err(|err| anyhow!(err))?,
                eth_lock_contract_address: ETHAddress::try_from(
                    lock_contract_addr.as_bytes().to_vec(),
                )
                .map_err(|err| anyhow!(err))?,
                eth_bridge_lock_hash,
                token_amount: burn_sudt_amount,
                fee: unlock_fee,
            };

            log::info!(
                "tx fee: {} burn amount : {}",
                eth_recipient_data.fee,
                eth_recipient_data.token_amount
            );

            let mol_eth_recipient_data = eth_recipient_data
                .as_molecule_data()
                .map_err(|err| anyhow!(err))?;
            let recipient_typescript_code_hash =
                hex::decode(&self.settings.recipient_typescript.code_hash)
                    .map_err(|err| anyhow!(err))?;

            let recipient_typescript: Script = Script::new_builder()
                .code_hash(Byte32::from_slice(&recipient_typescript_code_hash)?)
                .hash_type(DepType::Code.into())
                .build();

            let eth_recipient_output = CellOutput::new_builder()
                .lock(from_lockscript.clone())
                .type_(Some(recipient_typescript).pack())
                .build();
            helper.add_output_with_auto_capacity(eth_recipient_output, mol_eth_recipient_data);
        }

        helper
            .supply_sudt(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript.clone(),
                &self.genesis_info,
                burn_sudt_amount,
                sudt_typescript,
            )
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
        Ok(tx)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn transfer_sudt(
        &mut self,
        lock_contract_addr: H160,
        token_addr: H160,
        from_lockscript: Script,
        to_lockscript: Script,
        sudt_amount: u128,
        ckb_amount: u64,
        tx_fee: u64,
    ) -> Result<TransactionView> {
        let mut helper = TxHelper::default();

        // add cellDeps
        let outpoints = vec![
            self.settings.bridge_lockscript.outpoint.clone(),
            self.settings.sudt.outpoint.clone(),
        ];
        self.add_cell_deps(&mut helper, outpoints)
            .map_err(|err| anyhow!(err))?;

        let sudt_typescript = get_sudt_type_script(
            &self.settings.bridge_lockscript.code_hash,
            &self.settings.sudt.code_hash,
            token_addr,
            lock_contract_addr,
        )?;

        let sudt_output = CellOutput::new_builder()
            .capacity(Capacity::shannons(ckb_amount).pack())
            .type_(Some(sudt_typescript.clone()).pack())
            .lock(to_lockscript)
            .build();

        helper.add_output(sudt_output, sudt_amount.to_le_bytes().to_vec().into());

        helper
            .supply_sudt(
                &mut self.rpc_client,
                &mut self.indexer_client,
                from_lockscript.clone(),
                &self.genesis_info,
                sudt_amount,
                sudt_typescript,
            )
            .map_err(|err| anyhow!(err))?;

        // add signature to pay tx fee
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

    pub fn get_sudt_balance(
        &mut self,
        address: String,
        token_addr: H160,
        lock_contract_addr: H160,
    ) -> Result<u128> {
        let addr_lockscript: Script = Address::from_str(&address)
            .map_err(|err| anyhow!(err))?
            .payload()
            .into();

        let sudt_typescript = get_sudt_type_script(
            &self.settings.bridge_lockscript.code_hash,
            &self.settings.sudt.code_hash,
            token_addr,
            lock_contract_addr,
        )?;
        collect_sudt_amount(&mut self.indexer_client, addr_lockscript, sudt_typescript)
            .map_err(|err| anyhow!(err))
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

pub fn get_sudt_type_script(
    bridge_lock_code_hash: &str,
    sudt_code_hash: &str,
    token_addr: H160,
    lock_contract_addr: H160,
) -> Result<Script> {
    let bridge_lockscript_code_hash =
        hex::decode(bridge_lock_code_hash).map_err(|err| anyhow!(err))?;
    let bridge_lockscript = get_eth_bridge_lock_script(
        bridge_lockscript_code_hash.as_slice(),
        token_addr,
        lock_contract_addr,
    )?;

    let sudt_typescript_code_hash = hex::decode(sudt_code_hash).map_err(|err| anyhow!(err))?;
    Ok(Script::new_builder()
        .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).map_err(|err| anyhow!(err))?)
        .hash_type(DepType::Code.into())
        .args(bridge_lockscript.calc_script_hash().as_bytes().pack())
        .build())
}

pub fn get_eth_bridge_lock_script(
    bridge_lock_code_hash: &[u8],
    token_addr: H160,
    lock_contract_addr: H160,
) -> Result<Script> {
    let args = ETHBridgeLockArgs::new_builder()
        .eth_contract_address(
            ETHAddress::try_from(lock_contract_addr.as_bytes().to_vec())
                .map_err(|err| anyhow!(err))?
                .get_address()
                .into(),
        )
        .eth_token_address(
            ETHAddress::try_from(token_addr.as_bytes().to_vec())
                .map_err(|err| anyhow!(err))?
                .get_address()
                .into(),
        )
        .build();

    Ok(Script::new_builder()
        .code_hash(Byte32::from_slice(bridge_lock_code_hash).map_err(|err| anyhow!(err))?)
        .hash_type(DepType::Code.into())
        .args(args.as_bytes().pack())
        .build())
}

pub fn parse_privkey(privkey: &SecretKey) -> Script {
    let public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, privkey);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    Script::from(&address_payload)
}

pub fn build_outpoint(outpoint_conf: OutpointConf) -> Result<OutPoint> {
    let outpoint = OutPoint::new_builder()
        .tx_hash(
            Byte32::from_slice(&hex::decode(outpoint_conf.tx_hash).map_err(|e| anyhow!(e))?)
                .map_err(|e| anyhow!(e))?,
        )
        .index(outpoint_conf.index.pack())
        .build();
    Ok(outpoint)
}

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
}

pub fn build_lockscript_from_address(address: &str) -> Result<Script> {
    let recipient_lockscript = Script::from(
        Address::from_str(address)
            .map_err(|err| anyhow!(err))?
            .payload(),
    );
    Ok(recipient_lockscript)
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ETHSPVProofJson {
    pub log_index: u64,
    pub log_entry_data: String,
    pub receipt_index: u64,
    pub receipt_data: String,
    pub header_data: String,
    pub proof: Vec<Vec<u8>>,
    pub token: H160,
    pub lock_amount: u128,
    pub bridge_fee: u128,
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_outpoint: Vec<u8>,
    pub sudt_extra_data: Vec<u8>,
    pub eth_address: H160,
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
            .lock(Some(witness_data.as_bytes()).pack())
            .build();
        witness.as_bytes()
    }
}
