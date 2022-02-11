use anyhow::Result;
use rusqlite::{params, Connection};

use crate::PendingRun;

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
                weblink TEXT,
                comment TEXT,
                player_name TEXT,
                player_location TEXT,
                player_url TEXT,
                booked_by TEXT,
                submitted TEXT,
                times TEXT,
                deleted INT DEFAULT 0
            )
        "#,
            [],
        )?;

        Ok(DbConnection { conn })
    }
}

impl DbConnection {
    pub fn get_pending_runs(&mut self) -> Result<Vec<PendingRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, weblink, comment, player_name,
                player_location, player_url, booked_by, submitted, times
            FROM
                pending_runs
            WHERE deleted = 0
            ORDER BY submitted DESC
        "#,
        )?;

        let qm = stmt.query_map([], |row| {
            Ok(PendingRun {
                id: row.get(0)?,
                weblink: row.get(1)?,
                comment: row.get(2)?,
                player_name: row.get(3)?,
                player_location: row.get(4)?,
                player_url: row.get(5)?,
                booked_by: row.get(6)?,
                submitted: row.get(7)?,
                times: row.get(8)?,
            })
        })?;
        qm.map(|v| Ok(v?)).collect()
    }

    pub fn get_deleted_runs(&mut self) -> Result<Vec<PendingRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, weblink, comment, player_name,
                player_location, player_url, booked_by, submitted, times
            FROM
                pending_runs
            WHERE deleted = 1
            ORDER BY submitted DESC
        "#,
        )?;

        let qm = stmt.query_map([], |row| {
            Ok(PendingRun {
                id: row.get(0)?,
                weblink: row.get(1)?,
                comment: row.get(2)?,
                player_name: row.get(3)?,
                player_location: row.get(4)?,
                player_url: row.get(5)?,
                booked_by: row.get(6)?,
                submitted: row.get(7)?,
                times: row.get(8)?,
            })
        })?;
        qm.map(|v| Ok(v?)).collect()
    }

    pub fn put_pending_runs(&mut self, runs: &[PendingRun]) -> Result<()> {
        let mut stmt = self.conn.prepare(
            r#"
            INSERT OR IGNORE INTO pending_runs (
                id, weblink, comment, player_name,
                player_location, player_url, booked_by, submitted, times
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
            )
        "#,
        )?;

        self.conn.execute("BEGIN", [])?;

        for run in runs {
            let PendingRun {
                id,
                weblink,
                comment,
                player_name,
                player_location,
                player_url,
                booked_by,
                submitted,
                times,
            } = run;
            if let Err(e) = stmt.execute(params![
                id,
                weblink,
                comment,
                player_name,
                player_location,
                player_url,
                booked_by,
                submitted,
                times,
            ]) {
                self.conn.execute("ROLLBACK", [])?;
                return Err(anyhow::Error::from(e));
            }
        }
        self.conn.execute("COMMIT", [])?;

        Ok(())
    }

    pub fn undelete_run(&mut self, id: &str) -> Result<()> {
        self.conn.execute(
            r#"UPDATE pending_runs SET deleted = 0 WHERE id = ?1"#,
            params![id],
        )?;

        Ok(())
    }

    pub fn delete_run(&mut self, id: &str) -> Result<()> {
        self.conn.execute(
            r#"UPDATE pending_runs SET deleted = 1 WHERE id = ?1"#,
            params![id],
        )?;

        Ok(())
    }

    pub fn book_run(&mut self, run_id: &str, booked_by: &str) -> Result<()> {
        let query = r#"
            UPDATE pending_runs SET booked_by = ?2 WHERE id = ?1
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
}
