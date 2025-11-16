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
    #[sol(rpc, bytecode = "0x6080604052348015600e575f5ffd5b506103108061001c5f395ff3fe608060405234801561000f575f5ffd5b5060043610610034575f3560e01c806324990efb146100385780637b3ab2d014610054575b5f5ffd5b610052600480360381019061004d9190610213565b61005e565b005b61005c610098565b005b7f7005e8379b009089c7e9731341ce34751090e83c1a673673574e6cc97aadc5bc8160405161008d91906102ba565b60405180910390a150565b7fbcdfe0d5b27dd186282e187525415c57ea3077c34efb39148111e4d342e7ab0e60405160405180910390a1565b5f604051905090565b5f5ffd5b5f5ffd5b5f5ffd5b5f5ffd5b5f601f19601f8301169050919050565b7f4e487b71000000000000000000000000000000000000000000000000000000005f52604160045260245ffd5b610125826100df565b810181811067ffffffffffffffff82111715610144576101436100ef565b5b80604052505050565b5f6101566100c6565b9050610162828261011c565b919050565b5f67ffffffffffffffff821115610181576101806100ef565b5b61018a826100df565b9050602081019050919050565b828183375f83830152505050565b5f6101b76101b284610167565b61014d565b9050828152602081018484840111156101d3576101d26100db565b5b6101de848285610197565b509392505050565b5f82601f8301126101fa576101f96100d7565b5b813561020a8482602086016101a5565b91505092915050565b5f60208284031215610228576102276100cf565b5b5f82013567ffffffffffffffff811115610245576102446100d3565b5b610251848285016101e6565b91505092915050565b5f81519050919050565b5f82825260208201905092915050565b8281835e5f83830152505050565b5f61028c8261025a565b6102968185610264565b93506102a6818560208601610274565b6102af816100df565b840191505092915050565b5f6020820190508181035f8301526102d28184610282565b90509291505056fea26469706673582212200253eb0f024de2d20ede56ffaa1c5281e41abf0dbe6d587d94c4b9207559af9c64736f6c634300081d0033")]
    contract EventLogger {
        event Hello();
        event World(string world_name);

        function emitHello() public {
            emit Hello();
        }

        function emitWorld(string memory world_name) public {
            emit World(world_name);
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
    let anvil = Anvil::new().arg("--hardfork").arg("prague").try_spawn()?;
    println!("{}", anvil.endpoint());
    println!("{}", anvil.chain_id());

    let bob_sponsor: PrivateKeySigner = anvil.keys()[0].clone().into();
    let alice_1: PrivateKeySigner = anvil.keys()[1].clone().into();
    let alice_2: PrivateKeySigner = anvil.keys()[2].clone().into();

    println!("bob address: {:?}", bob_sponsor.address());
    println!("alice1 address: {:?}", alice_1.address());
    println!("dave1 address: {:?}", bob_sponsor.address());
    println!("alice2 address: {:?}", alice_2.address());
    println!("dave2 address: {:?}", bob_sponsor.address());

    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new()
        .wallet(bob_sponsor.clone())
        .connect_http(rpc_url.clone());

    let balance_bob = provider.get_balance(bob_sponsor.address()).await?;
    let balance_alice_1 = provider.get_balance(alice_1.address()).await?;
    let balance_alice_2 = provider.get_balance(alice_2.address()).await?;

    println!("bob balance init: {}", balance_bob);
    println!("alice1 balance init: {}", balance_alice_1);
    println!("alice2 balance init: {}", balance_alice_2);

    let contract = EventLogger::deploy(&provider).await?;
    let balance_bob = provider.get_balance(bob_sponsor.address()).await?;
    println!("bob balance after deploy contract: {}", balance_bob);

    // each Alice sign delegation to contract
    let nonce = provider.get_transaction_count(alice_1.address()).await?;
    let signed_auth_1 = sign_authorization(&alice_1, anvil.chain_id(), *contract.address(), nonce);
    let nonce = provider.get_transaction_count(alice_2.address()).await?;
    let signed_auth_2 = sign_authorization(&alice_2, anvil.chain_id(), *contract.address(), nonce);

    let calldata = contract
        .emitWorld("alice1 world".to_string())
        .calldata()
        .to_owned();

    // For alice2
    let tx = TransactionRequest::default()
        .with_to(*contract.address())
        .with_authorization_list(vec![signed_auth_1, signed_auth_2])
        .with_input(calldata);
    let _ = provider.send_transaction(tx).await?;

    let balance_bob = provider.get_balance(bob_sponsor.address()).await?;
    let balance_alice_1 = provider.get_balance(alice_1.address()).await?;
    let balance_alice_2 = provider.get_balance(alice_2.address()).await?;

    println!("bob balance after: {}", balance_bob);
    println!("alice1 balance after: {}", balance_alice_1);
    println!("alice2 balance after: {}", balance_alice_2);

    Ok(())
}
