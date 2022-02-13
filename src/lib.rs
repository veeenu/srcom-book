use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use self::db::DbConnection;

use anyhow::Result;

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
    pending_db: HashMap<String, String>,
    mut pending_online: HashMap<String, Vec<PendingRun>>,
) -> HashMap<String, Vec<PendingRun>> {
    for runs in pending_online.values_mut() {
        for mut run in runs {
            run.booked_by = pending_db.get(&run.id).cloned();
        }
    }
    pending_online
}

pub async fn book_run(id: String, booked_by: String, db: Arc<Mutex<DbConnection>>) -> Result<()> {
    db.lock().unwrap().book_run(&id, &booked_by)?;
    Ok(())
}

pub async fn unbook_run(id: String, booked_by: String, db: Arc<Mutex<DbConnection>>) -> Result<()> {
    db.lock().unwrap().unbook_run(&id, &booked_by)?;
    Ok(())
}
