use codec::{Decode, Encode};
use gstd::{String, Vec, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
use crate::{Royalties};


#[derive(Debug, Decode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub price: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct MintInput {
    pub token_id: U256,
    pub media: String,
    pub reference: String,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferInput {
    pub to: H256,
    pub token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ApproveInput {
    pub to: H256,
    pub token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ApproveForAllInput {
    pub to: H256,
    pub approve: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ToMarket {
    pub market_id: ActorId,
    pub tokens: Vec<U256>,
    pub price: u128,
    pub on_sale: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct NFTInfo {
    pub royalties: Option<Royalties>,
    pub minted_amount: U256,
    pub supply: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct NFTPayout {
    pub owner: ActorId,
    pub amount: u128,
}