// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title DemoMarket
 * @notice Simple binary prediction market for ArbiShark demo on Arbitrum Sepolia
 * @dev Emits QuoteUpdated events for Envio HyperIndex to track
 */
contract DemoMarket {
    struct Market {
        string question;
        uint256 yesPrice;  // Price in basis points (0-10000 = 0-100%)
        uint256 noPrice;   // Price in basis points
        uint256 liquidity; // Total liquidity in USDC (6 decimals)
        bool active;
    }

    Market public market;
    address public owner;
    uint256 public lastUpdateTime;

    event QuoteUpdated(
        uint256 indexed timestamp,
        uint256 yesPrice,
        uint256 noPrice,
        uint256 liquidity
    );

    event MarketCreated(
        string question,
        uint256 yesPrice,
        uint256 noPrice
    );

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    constructor(string memory _question) {
        owner = msg.sender;
        market = Market({
            question: _question,
            yesPrice: 5000,  // 50%
            noPrice: 5000,   // 50%
            liquidity: 1000 * 1e6,  // 1000 USDC
            active: true
        });
        lastUpdateTime = block.timestamp;
        
        emit MarketCreated(_question, 5000, 5000);
        emit QuoteUpdated(block.timestamp, 5000, 5000, 1000 * 1e6);
    }

    /**
     * @notice Update market prices (for demo purposes)
     * @param _yesPrice New YES price in basis points
     * @param _noPrice New NO price in basis points
     */
    function updatePrices(uint256 _yesPrice, uint256 _noPrice) external onlyOwner {
        require(market.active, "Market not active");
        require(_yesPrice <= 10000 && _noPrice <= 10000, "Invalid prices");
        
        market.yesPrice = _yesPrice;
        market.noPrice = _noPrice;
        lastUpdateTime = block.timestamp;
        
        emit QuoteUpdated(block.timestamp, _yesPrice, _noPrice, market.liquidity);
    }

    /**
     * @notice Update liquidity (for demo purposes)
     * @param _liquidity New liquidity amount
     */
    function updateLiquidity(uint256 _liquidity) external onlyOwner {
        require(market.active, "Market not active");
        
        market.liquidity = _liquidity;
        lastUpdateTime = block.timestamp;
        
        emit QuoteUpdated(block.timestamp, market.yesPrice, market.noPrice, _liquidity);
    }

    /**
     * @notice Simulate price movement (for demo/testing)
     * @dev Randomly adjusts prices within ±5% range
     */
    function simulatePriceMovement() external onlyOwner {
        require(market.active, "Market not active");
        
        // Simple pseudo-random price movement
        uint256 randomness = uint256(keccak256(abi.encodePacked(block.timestamp, block.prevrandao, msg.sender)));
        int256 change = int256(randomness % 500) - 250; // ±2.5%
        
        int256 newYesPrice = int256(market.yesPrice) + change;
        int256 newNoPrice = int256(market.noPrice) - change;
        
        // Clamp to valid range
        if (newYesPrice < 100) newYesPrice = 100;
        if (newYesPrice > 9900) newYesPrice = 9900;
        if (newNoPrice < 100) newNoPrice = 100;
        if (newNoPrice > 9900) newNoPrice = 9900;
        
        market.yesPrice = uint256(newYesPrice);
        market.noPrice = uint256(newNoPrice);
        lastUpdateTime = block.timestamp;
        
        emit QuoteUpdated(block.timestamp, uint256(newYesPrice), uint256(newNoPrice), market.liquidity);
    }

    /**
     * @notice Toggle market active status
     */
    function toggleActive() external onlyOwner {
        market.active = !market.active;
    }

    /**
     * @notice Get current market state
     */
    function getMarket() external view returns (
        string memory question,
        uint256 yesPrice,
        uint256 noPrice,
        uint256 liquidity,
        bool active
    ) {
        return (
            market.question,
            market.yesPrice,
            market.noPrice,
            market.liquidity,
            market.active
        );
    }

    /**
     * @notice Check if market has arbitrage opportunity
     * @dev Returns true if YES + NO != 10000 (100%)
     */
    function hasArbitrage() external view returns (bool, int256 spread) {
        int256 sum = int256(market.yesPrice + market.noPrice);
        spread = sum - 10000;
        return (spread != 0, spread);
    }
}
