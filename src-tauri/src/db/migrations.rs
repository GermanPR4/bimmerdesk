//! Numbered, sequential SQLite migrations. Never edit an already-applied
//! migration — add a new one instead (PROJECT_PRINCIPLES.md, Principio 12:
//! compatibilidad antes que ruptura). See PROJECT_PLAN.md section 9 and
//! `docs/architecture/database-er.md` for the schema this produces.

use rusqlite::Connection;

const MIGRATIONS: &[(&str, &str)] = &[("001_init", include_str!("../../migrations/001_init.sql"))];

pub fn apply_all(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?1)",
            [name],
            |row| row.get(0),
        )?;

        if already_applied {
            continue;
        }

        conn.execute_batch(sql)?;
        conn.execute("INSERT INTO schema_migrations (name) VALUES (?1)", [name])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_migrations_once_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        apply_all(&conn).unwrap();
        apply_all(&conn).unwrap(); // running twice must not fail or duplicate rows

        let vehicles_table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='vehicles')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(vehicles_table_exists);

        let applied_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(applied_count, 1);
    }
}
