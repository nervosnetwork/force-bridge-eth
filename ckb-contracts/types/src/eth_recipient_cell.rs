use crate::generated::eth_recipient_cell::{ETHRecipientCellData, ETHRecipientCellDataReader};
use core::convert::TryFrom;
use core::result::Result;
use molecule::{
    bytes::Bytes,
    error::VerificationError,
    prelude::Byte,
    prelude::{Builder, Entity, Reader},
};

#[cfg(not(feature = "std"))]
use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::convert::TryInto;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ETHAddress([Byte; 20]);

impl TryFrom<Vec<u8>> for ETHAddress {
    type Error = VerificationError;
    fn try_from(v: Vec<u8>) -> Result<Self, VerificationError> {
        if v.len() != 20 {
            return Err(VerificationError::TotalSizeNotMatch(
                "ETHAddress".to_owned(),
                20,
                v.len(),
            ));
        }
        let mut inner = ETHAddress::default();
        let v = v.into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.0.copy_from_slice(&v);
        Ok(inner)
    }
}

impl ETHAddress {
    pub fn get_address(&self) -> [Byte; 20] {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct ETHRecipientDataView {
    pub eth_recipient_address: ETHAddress,
    pub eth_token_address: ETHAddress,
    pub eth_lock_contract_address: ETHAddress,
    pub light_client_typescript_hash: [u8; 32],
    pub eth_bridge_lock_hash: [u8; 32],
    pub token_amount: u128,
    pub fee: u128,
}

impl ETHRecipientDataView {
    pub fn new(data: &[u8]) -> Result<ETHRecipientDataView, VerificationError> {
        ETHRecipientCellDataReader::verify(data, false)?;
        let data_reader = ETHRecipientCellDataReader::new_unchecked(data);

        let eth_recipient_address = ETHAddress::try_from(
            data_reader
                .eth_recipient_address()
                .to_entity()
                .raw_data()
                .to_vec(),
        )
        .expect("wrong eth address length");
        let eth_token_address = ETHAddress::try_from(
            data_reader
                .eth_token_address()
                .to_entity()
                .raw_data()
                .to_vec(),
        )
        .expect("wrong eth address length");
        let eth_lock_contract_address = ETHAddress::try_from(
            data_reader
                .eth_lock_contract_address()
                .to_entity()
                .raw_data()
                .to_vec(),
        )
        .expect("wrong eth address length");

        let mut light_client_typescript_hash = [0u8; 32];
        light_client_typescript_hash
            .copy_from_slice(data_reader.light_client_typescript_hash().raw_data());

        let mut eth_bridge_lock_hash = [0u8; 32];
        eth_bridge_lock_hash.copy_from_slice(data_reader.eth_bridge_lock_hash().raw_data());

        let mut token_amount = [0u8; 16];
        token_amount.copy_from_slice(data_reader.token_amount().raw_data());
        let token_amount: u128 = u128::from_le_bytes(token_amount);

        let mut fee = [0u8; 16];
        fee.copy_from_slice(data_reader.fee().raw_data());
        let fee: u128 = u128::from_le_bytes(fee);

        Ok(ETHRecipientDataView {
            eth_recipient_address,
            eth_token_address,
            eth_lock_contract_address,
            light_client_typescript_hash,
            eth_bridge_lock_hash,
            token_amount,
            fee,
        })
    }

    pub fn as_molecule_data(&self) -> Result<Bytes, VerificationError> {
        let mol_obj = ETHRecipientCellData::new_builder()
            .eth_recipient_address(self.eth_recipient_address.0.into())
            .eth_token_address(self.eth_token_address.0.into())
            .eth_lock_contract_address(self.eth_lock_contract_address.0.into())
            .light_client_typescript_hash(
                self.light_client_typescript_hash
                    .to_vec()
                    .try_into()
                    .expect("from vec to Byte32 fail"),
            )
            .eth_bridge_lock_hash(
                self.eth_bridge_lock_hash
                    .to_vec()
                    .try_into()
                    .expect("from vec to Byte32 fail"),
            )
            .token_amount(self.token_amount.into())
            .fee(self.fee.into())
            .build();
        Ok(mol_obj.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{ETHAddress, ETHRecipientDataView};
    use core::convert::TryFrom;

    #[test]
    fn test_eth_recipient_data() {
        let eth_recipient_data = ETHRecipientDataView {
            eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_lock_contract_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            light_client_typescript_hash: [2u8; 32],
            eth_bridge_lock_hash: [1u8; 32],
            token_amount: 100,
            fee: 100,
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
        assert_eq!(
            eth_recipient_data.eth_lock_contract_address,
            new_eth_recipient_data.eth_lock_contract_address
        );
        assert_eq!(
            eth_recipient_data.light_client_typescript_hash,
            new_eth_recipient_data.light_client_typescript_hash
        );
        assert_eq!(
            eth_recipient_data.eth_bridge_lock_hash,
            new_eth_recipient_data.eth_bridge_lock_hash
        );
    }

    #[test]
    #[should_panic]
    fn test_eth_recipient_data_when_eth_address_length_wrong() {
        ETHAddress::try_from(vec![0; 21]).unwrap();
    }
}
