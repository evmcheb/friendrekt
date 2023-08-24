pragma solidity ^0.8.10;

interface FriendtechSharesV1 {
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event Trade(
        address trader,
        address subject,
        bool isBuy,
        uint256 shareAmount,
        uint256 ethAmount,
        uint256 protocolEthAmount,
        uint256 subjectEthAmount,
        uint256 supply
    );

    function buyShares(address sharesSubject, uint256 amount) external payable;
    function getBuyPrice(address sharesSubject, uint256 amount) external view returns (uint256);
    function getBuyPriceAfterFee(address sharesSubject, uint256 amount) external view returns (uint256);
    function getPrice(uint256 supply, uint256 amount) external pure returns (uint256);
    function getSellPrice(address sharesSubject, uint256 amount) external view returns (uint256);
    function getSellPriceAfterFee(address sharesSubject, uint256 amount) external view returns (uint256);
    function owner() external view returns (address);
    function protocolFeeDestination() external view returns (address);
    function protocolFeePercent() external view returns (uint256);
    function renounceOwnership() external;
    function sellShares(address sharesSubject, uint256 amount) external payable;
    function setFeeDestination(address _feeDestination) external;
    function setProtocolFeePercent(uint256 _feePercent) external;
    function setSubjectFeePercent(uint256 _feePercent) external;
    function sharesBalance(address, address) external view returns (uint256);
    function sharesSupply(address) external view returns (uint256);
    function subjectFeePercent() external view returns (uint256);
    function transferOwnership(address newOwner) external;
}

