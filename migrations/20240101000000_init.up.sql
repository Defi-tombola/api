CREATE TABLE account (
    id UUID PRIMARY KEY,
    address TEXT NOT NULL,
    avatar TEXT,
    name TEXT,
    twitter TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE asset (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    asset_type VARCHAR(10) NOT NULL,
    symbol TEXT NOT NULL,
    shadow_symbol TEXT,
    decimals SMALLINT,
    deprecated BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE lottery (
    id UUID PRIMARY KEY,
    uid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    ticket_price DECIMAL NOT NULL,
    fee_ticket_amount DECIMAL NOT NULL,
    ticket_asset UUID NOT NULL REFERENCES asset(id),
    max_tickets INT,
    status VARCHAR(40) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE ticket (
    id UUID PRIMARY KEY,
    lottery_id UUID NOT NULL REFERENCES lottery(id),
    account_id UUID NOT NULL REFERENCES account(id),
    ticket_price DECIMAL NOT NULL,
    ticket_asset UUID NOT NULL REFERENCES asset(id),
    amount INT NOT NULL,
    purchased_at TIMESTAMPTZ NOT NULL,
    transaction_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE prize (
    id UUID PRIMARY KEY,
    lottery_id UUID NOT NULL REFERENCES lottery(id),
    prize_asset UUID NOT NULL REFERENCES asset(id),
    value DECIMAL NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE draw (
    id UUID PRIMARY KEY,
    lottery_id UUID NOT NULL REFERENCES lottery(id),
    winner UUID REFERENCES account(id),
    draw_date TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL,
    transaction_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_account_address ON account(address);
CREATE INDEX idx_asset_address ON asset(address);
CREATE INDEX idx_ticket_lottery_id ON ticket(lottery_id);
CREATE INDEX idx_ticket_account_id ON ticket(account_id);
CREATE UNIQUE INDEX idx_prize_lottery_id ON prize(lottery_id);
CREATE UNIQUE INDEX idx_draw_lottery_id ON draw(lottery_id);

CREATE TABLE transaction_log (
    id UUID PRIMARY KEY, 
    chain TEXT NOT NULL, 
    address TEXT NOT NULL, 
    block_number BIGINT NOT NULL, 
    transaction_hash TEXT NOT NULL, 
    log_index INTEGER NOT NULL, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() 
);

CREATE UNIQUE INDEX idx_transaction_log_unique ON transaction_log (transaction_hash, log_index);

CREATE TABLE transaction_log_side_effect (
    id UUID PRIMARY KEY, 
    transaction_log_id UUID NOT NULL REFERENCES transaction_log(id), 
    entity_id UUID NOT NULL, 
    entity_type TEXT NOT NULL, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() 
);

CREATE INDEX idx_transaction_log_side_effect_transaction_log_id ON transaction_log_side_effect (transaction_log_id);

CREATE INDEX idx_transaction_log_side_effect_entity ON transaction_log_side_effect (entity_id, entity_type);

CREATE TABLE chain_state (
    id UUID PRIMARY KEY, 
    chain TEXT NOT NULL, 
    value JSONB NOT NULL, 
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW() 
);

CREATE UNIQUE INDEX idx_chain_state_unique ON chain_state (chain);