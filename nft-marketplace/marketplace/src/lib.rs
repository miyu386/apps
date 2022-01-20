#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{collections::btree_map::Entry, debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
use nft::royalties::{Royalties, Payout};
pub mod nft_messages;
use nft_messages::{nft_owner_of, nft_transfer};

pub mod ft_messages;
use ft_messages::ft_transfer;

pub mod auction;
use auction::Auction;

pub mod offers;
use offers::Offer;

pub mod sale;
pub mod listing_nfts;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());
const OFFER_HISTORY_LENGTH_DEFAULT: u8 = 10;
pub type ContractAndTokenId = String;

#[derive(Debug, Decode, TypeInfo)]
pub struct InitConfig {
    pub owner_id: H256,
    pub treasury_id: H256,
    pub treasury_fee: u128,
    pub approved_ft_token: H256,
    pub offer_history_length: Option<u8>,
}

#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Item {
    pub owner_id: ActorId,
    pub nft_contract_id: ActorId,
    pub token_id: U256,
    pub price: u128,
    pub auction: Option<Auction>,
    pub offers: Option<Vec<Offer>>,
    pub on_sale: bool,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct Market {
    pub owner_id: ActorId,
    pub treasury_id: ActorId,
    pub treasury_fee: u128,
    pub items: BTreeMap<ContractAndTokenId, Item>,
    pub balances: BTreeMap<ActorId, u128>,
    pub approved_ft_token: ActorId,
    pub approved_nft_contracts: Vec<ActorId>,
    pub offer_history_length: u8,
}

static mut CONTRACT: Market = Market {
    owner_id: ZERO_ID,
    treasury_id: ZERO_ID,
    treasury_fee: 0,
    items: BTreeMap::new(),
    balances: BTreeMap::new(),
    approved_ft_token: ZERO_ID,
    approved_nft_contracts: Vec::new(),
    offer_history_length: 0,
};

impl Market {
    async fn add_nft_contract(&mut self, nft_contract_id: &ActorId) {
        self.approved_nft_contracts.push(*nft_contract_id);
        // let info = nft_info(nft_contract_id).await;
        // self.approved_nft_contracts.insert(*nft_contract_id, info.as_ref().unwrap().royalties.clone());
        // let minted_nft = info.unwrap().minted_amount;
        // for nft in minted_nft.iter() {
        //     self.create_item(nft_contract_id, i, 0).await;
        // }
    }

    // Creates new item.
    // Requirements:
    // * The proposal can be submitted only by the existing members or their delegate addresses
    // Arguments:
    // * `nft_contract_id`: an actor, who wishes to become a DAO member
    // * `token_id`: the number of tokens the applicant offered for shares in DAO
    // * `price`: the amount of shares the applicant is requesting for his token tribute
    async fn create_item(&mut self, nft_contract_id: &ActorId, token_id: U256, price: u128, on_sale: bool) {
        if self.item_exists(nft_contract_id, token_id) {
            panic!("That item already exists");
        }
        let owner_id = nft_owner_of(nft_contract_id, token_id).await;
       // nft_transfer(nft_contract_id, &exec::program_id(), token_id).await;
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let new_item = Item {
            owner_id,
            nft_contract_id: *nft_contract_id,
            token_id,
            price,
            auction: None,
            on_sale,
            offers: None,
        };
        self.items.insert(contract_and_token_id, new_item);
    }

    async fn deposit(&mut self, amount: u128) {
        ft_transfer(
            &self.approved_ft_token,
            &msg::source(),
            &exec::program_id(),
            amount,
        )
        .await;
        self.balances
            .entry(msg::source())
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
    }

    async fn withdraw(&mut self, amount: u128) {
        self.reduce_balance(&msg::source(), amount);
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &msg::source(),
            amount,
        )
        .await;
    }

    fn reduce_balance(
        &mut self, 
        account: &ActorId,
        amount: u128,
    ) {
        match self.balances.entry(*account) {
            Entry::Occupied(mut o) => {
                if o.get() < &amount {
                    panic!("not enough balanace to withdraw")
                }
                *o.get_mut() -= amount;
            }
            Entry::Vacant(_v) => {
                panic!("account has no balance");
            }
        };
    }
    fn item_exists(&mut self, nft_contract_id: &ActorId, token_id: U256) -> bool {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        match self.items.entry(contract_and_token_id) {
            Entry::Occupied(_o) => true,
            Entry::Vacant(_v) => false,
        }
    }
}

