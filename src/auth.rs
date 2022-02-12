use std::sync::{Arc, Mutex};

use crate::db::DbConnection;

use anyhow::Result;

#[derive(Clone)]
pub struct Auth {
    db: Arc<Mutex<DbConnection>>
}

impl Auth {
    pub fn new(db: Arc<Mutex<DbConnection>>) -> Self {
        Auth { db }
    }

    pub fn check(&self, username: &str, password: &str) -> Result<()> {
        if self.db.lock().unwrap().get_user(username)? == password {
            Ok(())
        } else {
            Err(anyhow::Error::msg(format!("Wrong password for {username}")))
        }
    }
}
