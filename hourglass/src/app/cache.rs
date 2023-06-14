use std::collections::HashMap;
use std::sync::Mutex;

use crate::app::issues::RepoIssue;

lazy_static! {
    pub static ref ISSUES_CACHE: Mutex<HashMap<&'static str, Vec<RepoIssue>>> =
        Mutex::new(HashMap::new());
}

pub fn update_issues_cache() {}
