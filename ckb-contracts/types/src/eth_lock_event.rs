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

#[derive(Debug, Clone, PartialEq)]
pub struct ETHLockEvent {
    pub token: [u8; 20],
    pub sender: [u8; 20],
    pub locked_amount: [u8; 32],
    pub bridge_fee: [u8; 32],
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_outpoint: [u8; 36],
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
        let mut locked_amount = [0u8; 32];
        locked_amount.copy_from_slice(&data[32 * 2..32 * 3]);
        let mut bridge_fee = [0u8; 32];
        bridge_fee.copy_from_slice(&data[32 * 3..32 * 4]);
        let recipient_lockscript_offset =
            parse_be_bytes_to_u64(&data[32 * 4..32 * 5]).unwrap() as usize;
        let replay_resist_outpoint_offset =
            parse_be_bytes_to_u64(&data[32 * 5..32 * 6]).unwrap() as usize;
        let sudt_extra_data_offset = parse_be_bytes_to_u64(&data[32 * 6..32 * 7]).unwrap() as usize;
        let recipient_lockscript_len = parse_be_bytes_to_u64(
            &data[recipient_lockscript_offset..(recipient_lockscript_offset + 32)],
        )
        .unwrap() as usize;
        let replay_resist_outpoint_len = parse_be_bytes_to_u64(
            &data[replay_resist_outpoint_offset..(replay_resist_outpoint_offset + 32)],
        )
        .unwrap() as usize;
        let sudt_extra_data_len =
            parse_be_bytes_to_u64(&data[sudt_extra_data_offset..(sudt_extra_data_offset + 32)])
                .unwrap() as usize;
        let recipient_lockscript = data[(recipient_lockscript_offset + 32)
            ..(recipient_lockscript_offset + 32 + recipient_lockscript_len)]
            .to_vec();
        debug_assert_eq!(replay_resist_outpoint_len, 36);
        let mut replay_resist_outpoint = [0u8; 36];
        replay_resist_outpoint.copy_from_slice(
            &data[(replay_resist_outpoint_offset + 32)
                ..(replay_resist_outpoint_offset + 32 + replay_resist_outpoint_len)],
        );
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

fn parse_be_bytes_to_u128(data: &[u8]) -> Result<u128, String> {
    let len = data.len();
    if len != 32 {
        return Err(format!("input data should be 32 bytes"));
    }
    if data[..(len - 128 / 8)].iter().any(|&b| b != 0u8) {
        return Err(format!("data overflow"));
    }
    let mut be_bytes = [0u8; 128 / 8];
    be_bytes.copy_from_slice(&data[(len - 128 / 8)..]);
    Ok(u128::from_be_bytes(be_bytes))
}

fn parse_be_bytes_to_u64(data: &[u8]) -> Result<u64, String> {
    let len = data.len();
    if len != 32 {
        return Err(format!("input data should be 32 bytes"));
    }
    if data[..(len - 64 / 8)].iter().any(|&b| b != 0u8) {
        return Err(format!("data overflow"));
    }
    let mut be_bytes = [0u8; 64 / 8];
    be_bytes.copy_from_slice(&data[(len - 64 / 8)..]);
    Ok(u64::from_be_bytes(be_bytes))
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
            Token::Uint(0.into()),
            Token::Uint(0.into()),
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
            locked_amount: [0u8; 32],
            bridge_fee: [0u8; 32],
            recipient_lockscript: b"ckb_recipient_address".to_vec(),
            replay_resist_outpoint: [0x33u8; 36],
            sudt_extra_data: b"sudt_extra_data".to_vec(),
        };
        assert_eq!(eth_lock_event, expected_eth_lock_event);
    }
}
