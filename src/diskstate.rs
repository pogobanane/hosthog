use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use crate::users;

const STATE_PATH: &str = "/tmp/hosthog.json";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Claim {
    pub timeout: DateTime<Local>,
    pub soft_timeout: Option<DateTime<Local>>,
    pub exclusive: bool,
    pub user: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Settings {
    /// This should be the same as AuthorizedKeysFile in /etc/ssh/sshd_config (see man
    /// sshd_config)
    pub authorized_keys_file: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DiskState {
    /// paths of all files that are bind-mounted to /dev/null
    pub overmounts: Vec<String>,
    /// current claims
    pub claims: Vec<Claim>,
    /// settings to be modified by users
    pub settings: Settings,
}

pub fn load() -> DiskState {
    if !std::path::Path::new(STATE_PATH).is_file() {
        let defaults = load_default();
        store(&defaults);
    }

    let text = std::fs::read_to_string(STATE_PATH).expect("failed to read state file");
    let state: DiskState = serde_json::from_str(&text).unwrap();
    return state;
}

pub fn store(state: &DiskState) {
    if !users::is_root() {
        panic!("must be root to update hosts hogging state");
    }
    let json = serde_json::to_string(&state).unwrap();
    std::fs::write(STATE_PATH, json).expect("failed to write state file");
}

pub fn load_default() -> DiskState {
    let state = DiskState {
        overmounts: vec![],
        claims: vec![],
        settings: Settings {
            authorized_keys_file: vec![
                String::from("%h/.ssh/authorized_keys"),
                String::from("/etc/ssh/authorized_keys.d/%u"),
            ],
        },
    };

    return state;
}

use crate::hog;

pub fn expand_authorized_keys_file(settings: &Settings, users: Vec<hog::User>) -> Vec<String> {
    let mut files = vec![];
    for user in users {
        for file in &settings.authorized_keys_file {
            // replace %h with home directory
            let file = file.replacen("%h", &user.home, 1);
            // replace %u with username
            let file = file.replacen("%u", &user.name, 1);
            files.push(file);
            // TODO other replacements and respect escaped %: %%
        }
    }
    return files;
}

/// remove all claims that have timed out
pub fn maintenance(state: &mut DiskState) {
    let now = Local::now();
    let mut new_claims = vec![];
    for claim in &state.claims {
        if claim.timeout > now {
            new_claims.push(claim.clone());
        }
    }
    state.claims = new_claims;
}
