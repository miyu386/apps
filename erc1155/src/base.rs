use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub trait Erc1155TokenBase {
    fn init(&mut self, name: String, symbol: String, base_uri: String);

    // https://docs.openzeppelin.com/contracts/4.x/api/token/erc1155

    fn balance_of(&self, account: &ActorId, token_id: U256);
    fn balance_of_batch(&self, accounts: &[ActorId], token_ids: &[U256]);
    fn set_approval_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool);
    fn is_approved_for_all(&self, account: &ActorId, operator: &ActorId);
    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, token_id: U256, value: U256);
    fn safe_batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        token_id: U256,
        values: &[U256],
    );

    fn owner_of(&self, token_id: U256);
}
