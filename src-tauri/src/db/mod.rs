//! SQLite connection + migration runner. One schema per module
//! (PROJECT_PLAN.md section 9): each module owns its own tables and
//! `Repository`; nothing here queries across module boundaries.

mod migrations;

use rusqlite::Connection;

#[derive(Debug)]
pub enum DbError {
    Connection(String),
    Migration(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Connection(reason) => write!(f, "database connection failed: {reason}"),
            DbError::Migration(reason) => write!(f, "migration failed: {reason}"),
        }
    }
}

impl std::error::Error for DbError {}

/// Opens the app database at `path` (use `:memory:` for tests) and applies
/// any pending migrations.
pub fn open_and_migrate(path: &str) -> Result<Connection, DbError> {
    let conn = Connection::open(path).map_err(|e| DbError::Connection(e.to_string()))?;
    migrations::apply_all(&conn).map_err(|e| DbError::Migration(e.to_string()))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_in_memory_db_and_applies_migrations() {
        let conn = open_and_migrate(":memory:").unwrap();
        let vehicles_table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='vehicles')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(vehicles_table_exists);
    }
}
