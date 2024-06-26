use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use crate::users;
use crate::util;
use once_cell::sync::Lazy;

static STATE_FILE: Lazy<String> = Lazy::new(|| format!("{}/hosthog.json", util::STATE_PATH));

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
    // Claim under which the system is currently hogged
    pub hogger: Option<Claim>,
    /// paths of all files that are bind-mounted to /dev/null
    /// If unmounting failed for some, the system may not be hogged but this list may contain
    /// items.
    pub overmounts: Vec<String>,
    /// current claims
    pub claims: Vec<Claim>,
    /// settings to be modified by users
    pub settings: Settings,
    pub disabled_systemd_timers: Vec<String>,
}

pub fn load() -> DiskState {
    if !std::path::Path::new(STATE_FILE.as_str()).is_file() {
        let defaults = load_default();
        store(&defaults);
    }

    let text = std::fs::read_to_string(STATE_FILE.as_str()).expect("failed to read state file");
    let state: DiskState = serde_json::from_str(&text).unwrap();
    return state;
}

pub fn store(state: &DiskState) {
    if !users::is_root() {
        panic!("must be root to update hosts hogging state");
    }
    let json = serde_json::to_string(&state).unwrap();
    // create parent directory if it does not exist
    let parent = std::path::Path::new(STATE_FILE.as_str()).parent().unwrap();
    if !parent.is_dir() {
        std::fs::create_dir_all(parent).expect("failed to create state directory");
    }
    std::fs::write(STATE_FILE.as_str(), json).expect("failed to write state file");
}

pub fn load_default() -> DiskState {
    let state = DiskState {
        hogger: None,
        overmounts: vec![],
        claims: vec![],
        settings: Settings {
            authorized_keys_file: vec![
                String::from("%h/.ssh/authorized_keys"),
                String::from("/etc/ssh/authorized_keys.d/%u"),
            ],
        },
        disabled_systemd_timers: vec![],
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
pub fn maintenance(state: &mut DiskState, needs_release: &mut bool) {
    let now = Local::now();
    let mut new_claims = vec![];
    let mut dropped_claims = vec![];
    for claim in &state.claims {
        if claim.timeout > now {
            new_claims.push(claim.clone());
        } else {
            dropped_claims.push(claim.clone());
        }
    }
    if let Some(hogger) = &state.hogger {
        for claim in &dropped_claims {
            if claim == hogger {
                // we just dropped the claim responsible for a current hogging
                *needs_release = true;
            }
        }
    }
    state.claims = new_claims;
    println!("Maintenance: {} claims expired, {} hogs released", dropped_claims.len(), if *needs_release { 1 } else { 0 });
}
