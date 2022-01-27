#![no_std]
#![feature(const_btree_new)]

// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.4.2/contracts/token/ERC1155/IERC1155.sol

use codec::{Decode, Encode};
use gstd::prelude::*;
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

pub mod base;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
struct Erc1155Token {
    name: String,
    symbol: String,
    description: String,
    base_uri: String,
    balances: BTreeMap<u128, BTreeMap<ActorId, u128>>
}

static mut ERC1155_TOKEN: Erc1155Token = Erc1155Token {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    description: String::new(),
    balances: BTreeMap::new(),
};

impl Erc1155Token {
    fn get_balance(&self, account: &ActorId, id: &u128) -> u128 {
        // TODO
        // unwrap panic
        *self.balances.get(id).unwrap().get(account).unwrap_or(&0)
    }

    fn balance_of(&self, account: &ActorId, id: &u128) -> u128 {
        self.get_balance(account, id)
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Mint,
    Burn(U256),
    Transfer { to: ActorId, token_id: U256 },
    Approve { to: ActorId, token_id: U256 },
    ApproveForAll { to: ActorId, approved: bool },
    OwnerOf(U256),
    BalanceOf(ActorId),
}

pub enum Event<'a> {
    TransferSingle {
        operator: ActorId,
        from: ActorId,
        to: ActorId,
        token_id: U256,
        value: U256,
    },
    TransferBatch {
        operator: ActorId,
        from: ActorId,
        to: ActorId,
        token_ids: &'a [U256],
        values: &'a [U256],
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
    URI {
        value: String,
        token_id: U256,
    },
}
