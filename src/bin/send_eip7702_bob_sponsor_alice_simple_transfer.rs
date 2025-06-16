//! Example: Bob sponsors gas for Alice to transfer ETH to Dave using EIP-7702

use alloy::{
    eips::eip7702::{Authorization, SignedAuthorization},
    network::{TransactionBuilder, TransactionBuilder7702},
    node_bindings::Anvil,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{SignerSync, local::PrivateKeySigner},
    sol,
};

// Codegen from embedded Solidity code and precompiled bytecode.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "0x6080604052348015600e575f5ffd5b506104cd8061001c5f395ff3fe60806040526004361061001d575f3560e01c8063beabacc814610021575b5f5ffd5b61003b6004803603810190610036919061029e565b61003d565b005b5f8273ffffffffffffffffffffffffffffffffffffffff16826040516100629061031b565b5f6040518083038185875af1925050503d805f811461009c576040519150601f19603f3d011682016040523d82523d5f602084013e6100a1565b606091505b50509050806100e5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016100dc90610389565b60405180910390fd5b813411156101a2575f8473ffffffffffffffffffffffffffffffffffffffff16833461011191906103d4565b60405161011d9061031b565b5f6040518083038185875af1925050503d805f8114610157576040519150601f19603f3d011682016040523d82523d5f602084013e61015c565b606091505b50509050806101a0576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161019790610451565b60405180910390fd5b505b8273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516101ff919061047e565b60405180910390a350505050565b5f5ffd5b5f73ffffffffffffffffffffffffffffffffffffffff82169050919050565b5f61023a82610211565b9050919050565b61024a81610230565b8114610254575f5ffd5b50565b5f8135905061026581610241565b92915050565b5f819050919050565b61027d8161026b565b8114610287575f5ffd5b50565b5f8135905061029881610274565b92915050565b5f5f5f606084860312156102b5576102b461020d565b5b5f6102c286828701610257565b93505060206102d386828701610257565b92505060406102e48682870161028a565b9150509250925092565b5f81905092915050565b50565b5f6103065f836102ee565b9150610311826102f8565b5f82019050919050565b5f610325826102fb565b9150819050919050565b5f82825260208201905092915050565b7f5472616e73666572206661696c656400000000000000000000000000000000005f82015250565b5f610373600f8361032f565b915061037e8261033f565b602082019050919050565b5f6020820190508181035f8301526103a081610367565b9050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52601160045260245ffd5b5f6103de8261026b565b91506103e98361026b565b9250828203905081811115610401576104006103a7565b5b92915050565b7f526566756e64206661696c6564000000000000000000000000000000000000005f82015250565b5f61043b600d8361032f565b915061044682610407565b602082019050919050565b5f6020820190508181035f8301526104688161042f565b9050919050565b6104788161026b565b82525050565b5f6020820190506104915f83018461046f565b9291505056fea264697066735822122004c971ab195684be10010586abf6ede90ae294a6fe8c53f8b46b0b8fce0ad41064736f6c634300081d0033")]
    contract SimpleTransfer {
        // Event to emit when a transfer occurs
        event Transfer(address indexed from, address indexed to, uint256 amount);

        // Function to transfer ETH from Alice to Bob
        function transfer(address from_alice, address to_bob, uint256 amount) public payable {
            // require(msg.sender == from_alice, "Only Alice can initiate the transfer");
            // require(msg.value >= amount, "Insufficient ETH sent");

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

fn sign_authorization(
    signer: &PrivateKeySigner,
    chain_id: u64,
    address: Address,
    nonce: u64,
) -> SignedAuthorization {
    let auth = Authorization {
        chain_id: U256::from(chain_id),
        address,
        nonce,
    };
    let sig = signer.sign_hash_sync(&auth.signature_hash()).unwrap();
    auth.into_signed(sig)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Spin up a forked Anvil node.
    // Ensure `anvil` is available in $PATH.
    let anvil = Anvil::new().arg("--hardfork").arg("prague").try_spawn()?;

    // Create three users: Alice, Bob (sponsor), and Dave
    let alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let bob: PrivateKeySigner = anvil.keys()[1].clone().into();
    let dave: PrivateKeySigner = anvil.keys()[2].clone().into();

    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new()
        .wallet(bob.clone())
        .connect_http(rpc_url.clone());

    // Deploy the `SimpleTransfer` contract.
    let contract = SimpleTransfer::deploy(provider.clone()).await?;

    // Get initial balances
    let alice_balance_before = provider.get_balance(alice.address()).await?;
    let bob_balance_before = provider.get_balance(bob.address()).await?;
    let dave_balance_before = provider.get_balance(dave.address()).await?;

    println!("Alice balance before: {}", alice_balance_before);
    println!("Bob balance before: {}", bob_balance_before);
    println!("Dave balance before: {}", dave_balance_before);

    // Alice signs delegation to contract
    let nonce = provider.get_transaction_count(alice.address()).await?;
    let signed_auth = sign_authorization(&alice, anvil.chain_id(), *contract.address(), nonce);

    // Transfer amount (1 ETH)
    let transfer_amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH in wei

    // Prepare the transaction with Alice's authorization
    let calldata = contract
        .transfer(alice.address(), dave.address(), transfer_amount)
        .calldata()
        .to_owned();

    // Create and send the transaction with Bob as the sponsor
    let tx = TransactionRequest::default()
        .with_to(alice.address())
        .with_authorization_list(vec![signed_auth])
        .with_input(calldata);

    let tx_hash = provider.send_transaction(tx).await?;
    println!("Transaction hash: {:?}", tx_hash);

    // Get final balances
    let alice_balance_after = provider.get_balance(alice.address()).await?;
    let bob_balance_after = provider.get_balance(bob.address()).await?;
    let dave_balance_after = provider.get_balance(dave.address()).await?;

    println!("Alice balance after: {}", alice_balance_after);
    println!("Bob balance after: {}", bob_balance_after);
    println!("Dave balance after: {}", dave_balance_after);

    assert_eq!(
        alice_balance_before - alice_balance_after,
        transfer_amount,
        "Alice's balance should decrease by transfer amount"
    );
    assert_eq!(
        dave_balance_after - dave_balance_before,
        transfer_amount,
        "Dave's balance should increase by transfer amount"
    );

    Ok(())
}
