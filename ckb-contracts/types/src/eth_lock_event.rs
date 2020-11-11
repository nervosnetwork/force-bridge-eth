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
use ethereum_types::U256;
use std::convert::TryInto;
use std::prelude::v1::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ETHLockEvent {
    pub token: [u8; 20],
    pub sender: [u8; 20],
    pub locked_amount: U256,
    pub bridge_fee: U256,
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_outpoint: Vec<u8>,
    pub sudt_extra_data: Vec<u8>,
}

impl ETHLockEvent {
    pub fn parse_from_event_data(data: &[u8]) -> Self {
        debug_assert_eq!(data.len() % 32, 0);
        // 32 + 32 + 32 + 32 + 32 * 3 * 3
        debug_assert!(data.len() >= 448, "event data invalid");
        let mut token = [0u8; 20];
        let mut sender = [0u8; 20];
        token.copy_from_slice(&data[12..32]);
        sender.copy_from_slice(&data[44..64]);
        let locked_amount = U256::from_big_endian(&data[32 * 2..32 * 3]);
        let bridge_fee = U256::from_big_endian(&data[32 * 3..32 * 4]);
        let recipient_lockscript_offset: usize = U256::from_big_endian(&data[32 * 4..32 * 5])
            .try_into()
            .unwrap();
        let replay_resist_outpoint_offset: usize = U256::from_big_endian(&data[32 * 5..32 * 6])
            .try_into()
            .unwrap();
        let sudt_extra_data_offset: usize = U256::from_big_endian(&data[32 * 6..32 * 7])
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
        Self {
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
    use ethabi::{decode, encode, ParamType, Token};

    const abi: [ParamType; 7] = [
        ParamType::Address,
        ParamType::Address,
        ParamType::Uint(256),
        ParamType::Uint(256),
        ParamType::Bytes,
        ParamType::Bytes,
        ParamType::Bytes,
    ];

    #[test]
    fn test_parse_from_event_data() {
        let tokens = [
            Token::Address([0x11u8; 20].into()),
            Token::Address([0x22u8; 20].into()),
            Token::Uint(100.into()),
            Token::Uint(2.into()),
            Token::Bytes(b"ckb_recipient_address".to_vec()),
            Token::Bytes((0..36).map(|_| 0x33u8).collect::<Vec<_>>()),
            Token::Bytes(b"sudt_extra_data".to_vec()),
        ];
        let data = encode(&tokens);
        dbg!(data.len());
        dbg!(data
            .chunks(32)
            .map(|s| hex::encode(s))
            .collect::<Vec<String>>());
        let decoded_tokens = decode(&abi, data.as_ref()).unwrap();
        assert_eq!(tokens.as_ref(), &decoded_tokens);
        let eth_lock_event = ETHLockEvent::parse_from_event_data(data.as_ref());
        let expected_eth_lock_event = ETHLockEvent {
            token: [0x11u8; 20],
            sender: [0x22u8; 20],
            locked_amount: 100.into(),
            bridge_fee: 2.into(),
            recipient_lockscript: b"ckb_recipient_address".to_vec(),
            replay_resist_outpoint: [0x33u8; 36].to_vec(),
            sudt_extra_data: b"sudt_extra_data".to_vec(),
        };
        assert_eq!(eth_lock_event, expected_eth_lock_event);
    }
}
