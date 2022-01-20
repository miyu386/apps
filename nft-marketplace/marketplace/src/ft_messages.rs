use gstd::{exec, msg, prelude::*, ActorId};
const GAS_RESERVE: u64 = 500_000_000;

pub async fn ft_transfer(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferFromInput {
        owner: *from,
        to: *to,
        amount,
    };
    let _transfer_response: FTEvent = msg::send_and_wait_for_reply(
        *token_id,
        FTAction::TransferFrom(transfer_data),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in transfer message");
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferFromInput {
    pub owner: ActorId,
    pub to: ActorId,
    pub amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferFromReply {
    pub owner: ActorId,
    pub sender: ActorId,
    pub recipient: ActorId,
    pub amount: u128,
    pub new_limit: u128,
}


#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum FTAction {
    Mint,
    Burn,
    Transfer,
    TransferFrom(TransferFromInput),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum FTEvent {
    Transfer,
    Approval,
    AdminAdded,
    AdminRemoved,
    TransferFrom(TransferFromReply),
}
