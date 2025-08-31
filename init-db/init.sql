-- CREATE TABLE IF NOT EXISTS prices (
--     id SERIAL PRIMARY KEY,
--     symbol VARCHAR(20) NOT NULL,
--     open FLOAT NOT NULL,
--     high FLOAT NOT NULL,
--     low FLOAT NOT NULL,
--     close FLOAT NOT NULL,
--     volume FLOAT NOT NULL,
--     timestamp BIGINT NOT NULL
-- );

CREATE TABLE IF NOT EXISTS trades (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    entry_price FLOAT NOT NULL,
    exit_price FLOAT,
    trade_size FLOAT NOT NULL CHECK (trade_size > 0),
    position_size FLOAT NOT NULL CHECK (position_size > 0),
    pnl FLOAT,
    entry_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    exit_time TIMESTAMPTZ,
    status VARCHAR(10) NOT NULL CHECK (status IN ('OPEN', 'CLOSED')),
    CHECK (
        (status = 'OPEN' AND exit_price IS NULL AND exit_time IS NULL) OR
        (status = 'CLOSED' AND exit_price IS NOT NULL AND exit_time IS NOT NULL)
    )
);

-- CREATE INDEX IF NOT EXISTS idx_prices_coin_timestamp ON prices(symbol, timestamp);
CREATE INDEX IF NOT EXISTS idx_trades_symbol_status ON trades(symbol, status);
CREATE INDEX IF NOT EXISTS idx_trades_entry_time ON trades(entry_time);
