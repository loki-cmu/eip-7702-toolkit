// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

contract SimpleTransfer {
    // Event to emit when a transfer occurs
    event Transfer(address indexed from, address indexed to, uint256 amount);

    // Function to transfer ETH from Alice to Bob
    function transfer(address from_alice, address to_bob, uint256 amount) public payable {
        require(msg.sender == from_alice, "Only Alice can initiate the transfer");
        require(msg.value >= amount, "Insufficient ETH sent");

        // Transfer ETH to Bob
        (bool success,) = to_bob.call{value: amount}("");
        require(success, "Transfer failed");

        // Refund excess ETH if any
        if (msg.value > amount) {
            (bool refundSuccess,) = from_alice.call{value: msg.value - amount}("");
            require(refundSuccess, "Refund failed");
        }

        emit Transfer(from_alice, to_bob, amount);
    }
}