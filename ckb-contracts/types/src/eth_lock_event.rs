#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]

/// Associated eth event:
///
/// event Locked(
///     address indexed token,
///     address indexed sender,
///     uint256 lockedAmount,
///     uint256 bridgeFee,
///     bytes recipientLockscript,
///     bytes replayResistOutpoint,
///     bytes sudtExtraData
/// );
///
/// references:
/// - https://ethereum.github.io/yellowpaper/paper.pdf
/// - https://docs.soliditylang.org/en/latest/abi-spec.html?highlight=event#events
///
use eth_spv_lib::eth_types::LogEntry;
use ethereum_types::U256;
use std::convert::TryInto;
use std::prelude::v1::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ETHLockEvent {
    pub contract_address: [u8; 20],
    pub token: [u8; 20],
    pub sender: [u8; 20],
    pub locked_amount: U256,
    pub bridge_fee: U256,
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_outpoint: Vec<u8>,
    pub sudt_extra_data: Vec<u8>,
}

impl ETHLockEvent {
    pub fn parse_from_event_data(log: &LogEntry) -> Self {
        let data = log.data.as_slice();
        debug_assert_eq!(data.len() % 32, 0);
        let topics_len = log.topics.len();
        let mut token = [0u8; 20];
        let mut sender = [0u8; 20];
        token.copy_from_slice(&log.topics[topics_len - 2].0.as_ref()[12..32]);
        sender.copy_from_slice(&log.topics[topics_len - 1].0.as_ref()[12..32]);
        let locked_amount = U256::from_big_endian(&data[32 * 0..32 * 1]);
        let bridge_fee = U256::from_big_endian(&data[32 * 1..32 * 2]);
        let recipient_lockscript_offset: usize = U256::from_big_endian(&data[32 * 2..32 * 3])
            .try_into()
            .unwrap();
        let replay_resist_outpoint_offset: usize = U256::from_big_endian(&data[32 * 3..32 * 4])
            .try_into()
            .unwrap();
        let sudt_extra_data_offset: usize = U256::from_big_endian(&data[32 * 4..32 * 5])
            .try_into()
            .unwrap();
        let recipient_lockscript_len: usize = U256::from_big_endian(
            &data[recipient_lockscript_offset..(recipient_lockscript_offset + 32)],
        )
        .try_into()
        .unwrap();
        let replay_resist_outpoint_len: usize = U256::from_big_endian(
            &data[replay_resist_outpoint_offset..(replay_resist_outpoint_offset + 32)],
        )
        .try_into()
        .unwrap();
        let sudt_extra_data_len: usize =
            U256::from_big_endian(&data[sudt_extra_data_offset..(sudt_extra_data_offset + 32)])
                .try_into()
                .unwrap();
        let recipient_lockscript = data[(recipient_lockscript_offset + 32)
            ..(recipient_lockscript_offset + 32 + recipient_lockscript_len)]
            .to_vec();
        debug_assert_eq!(replay_resist_outpoint_len, 36);
        let replay_resist_outpoint = data[(replay_resist_outpoint_offset + 32)
            ..(replay_resist_outpoint_offset + 32 + replay_resist_outpoint_len)]
            .to_vec();
        let sudt_extra_data = data
            [(sudt_extra_data_offset + 32)..(sudt_extra_data_offset + 32 + sudt_extra_data_len)]
            .to_vec();

        let contract_address = (log.address.0).0;
        Self {
            contract_address,
            token,
            sender,
            locked_amount,
            bridge_fee,
            recipient_lockscript,
            replay_resist_outpoint,
            sudt_extra_data,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_from_event_data() {
        let log_entry_data = "f9024194eab52f0d5c0f03c273372309b522a935f5b6ce12f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000003dc3d2369b6d9879e593c3a133055e0f03a52a74a000000000000000000000000046beac96b726a51c5703f99ec787ce12793dae11b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000c8328aabcd9b9e8e64fbc566c4385c3bdeb219d70000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002418fe25034cbc1f69c53df12f2083a23091d2e1830b911ae873265f03dd61b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000083307832";
        let log_entry: LogEntry = rlp::decode(&hex::decode(log_entry_data).unwrap()).unwrap();
        let eth_lock_event = ETHLockEvent::parse_from_event_data(&log_entry);
        // dbg!(&eth_lock_event);
        assert_eq!(
            eth_lock_event.token.to_vec(),
            hex::decode("3dc3d2369b6d9879e593c3a133055e0f03a52a74").unwrap()
        );
        assert_eq!(
            eth_lock_event.sender.to_vec(),
            hex::decode("46beac96b726a51c5703f99ec787ce12793dae11").unwrap()
        );
        assert_eq!(eth_lock_event.locked_amount, 100.into());
        assert_eq!(eth_lock_event.bridge_fee, 10.into());
        assert_eq!(
            eth_lock_event.recipient_lockscript,
            vec![
                73u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 155, 215, 224, 111, 62, 207,
                75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200, 142, 93, 75, 101, 168, 99,
                123, 23, 114, 59, 189, 163, 204, 232, 1, 20, 0, 0, 0, 200, 50, 138, 171, 205, 155,
                158, 142, 100, 251, 197, 102, 196, 56, 92, 59, 222, 178, 25, 215
            ]
        );
        assert_eq!(
            eth_lock_event.replay_resist_outpoint,
            vec![
                24u8, 254, 37, 3, 76, 188, 31, 105, 197, 61, 241, 47, 32, 131, 162, 48, 145, 210,
                225, 131, 11, 145, 26, 232, 115, 38, 95, 3, 221, 97, 176, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(eth_lock_event.sudt_extra_data, b"sudt_extra_data".to_vec());
    }
}
