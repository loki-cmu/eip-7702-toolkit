//! Example of using `MnemonicBuilder` to access a wallet from a mnemonic phrase.

use alloy::signers::local::{MnemonicBuilder, coins_bip39::English};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let phrase = "sport palace slab globe verify coach own update typical vintage awful divide";
    let index = 0;

    // Access mnemonic phrase with password.
    // Child key at derivation path: m/44'/60'/0'/0/{index}.
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .index(index)?
        .build()?;

    assert_eq!(
        "0x0F37B84E1FEb424B5be4BEaa046407158262A355",
        wallet.address().to_string()
    );
    println!("Wallet: {}", wallet.address());
    // let credential = wallet.into_credential();
    // let v = credential.verifying_key();

    let bytes = [
        3, 106, 216, 69, 214, 58, 31, 209, 31, 103, 163, 66, 57, 63, 50, 189, 216, 42, 34, 125,
        131, 166, 203, 80, 164, 242, 45, 12, 242, 122, 53, 100, 171,
    ];

    let pk_string: String = bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
    println!("Hex: {}", pk_string);

    assert_eq!(
        "036ad845d63a1fd11f67a342393f32bdd82a227d83a6cb50a4f22d0cf27a3564ab",
        pk_string
    );

    Ok(())
}
