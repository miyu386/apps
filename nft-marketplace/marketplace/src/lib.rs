#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId, collections::btree_map::Entry};
//use gstd::collections::btree_map::Entry;
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

pub mod nft_messages;
use nft_messages::{nft_transfer, nft_owner_of};

pub mod ft_messages;
use ft_messages::{ft_transfer};

pub mod auction;
use auction::{Auction, Bid};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());
pub type ContractAndTokenId = String;

#[derive(Debug, Decode, TypeInfo)]
pub struct InitConfig {
    pub owner_id: H256,
    pub treasury_id: H256,
    pub treasury_fee: u128,
    pub approved_ft_token: H256,
}

#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Item {
    pub owner_id: ActorId,
    pub nft_contract_id: ActorId,
    pub token_id: U256,
    pub price: u128,
    pub auction: Option<Auction>,
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
}

static mut CONTRACT: Market = Market {
    owner_id: ZERO_ID,
    treasury_id: ZERO_ID,
    treasury_fee: 0,
    items: BTreeMap::new(),
    balances: BTreeMap::new(),
    approved_ft_token: ZERO_ID,
    approved_nft_contracts: Vec::new(),
};

impl Market {

    fn add_nft_contract(
        &mut self,
        nft_contract_id: &ActorId,
    ) {
        self.approved_nft_contracts.push(*nft_contract_id);
    }

    async fn create_item(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        price: u128,
    ) {
        if self.item_exists(nft_contract_id, token_id) {
            panic!("That item already exists");
        }
        let owner_id = nft_owner_of(nft_contract_id, token_id).await;
        nft_transfer(
            nft_contract_id,
            &owner_id,
            token_id,
        ).await;
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let new_item = Item {
            owner_id,
            nft_contract_id: *nft_contract_id,
            token_id,
            price,
            auction: None,
            on_sale: true,
        };
        self.items.insert(contract_and_token_id, new_item);
    }

