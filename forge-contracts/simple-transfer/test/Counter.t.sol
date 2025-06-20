// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import {Test, console} from "forge-std/Test.sol";
import {SimpleTransfer} from "../src/Counter.sol";

contract SimpleTransferTest is Test {
    SimpleTransfer public simpleTransfer;
    address alice;
    address bob;
    uint256 constant INITIAL_BALANCE = 10 ether;

    function setUp() public {
        simpleTransfer = new SimpleTransfer();
        alice = makeAddr("alice");
        bob = makeAddr("bob");

        // Fund Alice with some ETH
        vm.deal(alice, INITIAL_BALANCE);
    }

    function test_SuccessfulTransfer() public {
        uint256 transferAmount = 1 ether;
        uint256 aliceBalanceBefore = alice.balance;
        uint256 bobBalanceBefore = bob.balance;

        // Execute transfer as Alice
        vm.prank(alice);
        simpleTransfer.transfer{value: transferAmount}(alice, bob, transferAmount);

        // Check balances after transfer
        assertEq(alice.balance, aliceBalanceBefore - transferAmount, "Alice's balance should decrease");
        assertEq(bob.balance, bobBalanceBefore + transferAmount, "Bob's balance should increase");
    }

    function test_InsufficientFunds() public {
        uint256 transferAmount = 1 ether;
        uint256 insufficientAmount = 0.5 ether;

        // Try to transfer with insufficient funds
        vm.prank(alice);
        vm.expectRevert("Insufficient ETH sent");
        simpleTransfer.transfer{value: insufficientAmount}(alice, bob, transferAmount);

        // Verify balances remain unchanged
        assertEq(alice.balance, INITIAL_BALANCE, "Alice's balance should remain unchanged");
        assertEq(bob.balance, 0, "Bob's balance should remain unchanged");
    }

    function test_TransferWithSig_Success() public {
        uint256 transferAmount = 1 ether;
        uint256 nonce = 1;
        address dave = makeAddr("dave");
        vm.deal(dave, INITIAL_BALANCE);

        uint256 alicePrivateKey = 0xA11CE; 
        alice = vm.addr(alicePrivateKey);

        // Alice signs the transfer
        bytes32 hash = keccak256(abi.encodePacked(alice, bob, transferAmount, nonce, address(simpleTransfer)));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePrivateKey, keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash)));
        bytes memory signature = abi.encodePacked(r, s, v);

        uint256 bobBalanceBefore = bob.balance;
        uint256 daveBalanceBefore = dave.balance;

        // Dave calls transferWithSig, pays gas and value
        vm.prank(dave);
        simpleTransfer.transferWithSig{value: transferAmount}(alice, bob, transferAmount, nonce, signature);

        // Bob should receive the amount
        assertEq(bob.balance, bobBalanceBefore + transferAmount, "Bob's balance should increase");
        // Dave's balance should decrease by transferAmount (gas cost ignored)
        assertEq(dave.balance, daveBalanceBefore - transferAmount, "Dave's balance should decrease");
    }
}