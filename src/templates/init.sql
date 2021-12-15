CREATE TABLE IF NOT EXISTS {table_name} (
  account TEXT NOT NULL,
  tx_id INTEGER NOT NULL,
  tx_date DATE NOT NULL,
  amount NUMERIC(13,4) NOT NULL,
  balance NUMERIC(13,4) NOT NULL,
  vendor TEXT NOT NULL,
  digits TEXT NULL,
  transaction_type TEXT NOT NULL,
  category TEXT NULL,
  subcategory TEXT NULL,
  notes TEXT NULL,
  PRIMARY KEY (account, tx_id)
);;;

CREATE INDEX IF NOT EXISTS idx_import_tx_date ON {table_name} (tx_date);;;
CREATE INDEX IF NOT EXISTS idx_import_tx_vendor ON {table_name} (vendor);;;
CREATE INDEX IF NOT EXISTS idx_import_tx_category ON {table_name} (category);;;
CREATE INDEX IF NOT EXISTS idx_import_tx_type ON {table_name} (transaction_type);;;
