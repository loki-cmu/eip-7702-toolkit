//! Example of spinning up a local Reth node instance and connecting it with a provider.

use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::{
    node_bindings::Reth,
    providers::{Provider, ProviderBuilder},
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a local Reth node.
    // Ensure `reth` is available in $PATH.
    let reth = Reth::new().dev().disable_discovery().instance(1).spawn();
    let provider = ProviderBuilder::new().connect_http(reth.endpoint().parse()?);

    let chain_id = provider.get_chain_id().await?;

    println!(
        "Reth running at: {} with chain id: {chain_id}",
        reth.endpoint()
    );

    assert_eq!(chain_id, 1337);
    assert_eq!(reth.http_port(), 8545);
    assert_eq!(reth.ws_port(), 8546);
    assert_eq!(reth.auth_port(), Some(8551));
    assert_eq!(reth.p2p_port(), None);

    let accounts: Vec<Address> = provider.get_accounts().await?;
    // dbg!(&reth.genesis());
    // dbg!(&accounts);

    // use default funded accounts from reth
    let signer_alice: PrivateKeySigner =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse()?;
    let address_alice = signer_alice.address();

    let signer_bob: PrivateKeySigner =
        "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".parse()?;
    let address_bob = signer_bob.address();
    assert!(
        address_alice
            .to_string()
            .eq_ignore_ascii_case(&accounts[0].to_string()),
    );

    let value_one_gwei = U256::from(1_000_000_000); // 1 Gwei

    let tx = TransactionRequest::default()
        .to(address_bob)
        .nonce(0)
        .with_chain_id(chain_id)
        .value(value_one_gwei)
        .gas_limit(21_000)
        .max_priority_fee_per_gas(1_000_000_000)
        .max_fee_per_gas(20_000_000_000);

    let wallet = EthereumWallet::from(signer_alice);
    let tx_envelope = tx.build(&wallet).await?;

    let receipt = provider
        .send_tx_envelope(tx_envelope)
        .await?
        .get_receipt()
        .await?;

    println!("Sent transaction: {}", receipt.transaction_hash);

    // explorer
    // https://testnet-explorer.optimism.io/

    assert_eq!(receipt.from, address_alice);
    assert_eq!(receipt.to, Some(address_bob));

    Ok(())
}
