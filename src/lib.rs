use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use self::db::DbConnection;
use self::srcom::get_pending_runs;

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

pub async fn fetch_and_store(db: Arc<Mutex<DbConnection>>) -> Result<()> {
    let pending_runs = get_pending_runs().await?;
    db.lock().unwrap().put_pending_runs(&pending_runs)?;
    Ok(())
}

pub fn merge_pendings(
    pending_db: Vec<PendingRun>,
    pending_online: Vec<PendingRun>,
) -> Vec<PendingRun> {
    let online_id_set = pending_online
        .into_iter()
        .map(|run| run.id)
        .collect::<HashSet<_>>();

    pending_db
        .into_iter()
        .filter(|run| online_id_set.contains(&run.id))
        .collect()
}

pub async fn book_run(id: String, booked_by: String, db: Arc<Mutex<DbConnection>>) -> Result<()> {
    if booked_by == "nobody" {
        db.lock().unwrap().unbook_run(&id)?;
    } else {
        db.lock().unwrap().book_run(&id, &booked_by)?;
    }
    Ok(())
}
