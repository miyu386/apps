use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
const GAS_RESERVE: u64 = 500_000_000;

pub async fn list_nfts_on_market(market_id: &ActorId, owner: &ActorId, tokens: Vec<U256>, price: u128, on_sale: bool) {
    let market_input = NFTContractCall {
        owner: *owner,
        tokens,
        price,
        on_sale,
    };
    let _market_response: MarketEvent = msg::send_and_wait_for_reply(
        *market_id,
        MarketAction::NFTContractCall(market_input),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("error in sending message to marketplace");
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MarketAction {
    AddNftContract,
    BuyItem,
    Deposit,
    Withdraw,
    AddBid,
    CreateAuction,
    SettleAuction,
    ListMyNFTs,
    NFTContractCall(NFTContractCall),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MarketEvent {
    ItemSold,
    BidAdded,
    AuctionCreated,
    AuctionSettled,
    AuctionCancelled,
    NFTsListed(NFTsListed),
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