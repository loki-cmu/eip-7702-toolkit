// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import {Test, console} from "forge-std/Test.sol";
import {SimpleTransfer} from "../src/Counter.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract SimpleTransferTest is Test {
    SimpleTransfer public simpleTransfer;
    address alice;
    uint256 alicePrivateKey;
    address bob;
    uint256 constant INITIAL_BALANCE = 10 ether;

    function setUp() public {
        simpleTransfer = new SimpleTransfer();
        (address _aliceAddr, uint256 _alicePrivateKey) = makeAddrAndKey("alice");
        alice = _aliceAddr;
        alicePrivateKey = _alicePrivateKey;
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

        // uint256 alicePrivateKey = 0xA11CE;
        // alice = vm.addr(alicePrivateKey);

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

    function signSignature(
        uint256 _alicePrivateKey,
        address _alice,
        address _bob,
        uint256 transferAmount,
        uint256 nonce,
        address contractAddr
    ) internal view returns (bytes memory) {
        bytes32 hash = keccak256(abi.encodePacked(_alice, _bob, transferAmount, nonce, contractAddr));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(_alicePrivateKey, keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash)));
        bytes memory sig = abi.encodePacked(r, s, v);
        return sig;
    }

    function test_VerifyProvidedSignature() public {
        // Test data provided by the user
        uint256 _alicePrivateKey = 0xd2eb31e7ec97467f3e382903851bced12ba44fc230d93cdddcfd7726c94a2f6e;
        address _alice = vm.addr(_alicePrivateKey);
        address dave = 0x87D20bDC94DCB9bF27cCf59E2B7be7C5afBac84b; // Dave's address
        uint256 transferAmount = 0.0001 ether;
        uint256 nonce = 1;
        address contractAddr = 0xA773b4CAfe39cf46e524F0f06e3Bd6C7eB396eba;

        // Call signSignature
        bytes memory signature = signSignature(_alicePrivateKey, _alice, dave, transferAmount, nonce, contractAddr);

        // Verify the signature
        console.logBytes(signature);

        // assertEq("", signature, "");
        bytes memory expected = hex"37235b5a4ca2938d6e8c22e137fbf2bc4c8909bbc9f4b4097848914129e4828e6f56cfbd4994b11551f328d49190cf77af99be28b98dfc7489637b5b8ddf227c1c";
        assertEq(signature, expected, "Signature does not match expected value");
    }
}