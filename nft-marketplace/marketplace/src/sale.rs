use crate::{ft_transfer, nft_transfer, Event, ItemSoldOutput, Market, GAS_RESERVE};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};

impl Market {
    pub async fn buy_item(&mut self, nft_contract_id: &ActorId, token_id: U256) {
        if !self.item_exists(nft_contract_id, token_id) {
            panic!("That item does not exist");
        }
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items.get_mut(&contract_and_token_id).unwrap();
        if !item.on_sale {
            panic!("The item is not on sale");
        }
        let buyer_id = msg::source();
        let balance: &mut u128 = if let Some(balance) = self.balances.get_mut(&buyer_id) {
            if balance < &mut item.price {
                panic!("That buyer balance is too low");
            } else {
                balance
            }
        } else {
            panic!("The buyer has no funds")
        };
        // TODO: Auction
        if item.auction.is_some() {}

        // transfer NFT to buyer
        nft_transfer(nft_contract_id, &buyer_id, token_id).await;

        // fee for treasury
        let treasury_fee = item.price * self.treasury_fee / 10_000u128;

        // transfer payment to owner
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &item.owner_id,
            item.price - treasury_fee,
        )
        .await;

        // transfer treasury fee to treasury id
        ft_transfer(
            &self.approved_ft_token,
            &exec::program_id(),
            &self.treasury_id,
            treasury_fee,
        )
        .await;

        //TODO: royalties
        item.owner_id = buyer_id;
        *balance -= item.price;

        msg::reply(
            Event::ItemSold(ItemSoldOutput {
                owner: H256::from_slice(buyer_id.as_ref()),
                nft_contract_id: H256::from_slice(nft_contract_id.as_ref()),
                token_id,
            }),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}
