use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lighter_common::prelude::*;

use super::internal::Auth;

#[derive(Clone)]
pub struct Authenticated {
    users: Arc<Mutex<BTreeMap<Uuid, Auth>>>,
}

impl Default for Authenticated {
    fn default() -> Self {
        Self {
            users: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl Authenticated {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, id: Uuid) -> Option<Auth> {
        self.users.lock().unwrap().get(&id).cloned()
    }

    pub async fn set(&self, id: Uuid, auth: &Auth) {
        self.users.lock().unwrap().insert(id, auth.clone());
    }

    pub async fn remove(&self, id: Uuid) {
        self.users.lock().unwrap().remove(&id);
    }

    pub async fn remove_delay(&self, id: Uuid, delay: Duration) {
        let s = self.clone();

        actix::spawn(async move {
            actix::clock::sleep(delay).await;

            s.remove(id).await;
        });
    }
}
