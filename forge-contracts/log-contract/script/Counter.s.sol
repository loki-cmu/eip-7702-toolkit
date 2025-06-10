// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {EventLogger} from "../src/Counter.sol";

contract LogScript is Script {
    EventLogger public logger;

    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // Deploy the EventLogger contract
        logger = new EventLogger();
        console.log("EventLogger contract deployed at:", address(logger));

        // Emit some example events
        logger.emitHello();
        logger.emitWorld("Ethereum");

        vm.stopBroadcast();
    }
}
