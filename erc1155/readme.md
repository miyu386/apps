# ERC-1155

## Description
Tokens standards like ERC-20 and ERC-721 require a separate contract to be deployed for each token type or collection. This places a lot of redundant bytecode on the Ethereum blockchain and limits certain functionality by the nature of separating each token contract into its own permissioned address. With the rise of blockchain games and platforms like Enjin Coin, game developers may be creating thousands of token types, and a new type of token standard is needed to support them. However, ERC-1155 is not specific to games and many other applications can benefit from this flexibility.

New functionality is possible with this design such as transferring multiple token types at once, saving on transaction costs. Trading (escrow / atomic swaps) of multiple tokens can be built on top of this standard and it removes the need to “approve” individual token contracts separately. It is also easy to describe and mix multiple fungible or non-fungible token types in a single contract.

## ERC-1155 functions

```rust
  fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, token_id: U256);
  fn safe_batch_transfer_from(&mut self, from: &ActorId, to: &ActorId, token_id: U256);

  fn approve_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool);
  fn is_approved_for_all(&self, account: &ActorId)

  fn balance_of(&self, account: &ActorId);
  fn balance_of_batch(&self, accounts: &ActorId)
```

### event

```js
   event ApprovalForAll(address indexed account, address indexed operator, bool approved);

    event URI(string value, uint256 indexed id);

    event TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value);

    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );
```

### implementation

```rust
struct ERC1155Token {
    name: String,
    symbol: String,
    base_uri: String,
    token_id: U256,
    token_owner: BTreeMap<U256, ActorId>,
    token_approvals: BTreeMap<U256, ActorId>,
    owned_tokens_count: BTreeMap<ActorId, U256>,
    operator_approval: BTreeMap<ActorId, BTreeMap<ActorId, bool>>,
}

enum Event {
    TransferSingle(TransferSingle),
    TransferBatch(TransferBatch),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    TokenURI(String),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Mint(MintInput),
    Burn(BurnInput),
    Transfer(TransferInput),
    TransferFrom(TransferFromInput),
    Approve(ApproveInput),
    IncreaseAllowance(ApproveInput),
    DecreaseAllowance(ApproveInput),
    BalanceOf(ActorId),
    AddAdmin(ActorId),
    RemoveAdmin(ActorId),
}
```

## Refer

https://eips.ethereum.org/EIPS/eip-1155

https://github.com/gear-tech/apps/pull/5

https://github.com/gear-tech/apps/pull/9

https://github.com/gear-tech/apps/pull/10
