use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::H256;
const GAS_RESERVE: u64 = 500_000_000;

pub async fn ft_transfer(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferData {
        from: H256::from_slice(from.as_ref()),
        to: H256::from_slice(to.as_ref()),
        amount,
    };

    let transfer_response: FTEvent = msg::send_and_wait_for_reply(
        *token_id,
        FTAction::Transfer(transfer_data),
        exec::gas_available() - GAS_RESERVE,
        0,
    )
    .await
    .unwrap();
    if let FTEvent::Transfer(transfer_response) = transfer_response {
        if transfer_response.amount != amount {
            panic!("error in transfer");
        }
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum FTEvent {
    Transfer(TransferData),
    Approval,
    TotalIssuance,
    Balance(u128),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum FTAction {
    Mint,
    Burn,
    Transfer(TransferData),
    TransferFrom,
    Approve,
    IncreaseAllowance,
    DecreaseAllowance,
    TotalIssuance,
    BalanceOf(H256),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct TransferData {
    pub from: H256,
    pub to: H256,
    pub amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct ApproveData {
    pub owner: H256,
    pub spender: H256,
    pub amount: u128,
}
