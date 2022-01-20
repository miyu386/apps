use crate::{ft_transfer, nft_transfer, nft_messages::nft_payouts, Event, ItemSoldOutput, Market, GAS_RESERVE};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};

impl Market {
    // Called when a user wants to buy NFT. 
    // Requirements:
    // * The NFT must exists and be on sale
    // * The buyer must have enough balance
    // * There must be no opened auctions
    // Arguments:
    // * `nft_contract_id`: NFT contract address
    // * `token_id`: the token ID
    pub async fn buy_item(&mut self, nft_contract_id: &ActorId, token_id: U256) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if item.auction.is_some() {
            panic!("There is an opened auction");
        }
        if !item.on_sale {
            panic!("The item is not on sale");
        }
        let buyer_id = msg::source();
        // transfer NFT to buyer
        nft_transfer(nft_contract_id, &buyer_id, token_id).await;

        // fee for treasury
        let treasury_fee = item.price * self.treasury_fee / 10_000u128;
        
        // transfer treasury fee to treasury id
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &self.treasury_id,
            treasury_fee,
        )
        .await;

        let payouts = nft_payouts(nft_contract_id, &item.owner_id, item.price - treasury_fee).await;
      
        for (account, amount) in payouts.iter() {
            ft_transfer(
                &self.approved_ft_token,
                &buyer_id,
                account,
                *amount,
            ).await
        }
    
        // transfer NFT to buyer
        nft_transfer(nft_contract_id, &buyer_id, token_id).await;

        item.owner_id = buyer_id;
        item.on_sale = false;

        msg::reply(
            Event::ItemSold(ItemSoldOutput {
                owner: buyer_id,
                nft_contract_id: *nft_contract_id,
                token_id,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}
