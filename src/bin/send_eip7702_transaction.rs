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
use hex::encode as hex_encode;

// Codegen from embedded Solidity code and precompiled bytecode.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc, bytecode = "6080806040523460135760c9908160188239f35b5f80fdfe6004361015600b575f80fd5b5f3560e01c80637b3ab2d014605f57639ee1a440146027575f80fd5b34605b575f366003190112605b577f2d67bb91f17bca05af6764ab411e86f4ddf757adb89fcec59a7d21c525d417125f80a1005b5f80fd5b34605b575f366003190112605b577fbcdfe0d5b27dd186282e187525415c57ea3077c34efb39148111e4d342e7ab0e5f80a100fea2646970667358221220f6b42b522bc9fb2b4c7d7e611c7c3e995d057ecab7fd7be4179712804c886b4f64736f6c63430008190033")]
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

fn print_code_status(code: &[u8], label: &str) {
    let code_hex = format!("0x{}", hex_encode(code));
    if code.is_empty() {
        println!("{label}账户无代码，已恢复为普通EOA");
    } else if let Some(delegate_addr) = code_hex.strip_prefix("0xef0100") {
        // let delegate_addr = &code_hex[8..];
        println!("{label}账户仍为EIP-7702委托账户，委托地址为: 0x{delegate_addr}");
    } else {
        println!("{label}账户有代码，但不是EIP-7702委托代码");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let anvil = Anvil::new().arg("--hardfork").arg("prague").try_spawn()?;
    println!("{}", anvil.endpoint());
    println!("{}", anvil.chain_id());

    let alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let bob: PrivateKeySigner = anvil.keys()[1].clone().into();
    let alice2: PrivateKeySigner = anvil.keys()[2].clone().into();

    println!("alice address: {:?}", alice.address());
    println!("bob address: {:?}", bob.address());
    println!("alice 2 address: {:?}", alice2.address());

    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new()
        .wallet(bob.clone())
        .connect_http(rpc_url.clone());

    let balance_alice = provider.get_balance(alice.address()).await?;
    let balance_bob = provider.get_balance(bob.address()).await?;
    println!("alice balance init: {}", balance_alice);
    println!("bob balance init: {}", balance_bob);

    let contract = Log::deploy(&provider).await?;

    // Alice signs delegation to contract
    let nonce = provider.get_transaction_count(alice.address()).await?;
    let signed_auth_1 = sign_authorization(&alice, anvil.chain_id(), *contract.address(), nonce);

    // Alice signs delegation to contract
    let nonce = provider.get_transaction_count(alice2.address()).await?;
    let signed_auth_2 = sign_authorization(&alice2, anvil.chain_id(), *contract.address(), nonce);

    let emit_hello_calldata = contract.emitHello().calldata().to_owned();

    let tx = TransactionRequest::default()
        .with_to(alice.address())
        .with_authorization_list(vec![signed_auth_1, signed_auth_2])
        .with_input(emit_hello_calldata);

    let pending_tx = provider.send_transaction(tx).await?;
    println!("Pending transaction... {}", pending_tx.tx_hash());

    let receipt = pending_tx.get_receipt().await?;
    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );

    assert!(receipt.status());
    assert_eq!(receipt.from, bob.address());
    assert_eq!(receipt.to, Some(alice.address()));
    assert_eq!(receipt.inner.logs().len(), 1);
    assert_eq!(receipt.inner.logs()[0].address(), alice.address());

    let balance_alice = provider.get_balance(alice.address()).await?;
    let balance_bob = provider.get_balance(bob.address()).await?;
    println!("bob sponsor代付 gas");
    println!("alice balance: {}", balance_alice);
    println!("bob balance: {}", balance_bob);
    let balance_alice2 = provider.get_balance(alice2.address()).await?;
    println!("alice2 balance: {}", balance_alice2);

    let code = provider.get_code_at(alice.address()).await?;
    print_code_status(&code, "Alice");

    println!("\nRevoke Alice's Delegation\n");

    // Alice signs revocation (address = zero)
    let nonce = provider.get_transaction_count(alice.address()).await?;
    let revoke_auth = sign_authorization(&alice, anvil.chain_id(), Address::ZERO, nonce);

    let tx = TransactionRequest::default()
        .with_to(alice.address())
        .with_authorization_list(vec![revoke_auth]);

    let pending_tx = provider.send_transaction(tx).await?;
    println!("Pending transaction... {}", pending_tx.tx_hash());

    let balance_alice = provider.get_balance(alice.address()).await?;
    println!("alice balance: {}", balance_alice);

    let code = provider.get_code_at(alice.address()).await?;
    print_code_status(&code, "Alice");

    Ok(())
}
