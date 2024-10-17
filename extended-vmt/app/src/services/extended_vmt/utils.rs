use sails_rs::prelude::*;

pub type Result<T, E = Error> = core::result::Result<T, E>;
pub type TokenId = U256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    ZeroAddress,
    LengthMismatch,
    IdIsNotUnique,
    MintMetadataToFungibleToken,
    TokenAlreadyExists,
    AmountGreaterThanOneForNft,
    WrongId,
    NotEnoughBalance,
}

#[derive(Debug, Decode, Encode, TypeInfo, Default, Clone, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub reference: Option<String>,
}