    async fn buy_item(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
    ) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist"); 
        }
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        let buyer_id = msg::source();
        let balance: &mut u128 = 
            if let Some(balance) = self.balances.get_mut(&buyer_id) {
                if balance < &mut item.price {
                    panic!("That buyer balance is too low");
                } else {
                    balance
                }
            } else {
                panic!("The buyer has no funds")
            };
        // TODO: Auction
        if item.auction.is_some() {

        }

        // transfer NFT to buyer
        nft_transfer(
            nft_contract_id,
            &buyer_id,
            token_id,
        )
        .await;
        
        // fee for treasury
        let treasury_fee = item.price * self.treasury_fee / 10_000u128;

        // transfer payment to owner
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &item.owner_id,
            item.price - treasury_fee,
        ).await;
        
        // transfer treasury fee to treasury id
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &self.treasury_id,
            treasury_fee,
        ).await;

        //TODO: royalties
        item.owner_id = buyer_id;
        *balance = *balance - &item.price;

        msg::reply(
            Event::ItemSold(
                ItemSoldOutput {
                    owner: H256::from_slice(buyer_id.as_ref()),
                    nft_contract_id:  H256::from_slice(nft_contract_id.as_ref()),
                    token_id,
                }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    async fn deposit(
        &mut self,
        amount: u128,
    ) {
        ft_transfer(
            &self.approved_ft_token,
            &msg::source(),
            &exec::program_id(),
            amount,
        ).await;
        self.balances.entry(msg::source())
            .and_modify(|balance| { *balance += amount })
            .or_insert(amount);
    }

    async fn withdraw(
        &mut self,
        amount: u128,
    ) {
        match self.balances.entry(msg::source()) {
            Entry::Occupied(mut o) => {
                if o.get() < &amount {
                    panic!("not enough balanace to withdraw")
                }
                *o.get_mut() -= amount;
            },
            Entry::Vacant(_v) => {
                panic!("account has no balance");
            },
        };
       
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &msg::source(),
            amount,
        ).await;
        
    }

    // `bid_period` - the time that the auction lasts until another bid occurs
    fn create_auction(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        min_price: u128,
        bid_period: u64,
    ) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist"); 
        }
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.auction.is_some() {
            panic!("auction already exists");
        }
        if item.owner_id != msg::source() {
            panic!("not nft owner");
        }
        item.auction = Some(Auction {
                        bid_period,
                        started_at: exec::block_timestamp(),
                        ended_at: exec::block_timestamp() + bid_period,
                        current_price: min_price,
                        bids: None,
                    });
        msg::reply(
            Event::AuctionCreated(
                ContractTokenPrice {
                    nft_contract_id:  H256::from_slice(nft_contract_id.as_ref()),
                    token_id,
                    price: min_price,
                }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn add_bid(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        price: u128,
    ) {        
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist"); 
        }
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);     
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.auction.is_some() {
            if !item.auction.as_ref().unwrap().is_auction_on_going() {
                panic!("Auction has already ended");
            }
        } else {
            panic!("Auction does not exist");
        }
        if let Some(balance) = self.balances.get(&msg::source()) {
            if balance < &price {
                panic!("That buyer balance is too low");
            } 
        } else {
            panic!("The buyer has no funds")
        };
        let mut auction = item.auction.as_ref().unwrap().clone();
        let mut bids = auction.bids.as_ref().unwrap().clone();
        if !auction.bids.is_some() {
            let current_bid = &bids[bids.len()-1];
            if price <= current_bid.price {
                panic!("Cant offer less than current bid price")
            }
        } 
        bids.push(Bid {
            id: msg::source(),
            price,
        });
        
        auction.ended_at = exec::block_timestamp() + auction.bid_period; 
        auction.bids = Some(bids);
        auction.current_price = price;
        item.auction = Some(auction);
        msg::reply(
            Event::BidAdded(
                ContractTokenPrice {
                    nft_contract_id:  H256::from_slice(nft_contract_id.as_ref()),
                    token_id,
                    price,
                }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    async fn settle_auction (
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
    ) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist"); 
        }
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);     
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.auction.is_some() {
            if item.auction.as_ref().unwrap().is_auction_on_going() {
                panic!("Auction is not over");
            }
        } else {
            panic!("Auction does not exist");
        }
        let auction = item.auction.as_ref().unwrap().clone();
        if auction.bids_len() > 0 {
            let highest_bid = &auction.get_bids()[auction.bids_len()-1];
            // transfer payment to owner
            ft_transfer(
                &self.approved_ft_token,
                &exec::program_id(),
                &item.owner_id,
                item.price,
            ).await;
            // transfer NFT 
            nft_transfer(
                nft_contract_id,
                &highest_bid.id,
                token_id,
            )
            .await;
            msg::reply(
                Event::AuctionSettled(
                    ContractTokenPrice {
                        nft_contract_id:  H256::from_slice(nft_contract_id.as_ref()),
                        token_id,
                        price: item.price,
                    }),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        } else {
            msg::reply(
                Event::AuctionCancelled(
                    ContractToken {
                        nft_contract_id:  H256::from_slice(nft_contract_id.as_ref()),
                        token_id,
                    }),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        item.auction = None;

    }   

    fn item_exists(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
    ) -> bool {
        let contract_and_token_id = format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        match self.items.entry(contract_and_token_id) {
            Entry::Occupied(_o) => return true,
            Entry::Vacant(_v) => return false,
        };
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

        }
        Action::CreateItem(input) => {
            CONTRACT.create_item(
                &ActorId::new(input.nft_contract_id.to_fixed_bytes()),
                input.token_id,
                input.price,
            ).await;
        }
        Action::BuyItem(input) => {
            CONTRACT.buy_item(
                &ActorId::new(input.nft_contract_id.to_fixed_bytes()),
                input.token_id,
            ).await;
        }
        Action::Deposit(input) => {
            CONTRACT.deposit(input).await;
        }
        Action::Withdraw(input) => {
            CONTRACT.withdraw(input).await;
        }
        Action::AddBid(input) => {
            CONTRACT.add_bid(
                &ActorId::new(input.nft_contract_id.to_fixed_bytes()),
                input.token_id,
                input.price,
            )
        }
        Action::CreateAuction(input) => {
            CONTRACT.create_auction(
                &ActorId::new(input.nft_contract_id.to_fixed_bytes()),
                input.token_id,
                input.price,
                input.bid_period,
            )
        }
        Action::SettleAuction(input) => {
            CONTRACT.settle_auction(
                &ActorId::new(input.nft_contract_id.to_fixed_bytes()),
                input.token_id,
            ).await;
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
    AddNftContract(H256),
    CreateItem(ContractTokenPrice),
    BuyItem(ContractToken),
    Deposit(u128),
    Withdraw(u128),
    AddBid(ContractTokenPrice),
    CreateAuction(CreateAuctionInput),
    SettleAuction(ContractToken),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    ItemSold(ItemSoldOutput),
    BidAdded(ContractTokenPrice),
    AuctionCreated(ContractTokenPrice),
    AuctionSettled(ContractTokenPrice),
    AuctionCancelled(ContractToken)
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
    nft_contract_id: H256,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ItemSoldOutput {
    owner: H256,
    nft_contract_id: H256,
    token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct ContractTokenPrice {
    nft_contract_id: H256,
    token_id: U256,
    price: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct CreateAuctionInput {
    nft_contract_id: H256,
    token_id: U256,
    price: u128,
    bid_period: u64,
}