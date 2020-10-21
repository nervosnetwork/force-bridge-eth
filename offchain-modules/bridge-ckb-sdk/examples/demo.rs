use anyhow::Result;
use ckb_hash::blake2b_256;
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, SECP256K1};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use ckb_types::{core::TransactionView, packed::Script};
use molecule::prelude::{Builder, Entity};
use std::str::FromStr;
use tockb_sdk::settings::{BtcDifficulty, OutpointConf, PriceOracle, ScriptConf};
use tockb_sdk::tx_helper::{deploy, sign};
use tockb_sdk::util::{ensure_indexer_sync, send_tx_sync};
use tockb_sdk::{generator::Generator, indexer::IndexerRpcClient, settings::Settings};
use tockb_types::generated::btc_difficulty::BTCDifficulty;

const TIMEOUT: u64 = 60;

fn main() -> Result<()> {
    env_logger::init();

    let rpc_url = "http://127.0.0.1:8114".to_owned();
    let indexer_url = "http://127.0.0.1:8116".to_owned();
    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let mut indexer_client = IndexerRpcClient::new(indexer_url.clone());

    let private_key_hex = "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
    let private_key = secp256k1::SecretKey::from_str(private_key_hex)?;
    let public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &private_key);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    let from_lockscript = Script::from(&address_payload);
    dbg!(hex::encode(from_lockscript.as_slice()));

    // dev deploy
    let typescript_bin = std::fs::read("../build/release/toCKB-typescript")?;
    let lockscript_bin = std::fs::read("../build/release/toCKB-lockscript")?;
    let sudt_bin = std::fs::read("../tests/deps/simple_udt")?;
    let typescript_code_hash = blake2b_256(&typescript_bin);
    let typescript_code_hash_hex = hex::encode(&typescript_code_hash);
    let lockscript_code_hash = blake2b_256(&lockscript_bin);
    let lockscript_code_hash_hex = hex::encode(&lockscript_code_hash);
    let sudt_code_hash = blake2b_256(&sudt_bin);
    let sudt_code_hash_hex = hex::encode(&sudt_code_hash);
    let price = 10000u128;
    let btc_difficulty: u64 = 17345997805929;
    let btc_difficulty_bytes = BTCDifficulty::new_builder()
        .previous(btc_difficulty.to_le_bytes().to_vec().into())
        .current(btc_difficulty.to_le_bytes().to_vec().into())
        .build()
        .as_bytes()
        .to_vec();
    let data = vec![
        typescript_bin,
        lockscript_bin,
        sudt_bin,
        price.to_le_bytes().to_vec(),
        btc_difficulty_bytes,
    ];

    let tx = deploy(&mut rpc_client, &mut indexer_client, &private_key, data).unwrap();
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, TIMEOUT).unwrap();
    let tx_hash_hex = hex::encode(tx_hash.as_bytes());
    let settings = Settings {
        typescript: ScriptConf {
            code_hash: typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 0,
            },
        },
        lockscript: ScriptConf {
            code_hash: lockscript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 1,
            },
        },
        sudt: ScriptConf {
            code_hash: sudt_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 2,
            },
        },
        price_oracle: PriceOracle {
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 3,
            },
        },
        btc_difficulty_cell: BtcDifficulty {
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 4,
            },
        },
    };
    // dbg!(&settings);

    let user_address = "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37";
    let user_lockscript = Script::from(Address::from_str(user_address.clone()).unwrap().payload());

    // deposit request
    log::info!("deposit_request start");
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let timeout = 60;
    let tx_fee = 1000_0000;
    let mut generator = Generator::new(rpc_url, indexer_url, settings).unwrap();
    let unsigned_tx = generator
        .deposit_request(
            from_lockscript.clone(),
            tx_fee,
            user_lockscript.clone(),
            10000,
            1,
            1,
        )
        .unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &private_key).unwrap();
    send_tx_sync(&mut rpc_client, &tx, timeout).unwrap();
    let cell_typescript = tx.output(0).unwrap().type_().to_opt().unwrap();
    let cell_typescript_hash = cell_typescript.calc_script_hash();
    log::info!("cell_typescript_hash: {}", cell_typescript_hash);

    // bonding
    log::info!("bonding start");
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let signer_lockscript = user_lockscript.clone();
    let lock_address = "bc1qdekmlav7pglh3k2xm6l7s49c8d0lt5cjxgf52j".to_owned();
    let unsigned_tx = generator
        .bonding(
            from_lockscript.clone(),
            tx_fee,
            cell_typescript.clone(),
            signer_lockscript,
            lock_address,
        )
        .unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &private_key).unwrap();
    log::info!(
        "tx: {}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .unwrap()
    );
    send_tx_sync(&mut rpc_client, &tx, timeout).unwrap();

    // mint_xt
    log::info!("mint_xt start");
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let spv_proof = hex::decode("900200002c000000300000005e000000a4000000a8000000c8000000d000000024010000880200008c020000020000002a00000001e120d5cc806577ed5d84a9da694f149f19e9229192818285906f4fa4d286ff7a0100000000ffffffff4200000002e0e60b00000000001976a914d51c2f82cef88dcbe6078198b59eaf923369a8dd88ac3d302b27000000001600146e6dbff59e0a3f78d946debfe854b83b5ff5d31200000000f8ea36b3298c05167889ab673d972da05ac001e2303bb4da3fe0d9ba5dae89131d00000000000000500000000000002098c981cb10662d3a815f23e79b24799415ba5d26de000d000000000000000000f3aa3ee9c06ea2e93150c7d7a8e67dea364d3168b617d3d6076ad5226c7073c794a1635f123a1017f1e97f0360010000beba0e94e6866d93db9bb095670dedb65c9b606e3762667447dd1ab134a54c97997d1b9108c23d4c46077ccd28844ca6fe4d60013c0ef1abd7a39b987e5ba5388088b5763da685292cab37dfe7281ceea637ed41a8aaf6d258c39a38fa30e92a5b7f5bb70a4bac9d19b7adf60c796aa677005d481123e6cd7d7b1d79aa9b79663c1b3dc533bc5c771324bda14770688bf81e4ec47f54cb48d8c6b9bf35f429b4fe348f7951390ddab9abf6952bc0deacaae675aecc1e1999666ade2e4ce9b6c1e47bf286f7871390fd3c1b66b3aadead3aa436ac5f4e496ab4b9a811b88580d3ffc09d21b994caf4abb98bb5058b12cbcef124279708a39e4683bc35371f25be14cc4826d97f7853e9612e431071a00ad09d6d219cf96ac7097733b4f8ff6dcaa591dea1579769d7a1d47f34ee1b25975789208745e9dbda47df60cbccbbbf9aa7fcaf28407724c9b06b8ef4cb3080caaf5d664a5fcfce36098c01b45b0b49e10100000000000000").unwrap();
    let unsigned_tx = generator
        .mint_xt(
            from_lockscript.clone(),
            tx_fee,
            cell_typescript.clone(),
            spv_proof,
        )
        .unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &private_key).unwrap();
    log::info!(
        "tx: {}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .unwrap()
    );
    send_tx_sync(&mut rpc_client, &tx, timeout).unwrap();

    // pre_term_redeem
    log::info!("pre_term_redeem start");
    ensure_indexer_sync(&mut rpc_client, &mut indexer_client, 60).unwrap();
    let unlock_address = "bc1qy90wlm8mujjuud6qs665gjp7hvn67ekef62aer".to_owned();
    let redeemer_address = "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37";
    let redeemer_lockscript = Script::from(
        Address::from_str(redeemer_address.clone())
            .unwrap()
            .payload(),
    );
    let unsigned_tx = generator
        .pre_term_redeem(
            from_lockscript.clone(),
            tx_fee,
            cell_typescript.clone(),
            unlock_address,
            redeemer_lockscript,
        )
        .unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &private_key).unwrap();
    send_tx_sync(&mut rpc_client, &tx, timeout).unwrap();

    Ok(())
}
