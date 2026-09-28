import sqlite3
from pathlib import Path

sql = (Path(__file__).parents[1] / 'src-tauri/migrations/001_initial.sql').read_text()
conn = sqlite3.connect(':memory:')
conn.execute('PRAGMA foreign_keys=ON')
conn.executescript('CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY,name TEXT NOT NULL UNIQUE,checksum TEXT NOT NULL,applied_at TEXT NOT NULL) STRICT;')
conn.executescript('BEGIN IMMEDIATE;\n' + sql + '\nINSERT INTO schema_migrations VALUES(1,"001_initial","test","now");\nCOMMIT;')
assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
assert conn.execute('SELECT COUNT(*) FROM schema_migrations').fetchone()[0] == 1
assert conn.execute("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='students'").fetchone()[0] == 1
print('SQL_MIGRATION_OK')
