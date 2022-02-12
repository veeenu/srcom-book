use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use self::db::DbConnection;

use anyhow::Result;

pub mod auth;
pub mod db;
pub mod srcom;

#[derive(Debug, serde::Serialize)]
pub struct PendingRun {
    id: String,
    weblink: String,
    comment: String,
    player_name: String,
    player_location: String,
    player_url: String,
    booked_by: Option<String>,
    submitted: String,
    times: String,
}

pub fn merge_pendings(
    pending_db: HashMap<String, Option<String>>,
    mut pending_online: Vec<PendingRun>,
) -> Vec<PendingRun> {
    for mut run in &mut pending_online {
        run.booked_by = pending_db.get(&run.id).cloned().flatten();
    }
    pending_online
}

pub async fn book_run(id: String, booked_by: String, db: Arc<Mutex<DbConnection>>) -> Result<()> {
    if booked_by == "nobody" {
        db.lock().unwrap().unbook_run(&id)?;
    } else {
        db.lock().unwrap().book_run(&id, &booked_by)?;
    }
    Ok(())
}
