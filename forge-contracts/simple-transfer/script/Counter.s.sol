// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import {Script, console} from "forge-std/Script.sol";
import {SimpleTransfer} from "../src/Counter.sol";

contract SimpleTransferScript is Script {
    SimpleTransfer public simpleTransfer;

    function setUp() public {}

    function run() public {
        // Get the private key from environment variable
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        // Start broadcasting transactions
        vm.startBroadcast(deployerPrivateKey);

        // Deploy the ETHTransfer contract
        simpleTransfer = new SimpleTransfer();
        console.log("ETHTransfer deployed at:", address(simpleTransfer));

        vm.stopBroadcast();
    }
}