// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Guard {
    mapping(address => bool) allowlist;
    address public immutable owner;

    constructor() {
        owner = msg.sender;
        allowlist[msg.sender] = true;
    }

    modifier guard() {
        require(allowlist[msg.sender], "Guard: not allowlisted");
        _;
    }

    modifier onlyowner() {
        require(msg.sender == owner, "Guard: not owner");
        _;
    }
}