gstd::metadata! {
    title: "NFTMarketplace",
        init:
            input: InitConfig,
        handle:
            input: Action,
            output: Event,
        state:
            input: State,
            output: StateReply,
}

#[gstd::async_main]
async fn main() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::AddNftContract(input) => {
            CONTRACT.add_nft_contract(&input).await;
        }
        Action::ListMyNFTs(input) => {
            CONTRACT.list_my_nfts(
                &input.nft_contract_id,
                input.price,
                input.on_sale,
            )
            .await;
        }
        Action::NFTContractCall(input) => {
            CONTRACT
                .call_from_nft_contract(
                    &input.owner,
                    input.tokens,
                    input.price,
                    input.on_sale,
                )
                .await;
        }
        Action::BuyItem(input) => {
            CONTRACT
                .buy_item(
                    &input.nft_contract_id,
                    input.token_id,
                )
                .await;
        }
        Action::Deposit(input) => {
            CONTRACT.deposit(input).await;
        }
        Action::Withdraw(input) => {
            CONTRACT.withdraw(input).await;
        }
        Action::AddBid(input) => CONTRACT.add_bid(
            &input.nft_contract_id,
            input.token_id,
            input.price,
        ),
        Action::CreateAuction(input) => CONTRACT.create_auction(
            &input.nft_contract_id,
            input.token_id,
            input.price,
            input.bid_period,
        ),
        Action::SettleAuction(input) => {
            CONTRACT
                .settle_auction(
                    &input.nft_contract_id,
                    input.token_id,
                )
                .await;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    CONTRACT.owner_id = ActorId::new(config.owner_id.to_fixed_bytes());
    CONTRACT.treasury_id = ActorId::new(config.treasury_id.to_fixed_bytes());
    CONTRACT.treasury_fee = config.treasury_fee;
    CONTRACT.approved_ft_token = ActorId::new(config.approved_ft_token.to_fixed_bytes());
    CONTRACT.offer_history_length = config
        .offer_history_length
        .unwrap_or(OFFER_HISTORY_LENGTH_DEFAULT);
}

// #[no_mangle]
// pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
//     let query: State = msg::load().expect("failed to decode input argument");
//     let encoded = match query {
//         State::ItemInfo => {

//         }
//     };
//     let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

//     core::mem::forget(encoded);

//     result
// }

#[derive(Debug, Decode, TypeInfo)]
pub enum Action {
    AddNftContract(ActorId),
    BuyItem(ContractToken),
    Deposit(u128),
    Withdraw(u128),
    AddBid(ContractTokenPrice),
    CreateAuction(CreateAuctionInput),
    SettleAuction(ContractToken),
    ListMyNFTs(ListMyNFTs),
    NFTContractCall(NFTContractCall),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    ItemSold(ItemSoldOutput),
    BidAdded(ContractTokenPrice),
    AuctionCreated(ContractTokenPrice),
    AuctionSettled(ContractTokenPrice),
    AuctionCancelled(ContractToken),
    NFTsListed(NFTsListed),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    ItemInfo,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum StateReply {
    ItemInfo,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ContractToken {
    nft_contract_id: ActorId,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ItemSoldOutput {
    owner: ActorId,
    nft_contract_id: ActorId,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ContractTokenPrice {
    nft_contract_id: ActorId,
    token_id: U256,
    price: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct CreateAuctionInput {
    nft_contract_id: ActorId,
    token_id: U256,
    price: u128,
    bid_period: u64,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ListMyNFTs {
    nft_contract_id: ActorId,
    price: u128,
    on_sale: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct NFTContractCall {
    owner: ActorId,
    tokens: Vec<U256>,
    price: u128,
    on_sale: bool,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct NFTsListed {
    nft_contract_id: ActorId,
    owner: ActorId,
    tokens: Vec<U256>,
    price: u128,
}