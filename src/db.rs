use std::collections::HashMap;

use anyhow::Result;
use rusqlite::{params, Connection};

pub struct DbConnection {
    conn: Connection,
}

impl TryFrom<Connection> for DbConnection {
    type Error = anyhow::Error;

    fn try_from(conn: Connection) -> Result<Self, Self::Error> {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS pending_runs (
                id TEXT NOT NULL PRIMARY KEY,
                booked_by TEXT
            )
        "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                username TEXT NOT NULL PRIMARY KEY,
                password TEXT
            )
            "#,
            [],
        )?;

        Ok(DbConnection { conn })
    }
}

impl DbConnection {
    pub fn get_bookings(&mut self) -> Result<HashMap<String, Option<String>>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, booked_by
            FROM
                pending_runs
        "#,
        )?;

        let qm = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
            ))
        })?;
        qm.map(|v| Ok(v?)).collect()
    }

    pub fn book_run(&mut self, run_id: &str, booked_by: &str) -> Result<()> {
        let query = r#"
            INSERT OR REPLACE INTO pending_runs (id, booked_by) VALUES (?1, ?2)
        "#;

        self.conn.execute(query, params![run_id, booked_by])?;

        Ok(())
    }

    pub fn unbook_run(&mut self, run_id: &str) -> Result<()> {
        let query = r#"
            UPDATE pending_runs SET booked_by = null WHERE id = ?1
        "#;

        self.conn.execute(query, params![run_id])?;

        Ok(())
    }

    pub fn get_user(&mut self, user_id: &str) -> Result<String> {
        let query = r#"SELECT password FROM users WHERE username = ?1"#;

        Ok(self
            .conn
            .query_row(query, params![user_id], |row| row.get(0))?)
    }
}
