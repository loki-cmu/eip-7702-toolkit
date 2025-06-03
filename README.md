### How to Run the Example

```
cargo run --bin send_eip7702_transaction
```

### Delegate Contract Reference

https://github.com/okx/wallet-core/tree/main

### 文件头部与依赖

```rust
//! Example showing how to send an [EIP-7702](<https://github.com/ethereum/EIPs/blob/master/EIPS/eip-7702.md>) transaction.

use alloy::{
    eips::eip7702::Authorization,
    network::{TransactionBuilder, TransactionBuilder7702},
    node_bindings::Anvil,
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, SignerSync},
    sol,
};
use anyhow::Result;

```

- 说明本例演示如何发送 EIP-7702 交易。
- 引入了 alloy 相关模块，包括 EIP-7702 授权、交易构建、Anvil 测试节点、U256 数字类型、Provider、交易请求、签名器等。
- 使用 anyhow 作为错误处理库。

---

### Solidity 合约嵌入

```rust
sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "...")]
    contract Log {
        #[derive(Debug)]
        event Hello();
        event World();

        function emitHello() public {
            emit Hello();
        }

        function emitWorld() public {
            emit World();
        }
    }
);

```

- 使用 alloy 的 `sol!` 宏嵌入了一个简单的 Solidity 合约 `Log`，包含两个事件和两个函数，分别触发 `Hello` 和 `World` 事件。
- 通过 `bytecode` 字符串直接嵌入了编译好的合约字节码，便于后续部署。

---

### 主函数入口

```rust
#[tokio::main]
async fn main() -> Result<()> {

```

- 使用 tokio 异步运行时，主函数为异步函数。

### 启动本地 Anvil 节点

```rust
    let anvil = Anvil::new().arg("--hardfork").arg("prague").try_spawn()?;

```

- 启动一个本地的 Anvil 节点（以太坊测试节点），并指定硬分叉为 Prague（布拉格），以支持 EIP-7702。

### 创建 Alice 和 Bob 两个用户

```rust
    let alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let bob: PrivateKeySigner = anvil.keys()[1].clone().into();

```

- 从 Anvil 节点获取的私钥中，取出两个账户，分别作为 Alice 和 Bob。
- Alice 用于签名授权，Bob 用于实际发送交易。

### 创建 Provider

```rust
    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new().wallet(bob.clone()).connect_http(rpc_url);

```

- 创建一个 provider，连接到本地 Anvil 节点，并用 Bob 的钱包进行签名。

### 部署合约

```rust
    let contract = Log::deploy(&provider).await?;

```

- 使用 provider 部署上面定义的 Log 合约，返回合约实例。

### 构造授权对象

```rust
    let authorization = Authorization {
        chain_id: U256::from(anvil.chain_id()),
        address: *contract.address(),
        nonce: provider.get_transaction_count(alice.address()).await?,
    };

```

- 构造 EIP-7702 的授权对象，包含链 ID、合约地址（将作为 authority code）、Alice 的 nonce。

### Alice 签名授权

```rust
    let signature = alice.sign_hash_sync(&authorization.signature_hash())?;
    let signed_authorization = authorization.into_signed(signature);

```

- Alice 对授权对象的哈希进行签名，生成签名后的授权对象。

### 构造调用数据, 也可以是合约初始化接口

```rust
    let call = contract.emitHello();
    let emit_hello_calldata = call.calldata().to_owned();

```

- 构造调用合约 `emitHello()` 的 calldata。

### 构建 EIP-7702 交易

```rust
    let tx = TransactionRequest::default()
        .with_to(alice.address())
        .with_authorization_list(vec![signed_authorization])
        .with_input(emit_hello_calldata);

```

- 构建交易请求，目标地址为 Alice（即 authority code 会被设置为合约），带上授权列表和 calldata。

### 发送交易并等待广播

```rust
    let pending_tx = provider.send_transaction(tx).await?;
    println!("Pending transaction... {}", pending_tx.tx_hash());

```

- 通过 provider 发送交易，获取 pending transaction，并打印交易哈希。

### 等待交易上链并获取回执

```rust
    let receipt = pending_tx.get_receipt().await?;
    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );

```

- 等待交易被打包进区块，获取交易回执，并打印区块号。

### 验证交易结果

```rust
    assert!(receipt.status());
    assert_eq!(receipt.from, bob.address());
    assert_eq!(receipt.to, Some(alice.address()));
    assert_eq!(receipt.inner.logs().len(), 1);
    assert_eq!(receipt.inner.logs()[0].address(), alice.address());

```

- 检查交易状态为成功。
- 检查发送方为 Bob，接收方为 Alice。
- 检查日志数量为 1，且日志地址为 Alice（即 authority code）。
- 打印日志，Bob支付了交易费用。

### 日志

```
http://localhost:56863
31337
alice address: 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
bob address: 0x70997970c51812dc3a010c7d01b50e0d17dc79c8
alice balance init: 10000000000000000000000
bob balance init: 10000000000000000000000
Pending transaction... 0xf0166c3f7fe2036bf3e4a2aa1f222a6651c47c3c6c6f2e42b7e7779aae6026b6
Transaction included in block 2
bob sponsor代付 gas
alice balance: 10000000000000000000000
bob balance: 9999999870255196639093
Alice账户仍为EIP-7702委托账户，委托地址为: 0x8464135c8f25da09e49bc8782676a84730c318bc

Revoke Alice's Delegation

Pending transaction... 0xb75ea4260ccf9c6b6b58d2f1989f9965e774937be9c8cbef71c195e2138e9485
alice balance: 10000000000000000000000
Alice账户无代码，已恢复为普通EOA

Process finished with exit code 0
```

### Alice Revoke Delegation Contract的实现

1. 获取 Alice 当前的 nonce，确保授权唯一且不可重放。
2. 调用 `sign_authorization`，将授权地址设为 `Address::ZERO`（全零地址），Alice 用私钥签名，生成撤销授权对象。
3. 构造交易，目标地址为 Alice 自己，带上撤销授权（`authorization_list`），不需要 calldata。
4. 发送该交易，链上节点收到后会识别为撤销委托，将 Alice 账户恢复为普通 EOA（无合约代码）。

这样 Alice 的账户就不再是 EIP-7702 委托账户，任何合约授权都被移除。

---

## 总结

send_eip7702_transaction 演示了如何在本地 Anvil 节点上，使用 EIP-7702 机制，Alice 授权 Bob 以它的名义调用合约，Bob
构造并发送交易，最终合约事件被成功触发。Bob代付交易费用，Alice账户仍为EIP-7702委托账户，Alice可以随时撤销合约授权委托。