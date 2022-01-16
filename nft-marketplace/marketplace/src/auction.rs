use crate::{
    ft_transfer, nft_transfer, ContractToken, ContractTokenPrice, Event, Market, GAS_RESERVE,
};
use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Bid {
    pub id: ActorId,
    pub price: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Auction {
    pub bid_period: u64,
    pub started_at: u64,
    pub ended_at: u64,
    pub current_price: u128,
    pub bids: Option<Vec<Bid>>,
}

impl Auction {
    pub fn is_auction_on_going(&self) -> bool {
        exec::block_timestamp() < self.ended_at
    }

    pub fn get_bids(&self) -> &Vec<Bid> {
        return self.bids.as_ref().unwrap();
    }

    pub fn bids_len(&self) -> usize {
        if self.bids.is_some() {
            return self.bids.as_ref().unwrap().len();
        } else {
           0
        }
    }
}

impl Market {
    // `bid_period` - the time that the auction lasts until another bid occurs
    pub fn create_auction(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        min_price: u128,
        bid_period: u64,
    ) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
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
            Event::AuctionCreated(ContractTokenPrice {
                nft_contract_id: H256::from_slice(nft_contract_id.as_ref()),
                token_id,
                price: min_price,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    pub async fn settle_auction(&mut self, nft_contract_id: &ActorId, token_id: U256) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
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
            let highest_bid = &auction.get_bids()[auction.bids_len() - 1];
            // transfer payment to owner
            ft_transfer(
                &self.approved_ft_token,
                &exec::program_id(),
                &item.owner_id,
                item.price,
            )
            .await;
            // transfer NFT
            nft_transfer(nft_contract_id, &highest_bid.id, token_id).await;
            msg::reply(
                Event::AuctionSettled(ContractTokenPrice {
                    nft_contract_id: H256::from_slice(nft_contract_id.as_ref()),
                    token_id,
                    price: item.price,
                }),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        } else {
            msg::reply(
                Event::AuctionCancelled(ContractToken {
                    nft_contract_id: H256::from_slice(nft_contract_id.as_ref()),
                    token_id,
                }),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        item.auction = None;
    }

    pub fn add_bid(&mut self, nft_contract_id: &ActorId, token_id: U256, price: u128) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
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
        if auction.bids.is_none() {
            let current_bid = &bids[bids.len() - 1];
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
            Event::BidAdded(ContractTokenPrice {
                nft_contract_id: H256::from_slice(nft_contract_id.as_ref()),
                token_id,
                price,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}
