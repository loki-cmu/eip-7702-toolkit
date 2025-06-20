// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import {Script, console} from "forge-std/Script.sol";
import {SimpleTransfer} from "../src/Counter.sol";

contract SimpleTransferScript is Script {
    SimpleTransfer public simpleTransfer;

    function setUp() public {}

    function run() public view {
        // Get the private key from environment variable
        // uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        // Start broadcasting transactions
        // vm.startBroadcast(deployerPrivateKey);

        // Deploy the ETHTransfer contract
        // simpleTransfer = new SimpleTransfer();
        // console.log("ETHTransfer deployed at:", address(simpleTransfer));

        // vm.stopBroadcast();

        // Example parameters for signSignature
        // The private key is now hardcoded for testing.
        uint256 alicePrivateKey = 0xd2eb31e7ec97467f3e382903851bced12ba44fc230d93cdddcfd7726c94a2f6e;
        address alice = vm.addr(alicePrivateKey);
        address bob = 0x87D20bDC94DCB9bF27cCf59E2B7be7C5afBac84b; // Bob's address
        uint256 transferAmount = 0.0001 ether;
        uint256 nonce = vm.getNonce(alice);
        console.logUint(nonce);
        address contractAddr = 0xA773b4CAfe39cf46e524F0f06e3Bd6C7eB396eba;

        bytes memory signature = signSignature(alicePrivateKey, alice, bob, transferAmount, nonce, contractAddr);
        console.logBytes(signature);
    }

    function signSignature(
        uint256 alicePrivateKey,   // Alice's private key for signing
        address alice,             // Sender address
        address bob,               // Recipient address
        uint256 transferAmount,    // Amount to transfer
        uint256 nonce,             // Unique nonce for replay protection
        address contractAddr       // Contract address involved in the transfer
    ) public pure returns (bytes memory) {
        // 1. Create a hash of the transfer parameters
        bytes32 hash = keccak256(abi.encodePacked(alice, bob, transferAmount, nonce, contractAddr));
        // 2. Sign the hash with Alice's private key, using Ethereum's message prefix
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePrivateKey, keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash)));
        // 3. Pack the signature components into a single bytes array
        bytes memory signature = abi.encodePacked(r, s, v);
        return signature;
    }
}