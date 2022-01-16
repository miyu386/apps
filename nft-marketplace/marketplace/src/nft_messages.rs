use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
const GAS_RESERVE: u64 = 500_000_000;

pub async fn nft_transfer(nft_program_id: &ActorId, to: &ActorId, token_id: U256) {
    let transfer_input = TransferInput {
        to: H256::from_slice(to.as_ref()),
        token_id,
    };
    let transfer_response: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::Transfer(transfer_input),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .unwrap();

    if let NFTEvent::Transfer(transfer_response) = transfer_response {
        if transfer_response.token_id != token_id {
            panic!("error in transfer");
        }
    }
}

pub async fn nft_owner_of(nft_program_id: &ActorId, token_id: U256) -> ActorId {
    let owner_of: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::OwnerOf(token_id),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .unwrap();

    if let NFTEvent::OwnerOf(owner_of) = owner_of {
        ActorId::new(owner_of.to_fixed_bytes())
    } else {
        panic!("Error in function 'nft_owner_of' call");
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum NFTEvent {
    Transfer(TransferInput),
    Approval,
    ApprovalForAll,
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum NFTAction {
    Mint,
    Burn(U256),
    Transfer(TransferInput),
    Approve,
    ApproveForAll,
    OwnerOf(U256),
    BalanceOf(H256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferInput {
    pub to: H256,
    pub token_id: U256,
}
