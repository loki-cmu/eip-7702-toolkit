// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import  {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";

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

    function transferWithSig(
        address from_alice,
        address to_bob,
        uint256 amount,
        uint256 nonce,
        bytes calldata signature
    ) external payable {
        // 1. Construct the hash
        bytes32 hash = keccak256(abi.encodePacked(from_alice, to_bob, amount, nonce, address(this)));
        // 2. Recover the signer
        address signer = recover(hash, signature);
        require(signer == from_alice, "Invalid signature");
        // 3. Other checks
        require(msg.value >= amount, "Insufficient ETH sent");
        // 4. Transfer logic same as original transfer
        (bool success,) = to_bob.call{value: amount}("");
        require(success, "Transfer failed");
        // 5. Refund excess ETH if any
        if (msg.value > amount) {
            (bool refundSuccess,) = msg.sender.call{value: msg.value - amount}("");
            require(refundSuccess, "Refund failed");
        }
        emit Transfer(from_alice, to_bob, amount);
    }

    function recover(bytes32 hash, bytes memory signature) internal pure returns (address) {
        return ECDSA.recover(MessageHashUtils.toEthSignedMessageHash(hash), signature);
    }
}