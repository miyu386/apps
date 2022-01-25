#![no_std]
#![feature(const_btree_new)]

// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.2/contracts/token/ERC1155/IERC1155.sol

use codec::{Decode, Encode};
use gstd::prelude::*;
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

// use token::Erc1155TokenBase;
// use Erc1155TokenBase;
pub mod base;

const GAS_RESERVE: u64 = 500_000_000;

#[derive(Debug)]
struct Erc1155Token {
    pub title: Option<String>,
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl base::Erc1155TokenBase for Erc1155Token {
    fn init(&mut self, name: String, symbol: String, base_uri: String) {}

    fn balance_of(&self, account: &ActorId, token_id: U256) {}
    fn balance_of_batch(&self, accounts: &[ActorId], token_ids: &[U256]) {}
    fn set_approval_for_all(&mut self, operator: &ActorId, approved: bool) {}
    fn is_approved_for_all(&self, account: &ActorId, operator: &ActorId) {}
    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, token_id: U256, value: U256) {}
    fn safe_batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        token_id: U256,
        values: &[U256],
    ) {
    }

    fn owner_of(&self, token_id: U256) {}
}

impl Erc1155Token {}
