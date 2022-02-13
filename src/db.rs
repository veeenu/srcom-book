use std::collections::{HashMap, HashSet};

use crate::PendingRun;

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
    pub fn get_bookings(&mut self) -> Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, booked_by
            FROM
                pending_runs
        "#,
        )?;

        let qm = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        qm.map(|v| Ok(v?)).collect()
    }

    pub fn book_run(&mut self, run_id: &str, booked_by: &str) -> Result<()> {
        let query = r#"
            INSERT INTO pending_runs (id, booked_by) VALUES (?1, ?2)
        "#;

        self.conn.execute(query, params![run_id, booked_by])?;

        Ok(())
    }

    pub fn unbook_run(&mut self, run_id: &str, user_name: &str) -> Result<()> {
        let query = r#"
            DELETE FROM pending_runs WHERE id = ?1 AND booked_by = ?2
        "#;

        if self.conn.execute(query, params![run_id, user_name])? == 1 {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Could not unbook run"))
        }
    }

    pub fn cleanup(&mut self, runs: &HashMap<String, Vec<PendingRun>>) -> Result<()> {
        let db_runs = self
            .get_bookings()?
            .values()
            .cloned()
            .collect::<HashSet<_>>();
        let pending_run_ids = runs
            .values()
            .flat_map(|runs| runs.iter().map(|run| run.id.clone()))
            .collect::<HashSet<_>>();
        let stale_runs = db_runs.difference(&pending_run_ids);

        self.conn.execute("BEGIN", [])?;
        let mut stmt = self.conn.prepare(r#"DELETE FROM pending_runs WHERE id = ?1"#)?;
        for stale_run in stale_runs {
            stmt.execute(params![stale_run])?;
        }
        self.conn.execute("COMMIT", [])?;

        Ok(())
    }
}
