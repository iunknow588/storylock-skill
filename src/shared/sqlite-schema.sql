PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS challenge_state (
  challenge_id TEXT PRIMARY KEY,
  identity_id TEXT NOT NULL,
  scope TEXT NOT NULL,
  status TEXT NOT NULL,
  expected_answer_digests_json TEXT NOT NULL,
  failure_count INTEGER NOT NULL DEFAULT 0,
  lock_until INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS session_store (
  session_id TEXT PRIMARY KEY,
  challenge_id TEXT NOT NULL,
  identity_id TEXT NOT NULL,
  scope TEXT NOT NULL,
  resource_scope_json TEXT NOT NULL,
  session_type TEXT NOT NULL,
  read_budget INTEGER NOT NULL DEFAULT 0,
  write_budget INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL,
  issued_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS request_store (
  request_id TEXT PRIMARY KEY,
  nonce TEXT NOT NULL,
  expiry INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS nonce_store (
  nonce TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  expiry INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS failure_window (
  identity_id TEXT PRIMARY KEY,
  window_start INTEGER NOT NULL,
  failure_count INTEGER NOT NULL DEFAULT 0,
  locked_until INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS answer_digest_set (
  identity_id TEXT NOT NULL,
  answer_digest TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (identity_id, answer_digest)
);

CREATE TABLE IF NOT EXISTS protected_story_objects (
  story_object_id TEXT PRIMARY KEY,
  encrypted_object_json TEXT NOT NULL,
  sensitivity TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
  audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_type TEXT NOT NULL,
  identity_id TEXT,
  story_object_id TEXT,
  request_id TEXT,
  result TEXT,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_nonce_store_expiry ON nonce_store(expiry);
CREATE INDEX IF NOT EXISTS idx_request_store_expiry ON request_store(expiry);
CREATE INDEX IF NOT EXISTS idx_session_store_identity_status ON session_store(identity_id, status);
CREATE INDEX IF NOT EXISTS idx_challenge_state_identity_status ON challenge_state(identity_id, status);
