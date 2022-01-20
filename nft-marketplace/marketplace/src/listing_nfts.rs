use crate::{nft_messages::tokens_for_owner, Event, Market, GAS_RESERVE, NFTsListed};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};

impl Market {
    pub async fn list_my_nfts(&mut self, nft_contract_id: &ActorId, price: u128, on_sale: bool) {
        let tokens = tokens_for_owner(nft_contract_id, &msg::source()).await;
        for token in tokens.iter() {
            self.create_item(nft_contract_id, *token, price, on_sale).await;
        }
        msg::reply(
            Event::NFTsListed(NFTsListed {
                nft_contract_id: *nft_contract_id,
                owner: msg::source(),
                tokens,
                price,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
    pub async fn call_from_nft_contract(&mut self, owner: &ActorId, tokens: Vec<U256>, price: u128, on_sale: bool) {
        if !self.approved_nft_contracts.contains(&msg::source()) {
            panic!("that nft contract is not approved");
        }
        for token in tokens.iter() {
            self.create_item(&msg::source(), *token, price, on_sale).await;
        }
        msg::reply(
            Event::NFTsListed(NFTsListed {
                nft_contract_id: msg::source(),
                owner: *owner,
                tokens,
                price,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );

    }
}