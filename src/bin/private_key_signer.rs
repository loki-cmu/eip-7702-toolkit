//! Example of using a local wallet to sign and send a transaction.

use alloy::{
    primitives::U256,
    rpc::client::{ClientBuilder, ReqwestClient},
    signers::local::PrivateKeySigner,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let signer: PrivateKeySigner =
        "d2eb31e7ec97467f3e382903851bced12ba44fc230d93cdddcfd7726c94a2f6e".parse()?;

    assert_eq!(
        "0xCc88733454E3eB0760fee4C4fF87f42492EEB95B",
        signer.address().to_string()
    );
    // alice
    let address = signer.address();

    // --------------

    let end_point = "https://go.getblock.io/065ee66be50a4037a8c71fb7e52ac013";
    let url = end_point.parse()?;

    // Instantiate a new client over a transport.
    let client: ReqwestClient = ClientBuilder::default().http(url);

    // Prepare a batch request to the server.
    let mut batch = client.new_batch();

    // Batches serialize params immediately. So we need to handle the result when
    // adding calls.
    let block_number_fut = batch.add_call("eth_blockNumber", &()).unwrap();
    let balance_fut = batch
        .add_call("eth_getBalance", &(address, "latest"))
        .unwrap();

    // Make sure to send the batch!
    batch.send().await.unwrap();

    // After the batch is complete, we can get the results.
    // Note that requests may error separately!
    let block_number: String = block_number_fut.await.unwrap();
    let balance: String = balance_fut.await.unwrap();

    dbg!(&block_number);
    dbg!(&balance);

    // Convert hex balance to U256 and then to ETH
    let balance_wei = U256::from_str_radix(&balance[2..], 16)?;
    dbg!(balance_wei);

    /*
    &block_number = "0x7bf9fd"
    &balance = "0xb1a2bc2ec50000"
    balance_wei = 50000000000000000
    */

    Ok(())
}
