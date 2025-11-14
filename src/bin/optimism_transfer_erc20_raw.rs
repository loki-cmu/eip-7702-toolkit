use alloy::network::EthereumWallet;
use alloy::network::TransactionBuilder;
use alloy::{
    primitives::{Address, ChainId, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use anyhow::Result;

#[allow(dead_code)]
fn display_eth_balance(balance: U256) {
    // exact decimal ETH (18 decimals)
    let wei_per_eth = U256::from(1_000_000_000_000_000_000u128);
    let eth_whole = balance / wei_per_eth;
    let remainder = balance % wei_per_eth;

    // pad remainder to 18 digits
    let mut rem_str = remainder.to_string();
    while rem_str.len() < 18 {
        rem_str.insert(0, '0');
    }
    // trim trailing zeros for shorter display (but keep "0" if exact)
    let rem_trimmed = rem_str.trim_end_matches('0');
    let eth_display = if rem_trimmed.is_empty() {
        format!("{}", eth_whole)
    } else {
        format!("{}.{}", eth_whole, rem_trimmed)
    };

    println!("Alice balance (ETH): {} ETH", eth_display);
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to an Optimism node.
    let chain_id: ChainId = 11155420;
    let rpc_url = "https://sepolia.optimism.io".parse::<Url>()?;
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let alice_signer: PrivateKeySigner =
        "cafabb29b447a5419ad84e6b7567a477ae9e3e5700d10733b17c472546354237".parse()?;
    let bob_signer: PrivateKeySigner =
        "43f982bd20bad345407d7a6e8f897696edaba5a2c56c5da70549955ab2845144".parse()?;

    let alice: Address = alice_signer.address();
    let bob: Address = bob_signer.address();

    // let balance = provider.get_balance(alice).await?;
    // println!("Alice balance: {}", balance);
    // display_eth_balance(balance);
    // let balance = provider.get_balance(bob).await?;
    // display_eth_balance(balance);

    let value_one_gwei = U256::from(1_000_000_000); // 1 Gwei
    // Build a transaction to send 1 Gwei from Alice to Bob.
    // The `from` field is automatically filled to the first signer's address (Alice).

    // Value 0.000000001 ETH
    // Transaction fee 0.000021000005250232 ETH
    // Gas price 0.00000000100000025 ETH (1.00000025 Gwei)
    // Gas usage & limit by txn 21,000 | 21,000 100%
    // Gas fees (Gwei) Base: 0.00000025| Max: 20| Max priority: 1
    // L1 gas used by txn 1,600
    // L1 gas price 0.000000000000000012 ETH (0.000000012 Gwei)
    // L1 fee 0.000000000000000232 ETH

    let tx = TransactionRequest::default()
        .to(bob)
        .nonce(0)
        .with_chain_id(chain_id)
        .value(value_one_gwei)
        .gas_limit(21_000)
        .max_priority_fee_per_gas(1_000_000_000)
        .max_fee_per_gas(20_000_000_000);

    let wallet = EthereumWallet::from(alice_signer);
    let tx_envelope = tx.build(&wallet).await?;

    let receipt = provider
        .send_tx_envelope(tx_envelope)
        .await?
        .get_receipt()
        .await?;

    println!("Sent transaction: {}", receipt.transaction_hash);

    // explorer
    // https://testnet-explorer.optimism.io/

    assert_eq!(receipt.from, alice);
    assert_eq!(receipt.to, Some(bob));

    Ok(())
}

#[cfg(test)]
mod tests {

    use alloy::{primitives::Address, signers::local::PrivateKeySigner};

    #[tokio::test]
    async fn test_address_gen() {
        let alice_signer: PrivateKeySigner =
            "cafabb29b447a5419ad84e6b7567a477ae9e3e5700d10733b17c472546354237"
                .parse()
                .unwrap();
        let bob_signer: PrivateKeySigner =
            "43f982bd20bad345407d7a6e8f897696edaba5a2c56c5da70549955ab2845144"
                .parse()
                .unwrap();

        let alice: Address = alice_signer.address();
        let bob: Address = bob_signer.address();

        println!("Alice Address: {alice}, Bob Address: {bob}");

        assert!(
            alice
                .to_string()
                .eq_ignore_ascii_case("0x6fa23657bacc2b2114d362bbf615490efae90efb")
        );
        assert!(
            bob.to_string()
                .eq_ignore_ascii_case("0xc3f324e89cb845cb22092f8cebca75864274bea2")
        );
    }
}
