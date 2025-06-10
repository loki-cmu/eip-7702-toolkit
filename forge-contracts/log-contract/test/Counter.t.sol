// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {EventLogger} from "../src/Counter.sol";

contract LogTest is Test {
    EventLogger public logger;

    function setUp() public {
        logger = new EventLogger();
    }

    function test_EmitHello() public {
        // Expect the Hello event to be emitted
        vm.expectEmit(true, true, true, true);
        emit EventLogger.Hello();
        logger.emitHello();
    }

    function test_EmitWorld() public {
        string memory worldName = "TestWorld";
        // Expect the World event to be emitted with the correct parameter
        vm.expectEmit(true, true, true, true);
        emit EventLogger.World(worldName);
        logger.emitWorld(worldName);
    }

    function testFuzz_EmitWorld(string memory worldName) public {
        // Expect the World event to be emitted with the fuzzed parameter
        vm.expectEmit(true, true, true, true);
        emit EventLogger.World(worldName);
        logger.emitWorld(worldName);
    }
}
