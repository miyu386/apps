#![no_std]

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;
use nft_example_io::{Action, Event};

#[derive(Encode, Debug, Decode, TypeInfo)]
pub enum Action {
    Mint,
    Burn(U256),
    Transfer { to: ActorId, token_id: U256 },
    Approve { to: ActorId, token_id: U256 },
    ApproveForAll { to: ActorId, approved: bool },
    OwnerOf(U256),
    BalanceOf(ActorId),
    Royalty { token_id: U256, price: u64 },
    AssignRoyalty {token_id: U256, rate: u64},
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: U256,
    },
    Approval {
        owner: ActorId,
        spender: ActorId,
        token_id: U256,
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
    OwnerOf(ActorId),
    BalanceOf(U256),
    Royalty {
        amount: u64,
        origin: ActorId,
    },
    AssignRoyalty {
        token_id: U256,
        recipient: ActorId,
    },
}
