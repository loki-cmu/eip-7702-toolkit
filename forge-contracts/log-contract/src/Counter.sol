// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

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