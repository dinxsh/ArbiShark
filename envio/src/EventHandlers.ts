import {
  DemoMarket,
  QuoteUpdated,
  MarketCreated,
} from "generated";

// Market state entity
DemoMarket.QuoteUpdated.handler(async ({ event, context }) => {
  const entity = {
    id: `${event.chainId}_${event.block.number}_${event.logIndex}`,
    timestamp: event.params.timestamp,
    yesPrice: event.params.yesPrice,
    noPrice: event.params.noPrice,
    liquidity: event.params.liquidity,
    blockNumber: event.block.number,
    blockTimestamp: event.block.timestamp,
    transactionHash: event.transaction.hash,
  };

  context.Quote.set(entity);

  // Update latest market state
  const marketState = {
    id: "current",
    yesPrice: event.params.yesPrice,
    noPrice: event.params.noPrice,
    liquidity: event.params.liquidity,
    lastUpdate: event.block.timestamp,
    blockNumber: event.block.number,
  };

  context.MarketState.set(marketState);
});

DemoMarket.MarketCreated.handler(async ({ event, context }) => {
  const entity = {
    id: "market_info",
    question: event.params.question,
    initialYesPrice: event.params.yesPrice,
    initialNoPrice: event.params.noPrice,
    createdAt: event.block.timestamp,
    blockNumber: event.block.number,
  };

  context.MarketInfo.set(entity);
});
