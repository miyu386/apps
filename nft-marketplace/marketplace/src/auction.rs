use codec::{Decode, Encode};
use gstd::{exec, prelude::*, ActorId};
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
    pub fn is_auction_on_going(
        &self,
    ) -> bool {
        return exec::block_timestamp() < self.ended_at;
    }

    pub fn get_bids(
        &self,
    ) -> &Vec<Bid> {
        return self.bids.as_ref().unwrap();
    }

    pub fn bids_len(
        &self,
    ) -> usize {
        if self.bids.is_some() {
            return self.bids.as_ref().unwrap().len();
        } else {
            return 0;
        }
    }
}
