use crate::{ft_transfer, nft_transfer, Market};
use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Offer {
    pub id: ActorId,
    pub price: u128,
    pub expires_at: u64,
}

impl Market {
    fn add_offer(&mut self, nft_contract_id: &ActorId, token_id: U256, price: u128, duration: u64) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.auction.is_some() && item.auction.as_ref().unwrap().is_auction_on_going()  {
                panic!("Auction must have ended");
            }
        let new_offer = Offer {
            id: msg::source(),
            price,
            expires_at: exec::block_timestamp() + duration,
        };

        let mut offers = item.offers.as_ref().unwrap_or(&Vec::new()).clone();
        offers.push(new_offer);
        offers.sort_by(|a, b| b.price.cmp(&a.price));
        if offers.len() > self.offer_history_length as usize {
            offers.remove(0);
        }
        item.offers = Some(offers);
        // msg reply
    }

    async fn accept_offer(&mut self, nft_contract_id: &ActorId, token_id: U256) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.owner_id != msg::source() {
            panic!("only NFT owner can accept offer");
        }
        if item.auction.is_some() && item.auction.as_ref().unwrap().is_auction_on_going()  {
            panic!("Auction must have ended");
        }
        let offers = item.offers.as_ref().unwrap_or(&Vec::new()).clone();
        let not_expired_offers: Vec<Offer> = offers
            .into_iter()
            .filter(|a| a.expires_at < exec::block_timestamp())
            .collect();
        let last_offer = &not_expired_offers[not_expired_offers.len() - 1];
        // transfer payment to owner
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &item.owner_id,
            last_offer.price,
        )
        .await;
        // transfer NFT
        nft_transfer(nft_contract_id, &last_offer.id, token_id).await;
        item.owner_id = last_offer.id;
        item.on_sale = false;
    }
}
