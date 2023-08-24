// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;
import "./Guard.sol";
import {ERC20} from "solmate/tokens/ERC20.sol";
import {SafeTransferLib} from "solmate/utils/SafeTransferLib.sol";
import {FriendtechSharesV1} from "./IFriendTech.sol";

contract Sniper is Guard {
    FriendtechSharesV1 public ft;

    receive() external payable {}

    constructor() {
        ft = FriendtechSharesV1(0xCF205808Ed36593aa40a44F10c7f7C2F67d4A4d4);
        address EOA = address(0);
        allowlist[EOA] = true;
    }

    /***
     * @notice Snipe a single share 
     * @param shareSubject The address of the subjects
     * @param maxwant The maximum amount of shares to buy.
     * @param limit Revert if the current supply is greater than limit
     */
    function doSnipeManyShares(
        address[] calldata shareSubject, uint256[] calldata maxwant, uint256[] calldata limit
        ) external payable guard {
        uint256 length = shareSubject.length;
        for (uint256 i; i < length; i++) {
            // Check if the supply is 0
            uint256 supply = ft.sharesSupply(shareSubject[i]);
            if (supply == 0 || supply > limit[i]) continue;
            // Check our own balance
            uint256 balance = ft.sharesBalance(shareSubject[i], address(this));
            if (balance >= maxwant[i]) continue;
            // We want to buy. Iterate and see how much we can buy for our limit
            uint256 to_buy = maxwant[i] - balance;
            // Get out current eth balance
            uint256 eth_balance = address(this).balance;
            uint256 getBuyPriceAfterFee = ft.getBuyPriceAfterFee(shareSubject[i], to_buy);
            // Iterate down until we can buy
            while (eth_balance < getBuyPriceAfterFee) {
                to_buy -= 1;
                getBuyPriceAfterFee = ft.getBuyPriceAfterFee(shareSubject[i], to_buy);
            }
            ft.buyShares{value: getBuyPriceAfterFee}(shareSubject[i], to_buy);
        }
    }

    /***
     * @notice Buy n shares
     * @param shareSubject The address of the subject
     * @param maxwant The n of shares to buy.
     */
    function buyShares(address shareSubject, uint256 amount) external payable guard {
        uint256 getBuyPriceAfterFee = ft.getBuyPriceAfterFee(shareSubject, amount);
        ft.buyShares{value: getBuyPriceAfterFee}(shareSubject, amount);
    }

    /***
     * @notice Sell n shares
     * @param shareSubject The address of the subject
     * @param maxwant The n of shares to sell.
     */
    function sellShares(address shareSubject, uint256 amount) external guard {
        ft.sellShares(shareSubject, amount);
    }

    /***
     * @notice Withdraw all ETH from the contract
     * @dev Only callable by the owner
     */
    function returnETH() external payable onlyowner {
        (bool success, ) = payable(msg.sender).call{value: address(this).balance}("");
        require(success);
    }

    /***
     * @notice Admin func
     */
    function execute(address target, bytes memory data, uint256 _value) external onlyowner {
        (bool success,) = target.call{value: _value}(data);
        require(success, "call failed");
    }

    /***
     * @notice Set friend.tech contract
     */
    function setFt(address _ft) external onlyowner {
        ft = FriendtechSharesV1(_ft);
    }
}
