use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use anyhow::Result;

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
    let rpc_url = "https://sepolia.optimism.io".parse::<Url>()?;
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let alice_signer: PrivateKeySigner =
        "cafabb29b447a5419ad84e6b7567a477ae9e3e5700d10733b17c472546354237".parse()?;
    let bob_signer: PrivateKeySigner =
        "43f982bd20bad345407d7a6e8f897696edaba5a2c56c5da70549955ab2845144".parse()?;

    let alice: Address = alice_signer.address();
    let bob: Address = bob_signer.address();

    let balance = provider.get_balance(alice).await?;
    display_eth_balance(balance);
    let balance = provider.get_balance(bob).await?;
    display_eth_balance(balance);

    // println!("Sent transaction: {tx_hash}");

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
