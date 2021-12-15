CREATE TABLE transactions (
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
);

CREATE INDEX idx_tx_date ON transactions (tx_date);
CREATE INDEX idx_vendor ON transactions (vendor);
CREATE INDEX idx_category ON transactions (category);
CREATE INDEX idx_tx_type ON transactions (transaction_type);
