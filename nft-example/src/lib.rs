#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;
use nft_example_io::*;
use nft_example_io::{Action, Event};


use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::NonFungibleToken;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
// const ROYALTY_MULTIPLIER: u64 = 5; // fixed royalty %? 

#[derive(Debug, Decode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Debug)]
pub struct NFT {
    pub token: NonFungibleToken,
    pub token_id: U256,
    pub owner: ActorId,
    pub origin_by_id: BTreeMap<U256, ActorId>,
    pub royalty_rate: BTreeMap<U256, u64>, //token id:rate 
}

static mut CONTRACT: NFT = NFT {
    token: NonFungibleToken::new(),
    token_id: U256::zero(),
    owner: ZERO_ID,
    origin_by_id: BTreeMap::new(),
    royalty_rate: BTreeMap::new(),
};

impl NFT {
    fn mint(&mut self) {
        self.token.owner_by_id.insert(self.token_id, msg::source());
        let balance = *self
            .token
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());

        self.token
            .balances
            .insert(msg::source(), balance.saturating_add(U256::one()));

        msg::reply(
            Event::Transfer {
                from: ZERO_ID,
                to: msg::source(),
                token_id: self.token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
        self.token_id = self.token_id.saturating_add(U256::one());
        self.origin_by_id.insert(self.token_id, msg::source());
    }

    fn royalty(&mut self, token_id: U256, price: u64) {
        if !self.token.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        let multiplier = *self.royalty_rate.get(&token_id).unwrap_or(&5);
        msg::reply(
            Event::Royalty {
                amount: price*multiplier/100,
                origin: *self.origin_by_id.get(&token_id).unwrap_or(&ZERO_ID),
            },
            0,
            0,
        );

        // if self.royalty_rate.contains_key(&token_id) {
        //     let multiplier = *self.royalty_rate.get(&token_id).unwrap_or(&0);
        //     msg::reply(
        //         Event::Royalty {
        //             amount: price*multiplier/100,
        //             origin: *self.origin_by_id.get(&token_id).unwrap_or(&ZERO_ID),
        //         },
        //         0,
        //         0,
        //     );
        // } else {
        //     msg::reply(
        //         Event::Royalty {
        //             amount: price*ROYALTY_MULTIPLIER/100,
        //             origin: *self.origin_by_id.get(&token_id).unwrap_or(&ZERO_ID),
        //         },
        //         0,
        //         0,
        //     );
    }

    fn assignroyalty(&mut self, token_id: U256, rate: u64) {
        if !self.token.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        msg::reply(
            Event::AssignRoyalty {
                token_id,
                recipient: *self.origin_by_id.get(&token_id).unwrap_or(&ZERO_ID),
            },
            0,
            0,
        );
        self.royalty_rate.insert(self.token_id, rate);
    }

    fn burn(&mut self, token_id: U256) {
        if !self.token.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.token.is_token_owner(token_id, &msg::source()) {
            panic!("NonFungibleToken: account is not owner");
        }
        self.token.token_approvals.remove(&token_id);
        self.token.owner_by_id.remove(&token_id);
        let balance = *self
            .token
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.token
            .balances
            .insert(msg::source(), balance.saturating_sub(U256::one()));
        msg::reply(
            Event::Transfer {
                from: msg::source(),
                to: ZERO_ID,
                token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

gstd::metadata! {
    title: "Royalty NFT Example",
        init:
            input: InitConfig,
        handle:
            input: Action,
            output: Event,
        state:
            input: State,
            output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Mint => {
            CONTRACT.mint();
        }
        Action::Royalty { token_id, price } => {
            CONTRACT.royalty(token_id, price); //update the state of the contract by updating the royalty amount
        }
        Action::Burn(amount) => {
            CONTRACT.burn(amount);
        }
        Action::Transfer { to, token_id } => {
            CONTRACT.token.transfer(&msg::source(), &to, token_id);

        }
        Action::Approve { to, token_id } => {
            CONTRACT.token.approve(&msg::source(), &to, token_id);
        }
        Action::ApproveForAll { to, approved } => {
            CONTRACT
                .token
                .approve_for_all(&msg::source(), &to, approved);
        }
        Action::OwnerOf(input) => {
            CONTRACT.token.owner_of(input);
        }
        Action::BalanceOf(input) => {
            CONTRACT.token.balance_of(&input);
        }
        Action::AssignRoyalty { token_id, rate } => {
            CONTRACT.assignroyalty(token_id, rate); //assigns a royalty rate to a token
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NFT {:?}", config);
    CONTRACT
        .token
        .init(config.name, config.symbol, config.base_uri); 
    CONTRACT.owner = msg::source();
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let encoded = match query {
        State::BalanceOfUser(input) => {
            StateReply::BalanceOfUser(*CONTRACT.token.balances.get(&input).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.token.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(*user).encode()
        }
        State::IsTokenOwner { account, token_id } => {
            let user = CONTRACT
                .token
                .owner_by_id
                .get(&token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &account).encode()
        }
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .token
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(*approved_address).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}
