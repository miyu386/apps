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
    balances: BTreeMap<u128, BTreeMap<ActorId, u128>>,
    operator_approvals: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

static mut ERC1155_TOKEN: Erc1155Token = Erc1155Token {
    name: String::new(),
    symbol: String::new(),
    base_uri: String::new(),
    description: String::new(),
    balances: BTreeMap::new(),
    operator_approvals: BTreeMap::new(),
};

impl Erc1155Token {
    fn get_balance(&self, account: &ActorId, id: &u128) -> u128 {
        // TODO
        // unwrap panic
        *self.balances.get(id).unwrap().get(account).unwrap_or(&0)
    }

    fn set_balance(&mut self, account: &ActorId, id: &u128, amount: u128) {
        let mut _balance = self.balances.get(id).unwrap().clone();
        _balance.insert(*account, amount);
    }

    fn balance_of(&self, account: &ActorId, id: &u128) -> u128 {
        self.get_balance(account, id)
    }

    fn mint(&mut self, account: &ActorId, id: &u128, amount: u128) {
        // check owner
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address");
        }
        unsafe {
            let old_balance = ERC1155_TOKEN.get_balance(account, id);
            self.set_balance(account, id, old_balance.saturating_add(amount));
        }
        // TransferSingle event
    }

    fn mint_batch(&mut self, account: &ActorId, ids: &[u128], amounts: &[u128]) {
        if account == &ZERO_ID {
            panic!("ERC1155: Mint to zero address");
        }

        if ids.len() != amounts.len() {
            panic!("ERC1155: ids and amounts length mismatch");
        }

        for (i, ele) in ids.iter().enumerate() {
            let amount = amounts[i];
            unsafe {
                let old_balance = ERC1155_TOKEN.get_balance(account, ele);
                self.set_balance(account, ele, old_balance.saturating_add(amount));
            }
        }

        // TransferBatch event
    }

    fn set_approval_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool) {
        if owner != operator {
            panic!("ERC1155: setting approval status for self")
        }
        self.operator_approvals
            .entry(*owner)
            .or_default()
            .insert(*operator, approved);
        // ApprovalForAll event
    }

    fn is_approved_for_all(&mut self, account: &ActorId, operator: &ActorId) -> &bool {
        self.get_approval(account, operator)
    }

    fn get_approval(&mut self, owner: &ActorId, operator: &ActorId) -> &bool {
        if owner != operator {
            panic!("ERC1155: setting approval status for self")
        }

        self.operator_approvals
            .entry(*owner)
            .or_default()
            .get(operator)
            .unwrap_or(&false)
    }

    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &u128, amount: u128) {
        if from == to {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if !self.get_approval(from, to) {
            panic!("ERC1155: caller is not owner nor approved")
        }

        if to == &ZERO_ID {
            panic!("ERC1155: transfer to the zero address")
        }

        let from_balance = self.get_balance(from, id);

        if from_balance < amount {
            panic!("ERC1155: insufficient balance for transfer")
        }

        self.set_balance(from, id, from_balance.saturating_sub(amount));
        let to_balance = self.get_balance(to, id);
        self.set_balance(to, id, to_balance.saturating_add(amount));
        // ApprovalForAll event
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
