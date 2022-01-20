use crate::{ZERO_ID, Royalties};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
pub type Payout = BTreeMap<ActorId, u128>;
const GAS_RESERVE: u64 = 500_000_000;

pub async fn nft_transfer(nft_program_id: &ActorId, to: &ActorId, token_id: U256) {
    let transfer_input = Transfer {
        to: *to,
        token_id,
    };
    let _transfer_response: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::Transfer(transfer_input),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("error in transfer");
}

pub async fn nft_owner_of(nft_program_id: &ActorId, token_id: U256) -> ActorId {
    let owner_of: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::OwnerOf(token_id),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_owner_of' call");
   match owner_of {
        NFTEvent::OwnerOf(owner) => owner,
        _ => ZERO_ID,
   }
}


pub async fn tokens_for_owner(nft_program_id: &ActorId, account: &ActorId) -> Vec<U256> {
    let tokens: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::TokensForOwner(*account),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_owner_of' call");
    match tokens {
        NFTEvent::TokensForOwner(tokens) => tokens,
        _ => vec![],
   }
}

pub async fn nft_payouts(nft_program_id: &ActorId, owner: &ActorId, amount: u128,) -> Payout {
    let payouts: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::NFTPayout(NFTPayout {
            owner: *owner,
            amount
        }),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_payout' call");
    match payouts {
        NFTEvent::NFTPayout(payouts) => payouts,
        _ => BTreeMap::new(),
   }
}
#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint,
    Burn(U256),
    Transfer(Transfer),
    Approve,
    ApproveForAll,
    OwnerOf(U256),
    BalanceOf(ActorId),
    SendToMarket,
    TokensForOwner(ActorId),
    NFTPayout(NFTPayout),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTEvent {
    Transfer(Transfer),
    Approval,
    ApprovalForAll,
    OwnerOf(ActorId),
    BalanceOf(U256),
    TokensForOwner(Vec<U256>),
    NFTPayout(Payout),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct Transfer {
    pub to: ActorId,
    pub token_id: U256,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct NFTPayout{
    pub owner: ActorId,
    pub amount: u128,
}