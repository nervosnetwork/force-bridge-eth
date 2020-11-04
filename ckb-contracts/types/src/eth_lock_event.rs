/// Associated eth event:
///
/// event Locked(
///     address indexed token,
///     address indexed sender,
///     uint256 amount,
///     bytes CkbRecipientAddr,
///     bytes replayResistCellOutpoint,
/// );

#[derive(Debug, Clone, PartialEq)]
pub struct ETHLockEvent {
    pub token_address: [u8; 20],
    pub token_sender: [u8; 20],
    pub token_amount: [u8; 32],
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_cell_id: [u8; 36],
}

impl ETHLockEvent {
    pub fn parse_from_event_data(data: &[u8]) -> Self {
        debug_assert_eq!(data.len() % 32, 0);
        // 32 + 32 + 32 + 32 * 3 + 32 * 3
        debug_assert!(data.len() >= 288, "event data invalid");
        let mut token_address = [0u8; 20];
        let mut token_sender = [0u8; 20];
        token_address.copy_from_slice(&data[12..32]);
        token_sender.copy_from_slice(&data[44..64]);
        let mut token_amount = [0u8; 32];
        token_amount.copy_from_slice(&data[32 * 2..32 * 3]);
        let recipient_lockscript_offset =
            parse_be_bytes_to_u64(&data[32 * 3..32 * 4]).unwrap() as usize;
        let replay_resist_cell_id_offset =
            parse_be_bytes_to_u64(&data[32 * 4..32 * 5]).unwrap() as usize;
        let recipient_lockscript_len = parse_be_bytes_to_u64(
            &data[recipient_lockscript_offset..(recipient_lockscript_offset + 32)],
        )
        .unwrap() as usize;
        let replay_resist_cell_id_len = parse_be_bytes_to_u64(
            &data[replay_resist_cell_id_offset..(replay_resist_cell_id_offset + 32)],
        )
        .unwrap() as usize;
        let recipient_lockscript = data[(recipient_lockscript_offset + 32)
            ..(recipient_lockscript_offset + 32 + recipient_lockscript_len)]
            .to_vec();
        debug_assert_eq!(replay_resist_cell_id_len, 36);
        let mut replay_resist_cell_id = [0u8; 36];
        replay_resist_cell_id.copy_from_slice(
            &data[(replay_resist_cell_id_offset + 32)
                ..(replay_resist_cell_id_offset + 32 + replay_resist_cell_id_len)],
        );
        Self {
            token_address,
            token_sender,
            token_amount,
            recipient_lockscript,
            replay_resist_cell_id,
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

    const abi: [ParamType; 5] = [
        ParamType::Address,
        ParamType::Address,
        ParamType::Uint(256),
        ParamType::Bytes,
        ParamType::Bytes,
    ];

    #[test]
    fn test_parse_from_event_data() {
        let tokens = [
            Token::Address([0x11u8; 20].into()),
            Token::Address([0x22u8; 20].into()),
            Token::Uint(0.into()),
            Token::Bytes(b"ckb_recipient_address".to_vec()),
            Token::Bytes((0..36).map(|_| 0x33u8).collect::<Vec<_>>()),
        ];
        let data = encode(&tokens);
        let decoded_tokens = decode(&abi, data.as_ref()).unwrap();
        assert_eq!(tokens.as_ref(), &decoded_tokens);
        let eth_lock_event = ETHLockEvent::parse_from_event_data(data.as_ref());
        let expected_eth_lock_event = ETHLockEvent {
            token_address: [0x11u8; 20],
            token_sender: [0x22u8; 20],
            token_amount: [0u8; 32],
            recipient_lockscript: b"ckb_recipient_address".to_vec(),
            replay_resist_cell_id: [0x33u8; 36],
        };
        assert_eq!(eth_lock_event, expected_eth_lock_event);
    }
}
