use crate::generated::eth_recipient_cell::{ETHRecipientCellData, ETHRecipientCellDataReader};
use core::result::Result;
use molecule::{
    bytes::Bytes,
    error::VerificationError,
    prelude::{Builder, Entity, Reader},
};

#[derive(Debug, Clone)]
pub struct ETHRecipientDataView {
    eth_recipient_address: Bytes,
    eth_token_address: Bytes,
    token_amount: u128,
}

impl ETHRecipientDataView {
    pub fn new(data: &[u8]) -> Result<ETHRecipientDataView, VerificationError> {
        ETHRecipientCellDataReader::verify(data, false)?;

        let data_reader = ETHRecipientCellDataReader::new_unchecked(data);

        let eth_recipient_address = data_reader.eth_recipient_address().to_entity().raw_data();
        let eth_token_address = data_reader.eth_token_address().to_entity().raw_data();
        let mut token_amount = [0u8; 16];
        token_amount.copy_from_slice(data_reader.token_amount().raw_data());
        let token_amount: u128 = u128::from_le_bytes(token_amount);

        Ok(ETHRecipientDataView {
            eth_recipient_address,
            eth_token_address,
            token_amount,
        })
    }

    pub fn as_molecule_data(&self) -> Result<Bytes, VerificationError> {
        let mol_obj = ETHRecipientCellData::new_builder()
            .eth_recipient_address(self.eth_recipient_address.to_vec().into())
            .eth_token_address(self.eth_token_address.to_vec().into())
            .token_amount(self.token_amount.into())
            .build();
        Ok(mol_obj.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::ETHRecipientDataView;
    use molecule::bytes::Bytes;
    use molecule::prelude::*;

    fn str_to_bytes(s: &str) -> Bytes {
        Bytes::copy_from_slice(s.as_bytes())
    }

    #[test]
    fn test_eth_recipient_data() {
        let eth_recipient_data = ETHRecipientDataView {
            eth_recipient_address: Bytes::new(),
            eth_token_address: str_to_bytes("0x734Ac651Dd95a339c633cdEd410228515F97fAfF"),
            token_amount: 100,
        };

        let mol_data = eth_recipient_data.as_molecule_data().unwrap();

        let new_eth_recipient_data = ETHRecipientDataView::new(mol_data.as_ref()).unwrap();

        assert_eq!(
            eth_recipient_data.token_amount,
            new_eth_recipient_data.token_amount
        );
        assert_eq!(
            eth_recipient_data.eth_recipient_address,
            new_eth_recipient_data.eth_recipient_address
        );
        assert_eq!(
            eth_recipient_data.eth_token_address,
            new_eth_recipient_data.eth_token_address
        );
    }
}
