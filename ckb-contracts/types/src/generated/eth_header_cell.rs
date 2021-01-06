// Generated by Molecule 0.6.1

use super::basic::*;
use molecule::prelude::*;
#[derive(Clone)]
pub struct ETHHeaderCellData(molecule::bytes::Bytes);
impl ::core::fmt::LowerHex for ETHHeaderCellData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl ::core::fmt::Debug for ETHHeaderCellData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl ::core::fmt::Display for ETHHeaderCellData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "merkle_root", self.merkle_root())?;
        write!(f, ", {}: {}", "start_height", self.start_height())?;
        write!(f, ", {}: {}", "latest_height", self.latest_height())?;
        write!(f, " }}")
    }
}
impl ::core::default::Default for ETHHeaderCellData {
    fn default() -> Self {
        let v: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        ETHHeaderCellData::new_unchecked(v.into())
    }
}
impl ETHHeaderCellData {
    pub const TOTAL_SIZE: usize = 48;
    pub const FIELD_SIZES: [usize; 3] = [32, 8, 8];
    pub const FIELD_COUNT: usize = 3;
    pub fn merkle_root(&self) -> Byte32 {
        Byte32::new_unchecked(self.0.slice(0..32))
    }
    pub fn start_height(&self) -> Uint64 {
        Uint64::new_unchecked(self.0.slice(32..40))
    }
    pub fn latest_height(&self) -> Uint64 {
        Uint64::new_unchecked(self.0.slice(40..48))
    }
    pub fn as_reader<'r>(&'r self) -> ETHHeaderCellDataReader<'r> {
        ETHHeaderCellDataReader::new_unchecked(self.as_slice())
    }
}
impl molecule::prelude::Entity for ETHHeaderCellData {
    type Builder = ETHHeaderCellDataBuilder;
    const NAME: &'static str = "ETHHeaderCellData";
    fn new_unchecked(data: molecule::bytes::Bytes) -> Self {
        ETHHeaderCellData(data)
    }
    fn as_bytes(&self) -> molecule::bytes::Bytes {
        self.0.clone()
    }
    fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
    fn from_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        ETHHeaderCellDataReader::from_slice(slice).map(|reader| reader.to_entity())
    }
    fn from_compatible_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
        ETHHeaderCellDataReader::from_compatible_slice(slice).map(|reader| reader.to_entity())
    }
    fn new_builder() -> Self::Builder {
        ::core::default::Default::default()
    }
    fn as_builder(self) -> Self::Builder {
        Self::new_builder()
            .merkle_root(self.merkle_root())
            .start_height(self.start_height())
            .latest_height(self.latest_height())
    }
}
#[derive(Clone, Copy)]
pub struct ETHHeaderCellDataReader<'r>(&'r [u8]);
impl<'r> ::core::fmt::LowerHex for ETHHeaderCellDataReader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use molecule::hex_string;
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex_string(self.as_slice()))
    }
}
impl<'r> ::core::fmt::Debug for ETHHeaderCellDataReader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{}({:#x})", Self::NAME, self)
    }
}
impl<'r> ::core::fmt::Display for ETHHeaderCellDataReader<'r> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} {{ ", Self::NAME)?;
        write!(f, "{}: {}", "merkle_root", self.merkle_root())?;
        write!(f, ", {}: {}", "start_height", self.start_height())?;
        write!(f, ", {}: {}", "latest_height", self.latest_height())?;
        write!(f, " }}")
    }
}
impl<'r> ETHHeaderCellDataReader<'r> {
    pub const TOTAL_SIZE: usize = 48;
    pub const FIELD_SIZES: [usize; 3] = [32, 8, 8];
    pub const FIELD_COUNT: usize = 3;
    pub fn merkle_root(&self) -> Byte32Reader<'r> {
        Byte32Reader::new_unchecked(&self.as_slice()[0..32])
    }
    pub fn start_height(&self) -> Uint64Reader<'r> {
        Uint64Reader::new_unchecked(&self.as_slice()[32..40])
    }
    pub fn latest_height(&self) -> Uint64Reader<'r> {
        Uint64Reader::new_unchecked(&self.as_slice()[40..48])
    }
}
impl<'r> molecule::prelude::Reader<'r> for ETHHeaderCellDataReader<'r> {
    type Entity = ETHHeaderCellData;
    const NAME: &'static str = "ETHHeaderCellDataReader";
    fn to_entity(&self) -> Self::Entity {
        Self::Entity::new_unchecked(self.as_slice().to_owned().into())
    }
    fn new_unchecked(slice: &'r [u8]) -> Self {
        ETHHeaderCellDataReader(slice)
    }
    fn as_slice(&self) -> &'r [u8] {
        self.0
    }
    fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
        use molecule::verification_error as ve;
        let slice_len = slice.len();
        if slice_len != Self::TOTAL_SIZE {
            return ve!(Self, TotalSizeNotMatch, Self::TOTAL_SIZE, slice_len);
        }
        Ok(())
    }
}
#[derive(Debug, Default)]
pub struct ETHHeaderCellDataBuilder {
    pub(crate) merkle_root: Byte32,
    pub(crate) start_height: Uint64,
    pub(crate) latest_height: Uint64,
}
impl ETHHeaderCellDataBuilder {
    pub const TOTAL_SIZE: usize = 48;
    pub const FIELD_SIZES: [usize; 3] = [32, 8, 8];
    pub const FIELD_COUNT: usize = 3;
    pub fn merkle_root(mut self, v: Byte32) -> Self {
        self.merkle_root = v;
        self
    }
    pub fn start_height(mut self, v: Uint64) -> Self {
        self.start_height = v;
        self
    }
    pub fn latest_height(mut self, v: Uint64) -> Self {
        self.latest_height = v;
        self
    }
}
impl molecule::prelude::Builder for ETHHeaderCellDataBuilder {
    type Entity = ETHHeaderCellData;
    const NAME: &'static str = "ETHHeaderCellDataBuilder";
    fn expected_length(&self) -> usize {
        Self::TOTAL_SIZE
    }
    fn write<W: ::molecule::io::Write>(&self, writer: &mut W) -> ::molecule::io::Result<()> {
        writer.write_all(self.merkle_root.as_slice())?;
        writer.write_all(self.start_height.as_slice())?;
        writer.write_all(self.latest_height.as_slice())?;
        Ok(())
    }
    fn build(&self) -> Self::Entity {
        let mut inner = Vec::with_capacity(self.expected_length());
        self.write(&mut inner)
            .unwrap_or_else(|_| panic!("{} build should be ok", Self::NAME));
        ETHHeaderCellData::new_unchecked(inner.into())
    }
}
