# ERC1155

## Interface

### function

```js
    function balanceOf(address account, uint256 id) external view returns (uint256);

    function balanceOfBatch(address[] calldata accounts, uint256[] calldata ids)
        external
        view
        returns (uint256[] memory);

    function setApprovalForAll(address operator, bool approved) external;

    function isApprovedForAll(address account, address operator) external view returns (bool);

    function safeTransferFrom(
        address from,
        address to,
        uint256 id,
        uint256 amount,
        bytes calldata data
    ) external;

    function safeBatchTransferFrom(
        address from,
        address to,
        uint256[] calldata ids,
        uint256[] calldata amounts,
        bytes calldata data
    ) external;
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
```
