//! Example of how to transfer ERC20 tokens from one account to another.

use alloy::node_bindings::Anvil;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    sol,
};

// Codegen from embedded Solidity code and precompiled bytecode.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "0x6080604052348015600e575f5ffd5b506106748061001c5f395ff3fe60806040526004361061001d575f3560e01c8063beabacc814610021575b5f5ffd5b61003b6004803603810190610036919061034f565b61003d565b005b8273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16146100ab576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100a29061041f565b60405180910390fd5b803410156100ee576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100e590610487565b60405180910390fd5b5f8273ffffffffffffffffffffffffffffffffffffffff1682604051610113906104d2565b5f6040518083038185875af1925050503d805f811461014d576040519150601f19603f3d011682016040523d82523d5f602084013e610152565b606091505b5050905080610196576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161018d90610530565b60405180910390fd5b81341115610253575f8473ffffffffffffffffffffffffffffffffffffffff1683346101c2919061057b565b6040516101ce906104d2565b5f6040518083038185875af1925050503d805f8114610208576040519150601f19603f3d011682016040523d82523d5f602084013e61020d565b606091505b5050905080610251576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610248906105f8565b60405180910390fd5b505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516102b09190610625565b60405180910390a350505050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f6102eb826102c2565b9050919050565b6102fb816102e1565b8114610305575f5ffd5b50565b5f81359050610316816102f2565b92915050565b5f819050919050565b61032e8161031c565b8114610338575f5ffd5b50565b5f8135905061034981610325565b92915050565b5f5f5f60608486031215610366576103656102be565b5b5f61037386828701610308565b935050602061038486828701610308565b92505060406103958682870161033b565b9150509250925092565b5f82825260208201905092915050565b7f4f6e6c7920416c6963652063616e20696e69746961746520746865207472616e5f8201527f7366657200000000000000000000000000000000000000000000000000000000602082015250565b5f61040960248361039f565b9150610414826103af565b604082019050919050565b5f6020820190508181035f830152610436816103fd565b9050919050565b7f496e73756666696369656e74204554482073656e7400000000000000000000005f82015250565b5f61047160158361039f565b915061047c8261043d565b602082019050919050565b5f6020820190508181035f83015261049e81610465565b9050919050565b5f81905092915050565b50565b5f6104bd5f836104a5565b91506104c8826104af565b5f82019050919050565b5f6104dc826104b2565b9150819050919050565b7f5472616e73666572206661696c656400000000000000000000000000000000005f82015250565b5f61051a600f8361039f565b9150610525826104e6565b602082019050919050565b5f6020820190508181035f8301526105478161050e565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6105858261031c565b91506105908361031c565b92508282039050818111156105a8576105a761054e565b5b92915050565b7f526566756e64206661696c6564000000000000000000000000000000000000005f82015250565b5f6105e2600d8361039f565b91506105ed826105ae565b602082019050919050565b5f6020820190508181035f83015261060f816105d6565b9050919050565b61061f8161031c565b82525050565b5f6020820190506106385f830184610616565b9291505056fea26469706673582212206f43b7c6930439eb324fe4fc8a642289633b6421da04ff2951ad16e78fa39c4564736f6c634300081d0033")]
    contract SimpleTransfer {
        // Event to emit when a transfer occurs
        event Transfer(address indexed from, address indexed to, uint256 amount);

        // Function to transfer ETH from Alice to Bob
        function transfer(address from_alice, address to_bob, uint256 amount) public payable {
            require(msg.sender == from_alice, "Only Alice can initiate the transfer");
            require(msg.value >= amount, "Insufficient ETH sent");

            // Transfer ETH to Bob
            (bool success,) = to_bob.call{value: amount}("");
            require(success, "Transfer failed");

            // Refund excess ETH if any
            if (msg.value > amount) {
                (bool refundSuccess,) = from_alice.call{value: msg.value - amount}("");
                require(refundSuccess, "Refund failed");
            }

            emit Transfer(from_alice, to_bob, amount);
        }
    }
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Spin up a forked Anvil node.
    // Ensure `anvil` is available in $PATH.
    let anvil = Anvil::new().arg("--hardfork").arg("prague").try_spawn()?;

    // Create two users, Alice and Bob.
    let alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let bob: PrivateKeySigner = anvil.keys()[1].clone().into();

    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new()
        .wallet(alice.clone())
        .connect_http(rpc_url.clone());

    // Deploy the `SimpleTransfer` contract.
    let contract = SimpleTransfer::deploy(provider.clone()).await?;

    // Get initial balances
    let alice_balance_before = provider.get_balance(alice.address()).await?;
    let bob_balance_before = provider.get_balance(bob.address()).await?;

    println!("Alice balance before: {}", alice_balance_before);
    println!("Bob balance before: {}", bob_balance_before);

    // Transfer amount (1 ETH)
    let transfer_amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH in wei

    // Call transfer function from Alice to Dave
    let tx = contract
        .transfer(alice.address(), bob.address(), transfer_amount)
        .value(transfer_amount)
        .send()
        .await?
        .watch()
        .await?;

    println!("Transfer transaction hash: {}", tx);

    // Get final balances
    let alice_balance_after = provider.get_balance(alice.address()).await?;
    let bob_balance_after = provider.get_balance(bob.address()).await?;

    println!("Alice balance after: {}", alice_balance_after);
    println!("Bob balance after: {}", bob_balance_after);

    assert_eq!(
        bob_balance_after - bob_balance_before,
        transfer_amount,
        "Dave's balance should increase by transfer amount"
    );

    Ok(())
}
