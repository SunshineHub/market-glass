PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS positions (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('fund', 'advisory', 'cash')),
    code TEXT,
    name TEXT NOT NULL,
    units TEXT NOT NULL DEFAULT '0',
    total_cost TEXT NOT NULL DEFAULT '0',
    manual_value TEXT,
    manual_day_percent TEXT,
    provider TEXT NOT NULL DEFAULT 'manual',
    strategy TEXT NOT NULL DEFAULT '未分类',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_positions_kind ON positions(kind);
CREATE UNIQUE INDEX IF NOT EXISTS idx_positions_fund_code
    ON positions(code) WHERE kind = 'fund' AND code IS NOT NULL;

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS quote_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_code TEXT NOT NULL,
    value TEXT NOT NULL,
    change_percent TEXT NOT NULL,
    data_nature TEXT NOT NULL,
    provider TEXT NOT NULL,
    source_time TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_quote_snapshots_code_time
    ON quote_snapshots(asset_code, source_time DESC);
