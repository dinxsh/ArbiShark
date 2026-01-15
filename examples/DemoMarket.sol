// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title DemoMarket - Simple YES/NO market for ArbiShark demo
contract DemoMarket {
    event QuoteUpdated(uint256 yesPrice, uint256 noPrice, uint256 liquidity);

    uint256 public yesPrice;
    uint256 public noPrice;
    uint256 public liquidity;

    constructor(uint256 _yes, uint256 _no, uint256 _liq) {
        yesPrice = _yes;
        noPrice = _no;
        liquidity = _liq;
        emit QuoteUpdated(_yes, _no, _liq);
    }

    function updateQuote(uint256 _yes, uint256 _no, uint256 _liq) external {
        yesPrice = _yes;
        noPrice = _no;
        liquidity = _liq;
        emit QuoteUpdated(_yes, _no, _liq);
    }
}
